use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread::{self, ThreadId};

pub type ThreadLocalResult<T> = Result<T, ThreadLocalError>;

#[derive(Debug, Clone)]
pub enum ThreadLocalError {
    NotInitialized,
    AlreadyInitialized,
    InvalidThreadId,
    PoisonError(String),
}

impl std::fmt::Display for ThreadLocalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ThreadLocalError::NotInitialized => write!(f, "Thread local not initialized"),
            ThreadLocalError::AlreadyInitialized => {
                write!(f, "Thread local already initialized")
            }
            ThreadLocalError::InvalidThreadId => write!(f, "Invalid thread ID"),
            ThreadLocalError::PoisonError(msg) => write!(f, "Poison error: {}", msg),
        }
    }
}

impl std::error::Error for ThreadLocalError {}

pub struct ThreadLocal<T> {
    inner: Arc<Mutex<HashMap<ThreadId, T>>>,
    init: Option<Box<dyn Fn() -> T + Send + Sync>>,
}

impl<T: Clone + Send + 'static> ThreadLocal<T> {
    pub fn new() -> Self {
        ThreadLocal {
            inner: Arc::new(Mutex::new(HashMap::new())),
            init: None,
        }
    }

    pub fn with_initial<F>(init: F) -> Self
    where
        F: Fn() -> T + Send + Sync + 'static,
    {
        ThreadLocal {
            inner: Arc::new(Mutex::new(HashMap::new())),
            init: Some(Box::new(init)),
        }
    }

    pub fn get(&self) -> ThreadLocalResult<T> {
        let thread_id = thread::current().id();
        let inner = self.inner.lock().map_err(|e| {
            ThreadLocalError::PoisonError(e.to_string())
        })?;

        inner.get(&thread_id)
            .cloned()
            .ok_or(ThreadLocalError::NotInitialized)
    }

    pub fn get_or_init(&self) -> T {
        let thread_id = thread::current().id();
        let mut inner = self.inner.lock().unwrap();

        if let Some(value) = inner.get(&thread_id) {
            value.clone()
        } else {
            let value = if let Some(ref init) = self.init {
                init()
            } else {
                panic!("Thread local not initialized and no init function provided");
            };
            inner.insert(thread_id, value.clone());
            value
        }
    }

    pub fn set(&self, value: T) {
        let thread_id = thread::current().id();
        let mut inner = self.inner.lock().unwrap();
        inner.insert(thread_id, value);
    }

    pub fn remove(&self) -> ThreadLocalResult<T> {
        let thread_id = thread::current().id();
        let mut inner = self.inner.lock().map_err(|e| {
            ThreadLocalError::PoisonError(e.to_string())
        })?;

        inner.remove(&thread_id)
            .ok_or(ThreadLocalError::NotInitialized)
    }

    pub fn clear(&self) {
        let thread_id = thread::current().id();
        let mut inner = self.inner.lock().unwrap();
        inner.remove(&thread_id);
    }

    pub fn contains(&self) -> bool {
        let thread_id = thread::current().id();
        let inner = self.inner.lock().unwrap();
        inner.contains_key(&thread_id)
    }
}

impl<T> Drop for ThreadLocal<T> {
    fn drop(&mut self) {
        let mut inner = self.inner.lock().unwrap();
        inner.clear();
    }
}

pub struct ThreadLocalRefCell<T> {
    inner: ThreadLocal<RefCell<T>>,
}

impl<T: Send + 'static> ThreadLocalRefCell<T> {
    pub fn new() -> Self {
        ThreadLocalRefCell {
            inner: ThreadLocal::new(),
        }
    }

    pub fn with_initial<F>(init: F) -> Self
    where
        F: Fn() -> T + Send + Sync + 'static,
    {
        ThreadLocalRefCell {
            inner: ThreadLocal::with_initial(move || RefCell::new(init())),
        }
    }

    pub fn with<R, F>(&self, f: F) -> R
    where
        F: FnOnce(&T) -> R,
    {
        let cell = self.inner.get_or_init();
        let borrowed = cell.borrow();
        f(&borrowed)
    }

    pub fn with_mut<R, F>(&self, f: F) -> R
    where
        F: FnOnce(&mut T) -> R,
    {
        let cell = self.inner.get_or_init();
        let mut borrowed = cell.borrow_mut();
        f(&mut borrowed)
    }

    pub fn replace(&self, value: T) -> T {
        let cell = self.inner.get_or_init();
        cell.replace(value)
    }

    pub fn take(&self) -> T
    where
        T: Default,
    {
        let cell = self.inner.get_or_init();
        cell.take()
    }
}

pub struct ThreadLocalKey<T> {
    key: thread::LocalKey<T>,
}

impl<T: Send + 'static> ThreadLocalKey<T> {
    pub const fn new(init: T) -> Self {
        ThreadLocalKey {
            key: thread::LocalKey::new(init),
        }
    }

    pub fn get(&self) -> &T {
        self.key.get(thread::current())
    }

    pub fn get_mut(&self) -> &mut T {
        self.key.get_mut(thread::current())
    }

    pub fn with<R, F>(&self, f: F) -> R
    where
        F: FnOnce(&T) -> R,
    {
        f(self.get())
    }

    pub fn with_mut<R, F>(&self, f: F) -> R
    where
        F: FnOnce(&mut T) -> R,
    {
        f(self.get_mut())
    }
}

pub struct ThreadLocalCounter {
    counter: ThreadLocal<usize>,
}

impl ThreadLocalCounter {
    pub fn new() -> Self {
        ThreadLocalCounter {
            counter: ThreadLocal::with_initial(|| 0),
        }
    }

    pub fn increment(&self) {
        let mut inner = self.counter.inner.lock().unwrap();
        let thread_id = thread::current().id();
        let value = inner.entry(thread_id).or_insert(0);
        *value += 1;
    }

    pub fn decrement(&self) {
        let mut inner = self.counter.inner.lock().unwrap();
        let thread_id = thread::current().id();
        if let Some(value) = inner.get_mut(&thread_id) {
            if *value > 0 {
                *value -= 1;
            }
        }
    }

    pub fn get(&self) -> usize {
        self.counter.get_or_init()
    }

    pub fn reset(&self) {
        self.counter.set(0);
    }

    pub fn total(&self) -> usize {
        let inner = self.counter.inner.lock().unwrap();
        inner.values().sum()
    }
}

pub struct ThreadLocalBuffer<T> {
    buffer: ThreadLocal<Vec<T>>,
}

impl<T: Clone + Send + 'static> ThreadLocalBuffer<T> {
    pub fn new() -> Self {
        ThreadLocalBuffer {
            buffer: ThreadLocal::with_initial(Vec::new),
        }
    }

    pub fn push(&self, value: T) {
        let mut buffer = self.buffer.inner.lock().unwrap();
        let thread_id = thread::current().id();
        buffer.entry(thread_id).or_insert_with(Vec::new).push(value);
    }

    pub fn pop(&self) -> Option<T> {
        let mut buffer = self.buffer.inner.lock().unwrap();
        let thread_id = thread::current().id();
        buffer.get_mut(&thread_id)?.pop()
    }

    pub fn drain(&self) -> Vec<T> {
        let mut buffer = self.buffer.inner.lock().unwrap();
        let thread_id = thread::current().id();
        buffer.remove(&thread_id).unwrap_or_default()
    }

    pub fn len(&self) -> usize {
        let buffer = self.buffer.inner.lock().unwrap();
        let thread_id = thread::current().id();
        buffer.get(&thread_id).map(|v| v.len()).unwrap_or(0)
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn clear(&self) {
        let mut buffer = self.buffer.inner.lock().unwrap();
        let thread_id = thread::current().id();
        buffer.remove(&thread_id);
    }
}

pub struct ThreadLocalScope<F, R>
where
    F: FnOnce() -> R,
{
    f: F,
}

impl<F, R> ThreadLocalScope<F, R>
where
    F: FnOnce() -> R,
{
    pub fn new(f: F) -> Self {
        ThreadLocalScope { f }
    }

    pub fn run(self) -> R {
        self.f()
    }
}

pub fn scoped_thread_local<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    let scope = ThreadLocalScope::new(f);
    scope.run()
}

pub struct ThreadLocalPool<T> {
    pool: ThreadLocal<Vec<T>>,
    max_size: usize,
}

impl<T: Clone + Send + 'static> ThreadLocalPool<T> {
    pub fn new(max_size: usize) -> Self {
        ThreadLocalPool {
            pool: ThreadLocal::with_initial(Vec::new),
            max_size,
        }
    }

    pub fn acquire(&self) -> Option<T> {
        let mut pool = self.pool.inner.lock().unwrap();
        let thread_id = thread::current().id();
        pool.get_mut(&thread_id)?.pop()
    }

    pub fn release(&self, value: T) {
        let mut pool = self.pool.inner.lock().unwrap();
        let thread_id = thread::current().id();
        let buffer = pool.entry(thread_id).or_insert_with(Vec::new);
        if buffer.len() < self.max_size {
            buffer.push(value);
        }
    }

    pub fn size(&self) -> usize {
        self.pool.get_or_init().len()
    }

    pub fn clear(&self) {
        self.pool.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_thread_local_basic() {
        let tl = ThreadLocal::new();
        tl.set(42);
        assert_eq!(tl.get().unwrap(), 42);
    }

    #[test]
    fn test_thread_local_with_initial() {
        let tl = ThreadLocal::with_initial(|| 100);
        assert_eq!(tl.get_or_init(), 100);
    }

    #[test]
    fn test_thread_local_refcell() {
        let tl = ThreadLocalRefCell::new();
        tl.set(10);
        let value = tl.with(|v| *v);
        assert_eq!(value, 10);
    }

    #[test]
    fn test_thread_local_counter() {
        let counter = ThreadLocalCounter::new();
        counter.increment();
        counter.increment();
        assert_eq!(counter.get(), 2);
    }

    #[test]
    fn test_thread_local_buffer() {
        let buffer = ThreadLocalBuffer::new();
        buffer.push(1);
        buffer.push(2);
        assert_eq!(buffer.len(), 2);
        assert_eq!(buffer.pop(), Some(2));
    }

    #[test]
    fn test_thread_local_pool() {
        let pool = ThreadLocalPool::new(10);
        pool.release(42);
        assert_eq!(pool.acquire(), Some(42));
        assert_eq!(pool.acquire(), None);
    }
}
