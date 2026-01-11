use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, AtomicUsize, AtomicU8, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, Instant};

pub type SyncResult<T> = Result<T, SyncError>;

#[derive(Debug, Clone)]
pub enum SyncError {
    Timeout,
    Poisoned,
    Closed,
    InvalidState,
}

impl std::fmt::Display for SyncError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SyncError::Timeout => write!(f, "Operation timed out"),
            SyncError::Poisoned => write!(f, "Lock was poisoned"),
            SyncError::Closed => write!(f, "Synchronization primitive was closed"),
            SyncError::InvalidState => write!(f, "Invalid state"),
        }
    }
}

impl std::error::Error for SyncError {}

pub struct Semaphore {
    permits: AtomicUsize,
    waiters: VecDeque<Arc<Condvar>>,
    mutex: Mutex<()>,
}

impl Semaphore {
    pub fn new(permits: usize) -> Self {
        Semaphore {
            permits: AtomicUsize::new(permits),
            waiters: VecDeque::new(),
            mutex: Mutex::new(()),
        }
    }

    pub fn acquire(&self) -> SyncResult<()> {
        loop {
            let mut permits = self.permits.load(Ordering::Acquire);
            while permits > 0 {
                match self.permits.compare_exchange_weak(
                    permits,
                    permits - 1,
                    Ordering::AcqRel,
                    Ordering::Acquire,
                ) {
                    Ok(_) => return Ok(()),
                    Err(new_permits) => permits = new_permits,
                }
            }

            let _guard = self.mutex.lock().unwrap();
            let cv = Arc::new(Condvar::new());
            self.waiters.push_back(cv.clone());
            cv.wait(_guard).unwrap();
        }
    }

    pub fn try_acquire(&self) -> SyncResult<()> {
        let mut permits = self.permits.load(Ordering::Acquire);
        while permits > 0 {
            match self.permits.compare_exchange_weak(
                permits,
                permits - 1,
                Ordering::AcqRel,
                Ordering::Acquire,
            ) {
                Ok(_) => return Ok(()),
                Err(new_permits) => permits = new_permits,
            }
        }
        Err(SyncError::Timeout)
    }

    pub fn acquire_timeout(&self, timeout: Duration) -> SyncResult<()> {
        let deadline = Instant::now() + timeout;
        loop {
            let mut permits = self.permits.load(Ordering::Acquire);
            while permits > 0 {
                match self.permits.compare_exchange_weak(
                    permits,
                    permits - 1,
                    Ordering::AcqRel,
                    Ordering::Acquire,
                ) {
                    Ok(_) => return Ok(()),
                    Err(new_permits) => permits = new_permits,
                }
            }

            let guard = self.mutex.lock().unwrap();
            let cv = Arc::new(Condvar::new());
            self.waiters.push_back(cv.clone());

            let remaining = deadline.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                return Err(SyncError::Timeout);
            }

            let result = cv.wait_timeout(guard, remaining).unwrap();
            if result.timed_out() {
                return Err(SyncError::Timeout);
            }
        }
    }

    pub fn release(&self) {
        self.permits.fetch_add(1, Ordering::Release);
        let mut waiters = self.waiters.lock().unwrap();
        if let Some(waiter) = waiters.pop_front() {
            waiter.notify_one();
        }
    }

    pub fn available_permits(&self) -> usize {
        self.permits.load(Ordering::Acquire)
    }
}

pub struct Barrier {
    count: AtomicUsize,
    generation: AtomicUsize,
    mutex: Mutex<()>,
    condvar: Condvar,
}

impl Barrier {
    pub fn new(n: usize) -> Self {
        Barrier {
            count: AtomicUsize::new(n),
            generation: AtomicUsize::new(0),
            mutex: Mutex::new(()),
            condvar: Condvar::new(),
        }
    }

    pub fn wait(&self) -> SyncResult<()> {
        let mut guard = self.mutex.lock().unwrap();
        let generation = self.generation.load(Ordering::Acquire);
        let count = self.count.fetch_sub(1, Ordering::AcqRel);

        if count == 1 {
            self.generation.fetch_add(1, Ordering::AcqRel);
            self.count.store(self.count.load(Ordering::Acquire) + count, Ordering::Release);
            self.condvar.notify_all();
            Ok(())
        } else {
            loop {
                self.condvar.wait(guard).unwrap();
                if self.generation.load(Ordering::Acquire) != generation {
                    return Ok(());
                }
            }
        }
    }

    pub fn wait_timeout(&self, timeout: Duration) -> SyncResult<()> {
        let deadline = Instant::now() + timeout;
        let mut guard = self.mutex.lock().unwrap();
        let generation = self.generation.load(Ordering::Acquire);
        let count = self.count.fetch_sub(1, Ordering::AcqRel);

        if count == 1 {
            self.generation.fetch_add(1, Ordering::AcqRel);
            self.count.store(self.count.load(Ordering::Acquire) + count, Ordering::Release);
            self.condvar.notify_all();
            Ok(())
        } else {
            loop {
                let remaining = deadline.saturating_duration_since(Instant::now());
                if remaining.is_zero() {
                    return Err(SyncError::Timeout);
                }

                let result = self.condvar.wait_timeout(guard, remaining).unwrap();
                if self.generation.load(Ordering::Acquire) != generation {
                    return Ok(());
                }

                if result.timed_out() {
                    return Err(SyncError::Timeout);
                }
            }
        }
    }
}

pub struct OnceCell<T> {
    state: AtomicU8,
    value: std::mem::MaybeUninit<T>,
}

const UNINITIALIZED: u8 = 0;
const INITIALIZING: u8 = 1;
const INITIALIZED: u8 = 2;

impl<T> OnceCell<T> {
    pub const fn new() -> Self {
        OnceCell {
            state: AtomicU8::new(UNINITIALIZED),
            value: std::mem::MaybeUninit::uninit(),
        }
    }

    pub fn get(&self) -> Option<&T> {
        if self.state.load(Ordering::Acquire) == INITIALIZED {
            unsafe { Some(self.value.assume_init_ref()) }
        } else {
            None
        }
    }

    pub fn get_mut(&mut self) -> Option<&mut T> {
        if self.state.load(Ordering::Acquire) == INITIALIZED {
            unsafe { Some(self.value.assume_init_mut()) }
        } else {
            None
        }
    }

    pub fn set(&self, value: T) -> Result<(), T> {
        match self.state.compare_exchange(
            UNINITIALIZED,
            INITIALIZING,
            Ordering::AcqRel,
            Ordering::Acquire,
        ) {
            Ok(_) => {
                unsafe {
                    self.value.write(value);
                }
                self.state.store(INITIALIZED, Ordering::Release);
                Ok(())
            }
            Err(_) => Err(value),
        }
    }

    pub fn get_or_init<F>(&self, f: F) -> &T
    where
        F: FnOnce() -> T,
    {
        if let Some(value) = self.get() {
            return value;
        }

        match self.state.compare_exchange(
            UNINITIALIZED,
            INITIALIZING,
            Ordering::AcqRel,
            Ordering::Acquire,
        ) {
            Ok(_) => {
                let value = f();
                unsafe {
                    self.value.write(value);
                }
                self.state.store(INITIALIZED, Ordering::Release);
                unsafe { self.value.assume_init_ref() }
            }
            Err(INITIALIZED) => unsafe { self.value.assume_init_ref() },
            Err(_) => {
                while self.state.load(Ordering::Acquire) != INITIALIZED {
                    std::hint::spin_loop();
                }
                unsafe { self.value.assume_init_ref() }
            }
        }
    }

    pub fn into_inner(self) -> Option<T> {
        if self.state.load(Ordering::Acquire) == INITIALIZED {
            unsafe { Some(self.value.assume_init()) }
        } else {
            None
        }
    }
}

unsafe impl<T: Send> Send for OnceCell<T> {}
unsafe impl<T: Sync> Sync for OnceCell<T> {}

pub struct Once {
    state: AtomicUsize,
}

const ONCE_NOT_STARTED: usize = 0;
const ONCE_IN_PROGRESS: usize = 1;
const ONCE_DONE: usize = 2;

impl Once {
    pub const fn new() -> Self {
        Once {
            state: AtomicUsize::new(ONCE_NOT_STARTED),
        }
    }

    pub fn call_once<F>(&self, f: F)
    where
        F: FnOnce(),
    {
        let mut state = self.state.load(Ordering::Acquire);
        while state == ONCE_NOT_STARTED {
            match self.state.compare_exchange_weak(
                ONCE_NOT_STARTED,
                ONCE_IN_PROGRESS,
                Ordering::AcqRel,
                Ordering::Acquire,
            ) {
                Ok(_) => {
                    f();
                    self.state.store(ONCE_DONE, Ordering::Release);
                    return;
                }
                Err(new_state) => state = new_state,
            }
        }

        while state == ONCE_IN_PROGRESS {
            state = self.state.load(Ordering::Acquire);
            std::hint::spin_loop();
        }
    }

    pub fn is_completed(&self) -> bool {
        self.state.load(Ordering::Acquire) == ONCE_DONE
    }
}

pub struct RwLock<T> {
    inner: parking_lot::RwLock<T>,
}

impl<T> RwLock<T> {
    pub fn new(value: T) -> Self {
        RwLock {
            inner: parking_lot::RwLock::new(value),
        }
    }

    pub fn read(&self) -> RwLockReadGuard<'_, T> {
        RwLockReadGuard {
            inner: self.inner.read(),
        }
    }

    pub fn try_read(&self) -> Option<RwLockReadGuard<'_, T>> {
        self.inner.try_read().map(|guard| RwLockReadGuard { inner: guard })
    }

    pub fn write(&self) -> RwLockWriteGuard<'_, T> {
        RwLockWriteGuard {
            inner: self.inner.write(),
        }
    }

    pub fn try_write(&self) -> Option<RwLockWriteGuard<'_, T>> {
        self.inner.try_write().map(|guard| RwLockWriteGuard { inner: guard })
    }

    pub fn into_inner(self) -> T {
        self.inner.into_inner()
    }
}

pub struct RwLockReadGuard<'a, T> {
    inner: parking_lot::RwLockReadGuard<'a, T>,
}

impl<'a, T> std::ops::Deref for RwLockReadGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

pub struct RwLockWriteGuard<'a, T> {
    inner: parking_lot::RwLockWriteGuard<'a, T>,
}

impl<'a, T> std::ops::Deref for RwLockWriteGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<'a, T> std::ops::DerefMut for RwLockWriteGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

pub struct ReentrantLock<T> {
    inner: Mutex<T>,
    owner: AtomicUsize,
    count: AtomicUsize,
}

impl<T> ReentrantLock<T> {
    pub fn new(value: T) -> Self {
        ReentrantLock {
            inner: Mutex::new(value),
            owner: AtomicUsize::new(0),
            count: AtomicUsize::new(0),
        }
    }

    pub fn lock(&self) -> ReentrantLockGuard<'_, T> {
        let thread_id = thread_id();
        let current_owner = self.owner.load(Ordering::Acquire);

        if current_owner == thread_id {
            self.count.fetch_add(1, Ordering::Relaxed);
        } else {
            let _guard = self.inner.lock().unwrap();
            self.owner.store(thread_id, Ordering::Release);
            self.count.store(1, Ordering::Release);
        }

        ReentrantLockGuard { lock: self }
    }

    pub fn try_lock(&self) -> Option<ReentrantLockGuard<'_, T>> {
        let thread_id = thread_id();
        let current_owner = self.owner.load(Ordering::Acquire);

        if current_owner == thread_id {
            self.count.fetch_add(1, Ordering::Relaxed);
            Some(ReentrantLockGuard { lock: self })
        } else if self.inner.try_lock().is_ok() {
            self.owner.store(thread_id, Ordering::Release);
            self.count.store(1, Ordering::Release);
            Some(ReentrantLockGuard { lock: self })
        } else {
            None
        }
    }
}

fn thread_id() -> usize {
    std::thread::current().id().as_u64().get() as usize
}

pub struct ReentrantLockGuard<'a, T> {
    lock: &'a ReentrantLock<T>,
}

impl<'a, T> Drop for ReentrantLockGuard<'a, T> {
    fn drop(&mut self) {
        let new_count = self.lock.count.fetch_sub(1, Ordering::Relaxed);
        if new_count == 0 {
            self.lock.owner.store(0, Ordering::Release);
            drop(self.lock.inner.lock().unwrap());
        }
    }
}

impl<'a, T> std::ops::Deref for ReentrantLockGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.lock.inner.lock().unwrap() }
    }
}

impl<'a, T> std::ops::DerefMut for ReentrantLockGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.lock.inner.lock().unwrap() }
    }
}

pub struct Event {
    set: AtomicBool,
    mutex: Mutex<()>,
    condvar: Condvar,
}

impl Event {
    pub fn new() -> Self {
        Event {
            set: AtomicBool::new(false),
            mutex: Mutex::new(()),
            condvar: Condvar::new(),
        }
    }

    pub fn wait(&self) {
        let mut guard = self.mutex.lock().unwrap();
        while !self.set.load(Ordering::Acquire) {
            self.condvar.wait(guard).unwrap();
        }
    }

    pub fn wait_timeout(&self, timeout: Duration) -> SyncResult<()> {
        let mut guard = self.mutex.lock().unwrap();
        let deadline = Instant::now() + timeout;

        while !self.set.load(Ordering::Acquire) {
            let remaining = deadline.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                return Err(SyncError::Timeout);
            }

            let result = self.condvar.wait_timeout(guard, remaining).unwrap();
            if result.timed_out() {
                return Err(SyncError::Timeout);
            }
        }
        Ok(())
    }

    pub fn set(&self) {
        self.set.store(true, Ordering::Release);
        let _guard = self.mutex.lock().unwrap();
        self.condvar.notify_all();
    }

    pub fn reset(&self) {
        self.set.store(false, Ordering::Release);
    }

    pub fn is_set(&self) -> bool {
        self.set.load(Ordering::Acquire)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_semaphore() {
        let sem = Semaphore::new(2);
        assert_eq!(sem.available_permits(), 2);
        sem.acquire().unwrap();
        assert_eq!(sem.available_permits(), 1);
        sem.release();
        assert_eq!(sem.available_permits(), 2);
    }

    #[test]
    fn test_barrier() {
        let barrier = Arc::new(Barrier::new(3));
        let mut handles = vec![];

        for _ in 0..3 {
            let b = barrier.clone();
            handles.push(std::thread::spawn(move || {
                b.wait().unwrap();
            }));
        }

        for handle in handles {
            handle.join().unwrap();
        }
    }

    #[test]
    fn test_once_cell() {
        let cell = OnceCell::new();
        assert!(cell.get().is_none());
        cell.set(42).unwrap();
        assert_eq!(*cell.get().unwrap(), 42);
    }

    #[test]
    fn test_once_cell_get_or_init() {
        let cell = OnceCell::new();
        let value = cell.get_or_init(|| 100);
        assert_eq!(*value, 100);
        assert_eq!(*cell.get_or_init(|| 200), 100);
    }

    #[test]
    fn test_once() {
        let once = Once::new();
        let mut counter = 0;
        once.call_once(|| counter += 1);
        once.call_once(|| counter += 1);
        assert_eq!(counter, 1);
    }

    #[test]
    fn test_rwlock() {
        let lock = RwLock::new(42);
        {
            let r1 = lock.read();
            let r2 = lock.read();
            assert_eq!(*r1, 42);
            assert_eq!(*r2, 42);
        }
        {
            let mut w = lock.write();
            *w = 100;
        }
        assert_eq!(*lock.read(), 100);
    }

    #[test]
    fn test_event() {
        let event = Event::new();
        assert!(!event.is_set());
        event.set();
        assert!(event.is_set());
        event.wait();
        event.reset();
        assert!(!event.is_set());
    }
}
