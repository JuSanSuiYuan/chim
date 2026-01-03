//! 动态数组实现

use std::alloc::{alloc, dealloc, realloc, Layout};
use std::ptr;

/// 动态数组
pub struct Vec<T> {
    ptr: *mut T,
    len: usize,
    capacity: usize,
}

impl<T> Vec<T> {
    /// 创建新的空数组
    pub fn new() -> Self {
        Self {
            ptr: ptr::null_mut(),
            len: 0,
            capacity: 0,
        }
    }
    
    /// 创建指定容量的数组
    pub fn with_capacity(capacity: usize) -> Self {
        if capacity == 0 {
            return Self::new();
        }
        
        let layout = Layout::array::<T>(capacity).unwrap();
        let ptr = unsafe { alloc(layout) as *mut T };
        
        Self {
            ptr,
            len: 0,
            capacity,
        }
    }
    
    /// 获取长度
    pub fn len(&self) -> usize {
        self.len
    }
    
    /// 检查是否为空
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
    
    /// 获取容量
    pub fn capacity(&self) -> usize {
        self.capacity
    }
    
    /// 添加元素
    pub fn push(&mut self, value: T) {
        if self.len == self.capacity {
            self.grow();
        }
        
        unsafe {
            ptr::write(self.ptr.add(self.len), value);
        }
        self.len += 1;
    }
    
    /// 弹出元素
    pub fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            return None;
        }
        
        self.len -= 1;
        unsafe {
            Some(ptr::read(self.ptr.add(self.len)))
        }
    }
    
    /// 获取元素引用
    pub fn get(&self, index: usize) -> Option<&T> {
        if index < self.len {
            unsafe {
                Some(&*self.ptr.add(index))
            }
        } else {
            None
        }
    }
    
    /// 获取可变元素引用
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        if index < self.len {
            unsafe {
                Some(&mut *self.ptr.add(index))
            }
        } else {
            None
        }
    }
    
    /// 扩容
    fn grow(&mut self) {
        let new_capacity = if self.capacity == 0 {
            4
        } else {
            self.capacity * 2
        };
        
        let new_layout = Layout::array::<T>(new_capacity).unwrap();
        
        let new_ptr = if self.capacity == 0 {
            unsafe { alloc(new_layout) as *mut T }
        } else {
            let old_layout = Layout::array::<T>(self.capacity).unwrap();
            unsafe {
                realloc(self.ptr as *mut u8, old_layout, new_layout.size()) as *mut T
            }
        };
        
        self.ptr = new_ptr;
        self.capacity = new_capacity;
    }
}

impl<T> Drop for Vec<T> {
    fn drop(&mut self) {
        if self.capacity > 0 {
            unsafe {
                // 手动drop所有元素
                for i in 0..self.len {
                    ptr::drop_in_place(self.ptr.add(i));
                }
                
                // 释放内存
                let layout = Layout::array::<T>(self.capacity).unwrap();
                dealloc(self.ptr as *mut u8, layout);
            }
        }
    }
}

impl<T> Default for Vec<T> {
    fn default() -> Self {
        Self::new()
    }
}
