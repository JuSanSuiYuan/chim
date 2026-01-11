use std::collections::{HashMap, VecDeque};
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Waker};

pub type TaskId = u64;

#[derive(Debug, Clone, PartialEq)]
pub enum TaskState {
    Ready,
    Running,
    Waiting,
    Completed,
    Failed(String),
}

pub struct Task {
    pub id: TaskId,
    pub state: TaskState,
    pub future: Option<Box<dyn Future<Output = ()> + Send>>,
    pub waker: Option<Waker>,
}

impl Task {
    pub fn new(id: TaskId, future: Box<dyn Future<Output = ()> + Send>) -> Self {
        Task {
            id,
            state: TaskState::Ready,
            future: Some(future),
            waker: None,
        }
    }

    pub fn poll(&mut self, cx: &mut Context<'_>) -> Poll<()> {
        if let Some(future) = self.future.as_mut() {
            let pinned = Pin::new(future);
            pinned.poll(cx)
        } else {
            Poll::Ready(())
        }
    }
}

pub struct Executor {
    ready_queue: VecDeque<TaskId>,
    tasks: Arc<Mutex<HashMap<TaskId, Task>>>,
    next_task_id: TaskId,
}

impl Executor {
    pub fn new() -> Self {
        Executor {
            ready_queue: VecDeque::new(),
            tasks: Arc::new(Mutex::new(HashMap::new())),
            next_task_id: 1,
        }
    }

    pub fn spawn<F>(&mut self, future: F) -> TaskId
    where
        F: Future<Output = ()> + Send + 'static,
    {
        let id = self.next_task_id;
        self.next_task_id += 1;

        let task = Task::new(id, Box::new(future));
        {
            let mut tasks = self.tasks.lock().unwrap();
            tasks.insert(id, task);
        }
        self.ready_queue.push_back(id);
        id
    }

    pub fn wake_task(&mut self, id: TaskId) {
        let mut tasks = self.tasks.lock().unwrap();
        if let Some(task) = tasks.get_mut(&id) {
            if task.state == TaskState::Waiting {
                task.state = TaskState::Ready;
                self.ready_queue.push_back(id);
            }
        }
    }

    pub fn run_one(&mut self) -> bool {
        if let Some(task_id) = self.ready_queue.pop_front() {
            let mut tasks = self.tasks.lock().unwrap();
            if let Some(task) = tasks.get_mut(&task_id) {
                task.state = TaskState::Running;

                let waker = Waker::noop();
                let mut cx = Context::from_waker(&waker);

                match task.poll(&mut cx) {
                    Poll::Ready(()) => {
                        task.state = TaskState::Completed;
                        task.future = None;
                    }
                    Poll::Pending => {
                        task.state = TaskState::Waiting;
                    }
                }
            }
            true
        } else {
            false
        }
    }

    pub fn run(&mut self) {
        while self.run_one() {}
    }

    pub fn run_until(&mut self, condition: impl Fn() -> bool) {
        while !condition() {
            if !self.run_one() {
                break;
            }
        }
    }
}

pub struct Runtime {
    executor: Executor,
    timers: Vec<TimerEvent>,
}

impl Runtime {
    pub fn new() -> Self {
        Runtime {
            executor: Executor::new(),
            timers: Vec::new(),
        }
    }

    pub fn spawn<F>(&mut self, future: F) -> TaskId
    where
        F: Future<Output = ()> + Send + 'static,
    {
        self.executor.spawn(future)
    }

    pub fn block_on<F>(&mut self, future: F) -> F::Output
    where
        F: Future,
    {
        let waker = Waker::noop();
        let mut cx = Context::from_waker(&waker);
        let mut pinned = Pin::new(future);

        loop {
            match pinned.as_mut().poll(&mut cx) {
                Poll::Ready(val) => return val,
                Poll::Pending => {
                    if !self.executor.run_one() {
                        std::thread::sleep(std::time::Duration::from_millis(1));
                    }
                }
            }
        }
    }

    pub fn sleep(&mut self, duration: std::time::Duration) -> impl Future<Output = ()> + Send {
        struct SleepFuture {
            duration: std::time::Duration,
            start: Option<std::time::Instant>,
        }

        impl Future for SleepFuture {
            type Output = ();

            fn poll(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
                let start = self.start.get_or_insert_with(std::time::Instant::now);
                if start.elapsed() >= self.duration {
                    Poll::Ready(())
                } else {
                    Poll::Pending
                }
            }
        }

        SleepFuture {
            duration,
            start: None,
        }
    }
}

pub struct TimerEvent {
    pub deadline: std::time::Instant,
    pub task_id: TaskId,
}

pub struct JoinHandle<T> {
    task_id: TaskId,
    tasks: Arc<Mutex<HashMap<TaskId, Task>>>,
    _phantom: std::marker::PhantomData<T>,
}

impl<T> JoinHandle<T> {
    pub fn new(task_id: TaskId, tasks: Arc<Mutex<HashMap<TaskId, Task>>>) -> Self {
        JoinHandle {
            task_id,
            tasks,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T> Future for JoinHandle<T>
where
    T: Send + 'static,
{
    type Output = T;

    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        let tasks = self.tasks.lock().unwrap();
        if let Some(task) = tasks.get(&self.task_id) {
            if task.state == TaskState::Completed {
                Poll::Ready(unsafe { std::mem::zeroed() })
            } else {
                Poll::Pending
            }
        } else {
            Poll::Pending
        }
    }
}

pub mod channel {
    use std::sync::mpsc;
    use std::task::{Context, Poll, Waker};

    pub struct Sender<T> {
        inner: mpsc::Sender<T>,
    }

    pub struct Receiver<T> {
        inner: mpsc::Receiver<T>,
        waker: Option<Waker>,
    }

    impl<T> Sender<T> {
        pub fn send(&self, value: T) -> Result<(), mpsc::SendError<T>> {
            self.inner.send(value)
        }
    }

    impl<T> Receiver<T> {
        pub fn recv(&mut self) -> Poll<Option<T>> {
            match self.inner.try_recv() {
                Ok(value) => Poll::Ready(Some(value)),
                Err(mpsc::TryRecvError::Empty) => Poll::Pending,
                Err(mpsc::TryRecvError::Disconnected) => Poll::Ready(None),
            }
        }
    }

    pub fn channel<T>() -> (Sender<T>, Receiver<T>) {
        let (tx, rx) = mpsc::channel();
        (Sender { inner: tx }, Receiver { inner: rx, waker: None })
    }
}

pub mod sync {
    use std::sync::{Arc, Mutex};
    use std::task::{Context, Poll, Waker};

    pub struct MutexLock<T> {
        inner: Arc<Mutex<T>>,
        waker: Option<Waker>,
    }

    impl<T> MutexLock<T> {
        pub fn new(value: T) -> Self {
            MutexLock {
                inner: Arc::new(Mutex::new(value)),
                waker: None,
            }
        }

        pub fn lock(&mut self, _cx: &mut Context<'_>) -> Poll<MutexGuard<'_, T>> {
            if let Ok(guard) = self.inner.try_lock() {
                Poll::Ready(guard)
            } else {
                Poll::Pending
            }
        }
    }

    pub struct MutexGuard<'a, T> {
        inner: std::sync::MutexGuard<'a, T>,
    }

    impl<'a, T> std::ops::Deref for MutexGuard<'a, T> {
        type Target = T;

        fn deref(&self) -> &Self::Target {
            &self.inner
        }
    }

    impl<'a, T> std::ops::DerefMut for MutexGuard<'a, T> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.inner
        }
    }

    pub struct RwLock<T> {
        inner: Arc<parking_lot::RwLock<T>>,
    }

    impl<T> RwLock<T> {
        pub fn new(value: T) -> Self {
            RwLock {
                inner: Arc::new(parking_lot::RwLock::new(value)),
            }
        }

        pub fn read(&self) -> parking_lot::RwLockReadGuard<'_, T> {
            self.inner.read()
        }

        pub fn write(&self) -> parking_lot::RwLockWriteGuard<'_, T> {
            self.inner.write()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_executor_spawn() {
        let mut executor = Executor::new();
        let task_id = executor.spawn(async {});
        assert!(task_id > 0);
    }

    #[test]
    fn test_executor_run() {
        let mut executor = Executor::new();
        executor.spawn(async {
            println!("Task executed");
        });
        executor.run();
    }

    #[test]
    fn test_runtime_block_on() {
        let mut runtime = Runtime::new();
        let result = runtime.block_on(async { 42 });
        assert_eq!(result, 42);
    }

    #[test]
    fn test_channel() {
        let (tx, mut rx) = channel::channel::<i32>();
        tx.send(42).unwrap();
        let mut cx = Context::from_waker(&Waker::noop());
        match rx.recv() {
            Poll::Ready(Some(val)) => assert_eq!(val, 42),
            _ => panic!("Expected value"),
        }
    }
}
