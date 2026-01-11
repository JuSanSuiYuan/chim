use std::alloc::{GlobalAlloc, Layout, alloc, dealloc};
use std::sync::atomic::{AtomicUsize, Ordering};

pub static ALLOCATED_BYTES: AtomicUsize = AtomicUsize::new(0);
pub static FREED_BYTES: AtomicUsize = AtomicUsize::new(0);
pub static ALLOCATION_COUNT: AtomicUsize = AtomicUsize::new(0);
pub static FREE_COUNT: AtomicUsize = AtomicUsize::new(0);

#[global_allocator]
static GLOBAL: Allocator = Allocator;

struct Allocator;

unsafe impl GlobalAlloc for Allocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let size = layout.size();
        let align = layout.align();

        let ptr = if align <= 16 {
            alloc(layout)
        } else {
            let layout = Layout::from_size_align(size, 16).unwrap();
            alloc(layout)
        };

        if !ptr.is_null() {
            ALLOCATED_BYTES.fetch_add(size, Ordering::Relaxed);
            ALLOCATION_COUNT.fetch_add(1, Ordering::Relaxed);
        }

        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let size = layout.size();
        dealloc(ptr, layout);
        FREED_BYTES.fetch_add(size, Ordering::Relaxed);
        FREE_COUNT.fetch_add(1, Ordering::Relaxed);
    }
}

pub fn allocated_bytes() -> usize {
    ALLOCATED_BYTES.load(Ordering::Relaxed)
}

pub fn freed_bytes() -> usize {
    FREED_BYTES.load(Ordering::Relaxed)
}

pub fn current_allocated() -> isize {
    allocated_bytes() as isize - freed_bytes() as isize
}

pub fn allocation_count() -> usize {
    ALLOCATION_COUNT.load(Ordering::Relaxed)
}

pub fn free_count() -> usize {
    FREE_COUNT.load(Ordering::Relaxed)
}

pub fn reset_stats() {
    ALLOCATED_BYTES.store(0, Ordering::Relaxed);
    FREED_BYTES.store(0, Ordering::Relaxed);
    ALLOCATION_COUNT.store(0, Ordering::Relaxed);
    FREE_COUNT.store(0, Ordering::Relaxed);
}

pub mod memory {
    use super::*;

    pub struct MemoryBlock {
        ptr: *mut u8,
        size: usize,
    }

    impl MemoryBlock {
        pub fn new(size: usize) -> Self {
            let layout = Layout::from_size_align(size, 16).unwrap();
            let ptr = unsafe { alloc(layout) };

            if ptr.is_null() {
                panic!("memory allocation failed");
            }

            MemoryBlock { ptr, size }
        }

        pub fn as_ptr(&self) -> *mut u8 {
            self.ptr
        }

        pub fn as_slice(&mut self) -> &mut [u8] {
            unsafe { std::slice::from_raw_parts_mut(self.ptr, self.size) }
        }

        pub fn size(&self) -> usize {
            self.size
        }

        pub fn clear(&mut self) {
            unsafe {
                std::ptr::write_bytes(self.ptr, 0, self.size);
            }
        }
    }

    impl Drop for MemoryBlock {
        fn drop(&mut self) {
            let layout = Layout::from_size_align(self.size, 16).unwrap();
            unsafe {
                dealloc(self.ptr, layout);
            }
        }
    }

    pub struct MemoryPool {
        block_size: usize,
        blocks: Vec<MemoryBlock>,
        free_list: Vec<*mut u8>,
    }

    impl MemoryPool {
        pub fn new(block_size: usize, initial_blocks: usize) -> Self {
            let mut pool = MemoryPool {
                block_size,
                blocks: Vec::with_capacity(initial_blocks),
                free_list: Vec::with_capacity(initial_blocks),
            };

            for _ in 0..initial_blocks {
                let block = MemoryBlock::new(block_size);
                pool.free_list.push(block.ptr);
                pool.blocks.push(block);
            }

            pool
        }

        pub fn allocate(&mut self) -> *mut u8 {
            if let Some(ptr) = self.free_list.pop() {
                ptr
            } else {
                let ptr = {
                    let block = MemoryBlock::new(self.block_size);
                    let block_ptr = block.ptr;
                    self.blocks.push(block);
                    block_ptr
                };
                ptr
            }
        }

        pub fn deallocate(&mut self, ptr: *mut u8) {
            self.free_list.push(ptr);
        }

        pub fn block_size(&self) -> usize {
            self.block_size
        }

        pub fn available(&self) -> usize {
            self.free_list.len() * self.block_size
        }
    }

    pub struct Arena {
        pools: Vec<MemoryPool>,
        current_pool: usize,
    }

    impl Arena {
        pub fn new() -> Self {
            Arena {
                pools: Vec::new(),
                current_pool: 0,
            }
        }

        pub fn add_pool(&mut self, block_size: usize, initial_blocks: usize) {
            self.pools.push(MemoryPool::new(block_size, initial_blocks));
        }

        pub fn allocate(&mut self, size: usize) -> *mut u8 {
            if self.pools.is_empty() {
                self.add_pool(1024, 4);
            }

            for (i, pool) in self.pools.iter().enumerate() {
                if size <= pool.block_size() {
                    return self.pools[i].allocate();
                }
            }

            let mut new_pool = MemoryPool::new(size.next_power_of_two(), 1);
            let ptr = new_pool.allocate();
            self.pools.push(new_pool);
            ptr
        }

        pub fn deallocate(&mut self, ptr: *mut u8, size: usize) {
            for pool in &mut self.pools {
                if size <= pool.block_size() {
                    pool.deallocate(ptr);
                    return;
                }
            }
        }

        pub fn reset(&mut self) {
            for pool in &mut self.pools {
                for block in &mut pool.blocks {
                    block.clear();
                }
                pool.free_list.clear();
                for block in &pool.blocks {
                    pool.free_list.push(block.ptr);
                }
            }
        }
        pub fn total_size(&self) -> usize {
            self.pools.iter().map(|p| p.blocks.len() * p.block_size()).sum()
        }
    }

    impl Default for Arena {
        fn default() -> Self {
            Arena::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_block() {
        let mut block = MemoryBlock::new(100);
        assert!(!block.ptr.is_null());
        assert_eq!(block.size(), 100);
    }

    #[test]
    fn test_memory_pool() {
        let mut pool = MemoryPool::new(64, 4);
        let ptr = pool.allocate();
        assert!(!ptr.is_null());
        pool.deallocate(ptr);
        assert_eq!(pool.available(), 64);
    }

    #[test]
    fn test_arena() {
        let mut arena = Arena::new();
        let ptr = arena.allocate(100);
        assert!(!ptr.is_null());
    }
}
