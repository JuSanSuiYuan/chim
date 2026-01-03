//! 链表实现 - 占位符

/// 链表节点
pub struct Node<T> {
    pub value: T,
    pub next: Option<Box<Node<T>>>,
}

/// 链表
pub struct List<T> {
    head: Option<Box<Node<T>>>,
    len: usize,
}

impl<T> List<T> {
    pub fn new() -> Self {
        Self {
            head: None,
            len: 0,
        }
    }
    
    pub fn push_front(&mut self, value: T) {
        let new_node = Box::new(Node {
            value,
            next: self.head.take(),
        });
        self.head = Some(new_node);
        self.len += 1;
    }
    
    pub fn pop_front(&mut self) -> Option<T> {
        self.head.take().map(|node| {
            self.head = node.next;
            self.len -= 1;
            node.value
        })
    }
    
    pub fn peek_front(&self) -> Option<&T> {
        self.head.as_ref().map(|node| &node.value)
    }
    
    pub fn len(&self) -> usize {
        self.len
    }
    
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
    
    pub fn clear(&mut self) {
        while self.pop_front().is_some() {}
    }
    
    pub fn iter(&self) -> ListIter<'_, T> {
        ListIter {
            current: self.head.as_deref(),
        }
    }
}

pub struct ListIter<'a, T> {
    current: Option<&'a Node<T>>,
}

impl<'a, T> Iterator for ListIter<'a, T> {
    type Item = &'a T;
    
    fn next(&mut self) -> Option<Self::Item> {
        self.current.map(|node| {
            self.current = node.next.as_deref();
            &node.value
        })
    }
}

impl<T> Default for List<T> {
    fn default() -> Self {
        Self::new()
    }
}
