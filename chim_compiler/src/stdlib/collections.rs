// ==================== 集合标准库 - 简化的动态数组 ====================

pub struct Vec {
    data: [int],
    length: int,
}

impl Vec {
    pub fn new() -> Vec {
        Vec { data: [], length: 0 }
    }
    
    pub fn len(&self) -> int {
        self.length
    }
    
    pub fn is_empty(&self) -> bool {
        self.length == 0
    }
    
    pub fn push(&mut self, value: int) {
        self.data[self.length] = value;
        self.length = self.length + 1;
    }
    
    pub fn pop(&mut self) -> int {
        self.length = self.length - 1;
        self.data[self.length]
    }
    
    pub fn get(&self, index: int) -> int {
        self.data[index]
    }
    
    pub fn set(&mut self, index: int, value: int) {
        self.data[index] = value;
    }
    
    pub fn clear(&mut self) {
        self.length = 0;
    }
}

// ==================== 栈 ====================
pub struct Stack {
    data: Vec,
}

impl Stack {
    pub fn new() -> Stack {
        Stack { data: Vec::new() }
    }
    
    pub fn push(&mut self, value: int) {
        self.data.push(value);
    }
    
    pub fn pop(&mut self) -> int {
        self.data.pop()
    }
    
    pub fn peek(&self) -> int {
        self.data.get(self.data.len() - 1)
    }
    
    pub fn len(&self) -> int {
        self.data.len()
    }
    
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

// ==================== 队列 ====================
pub struct Queue {
    data: Vec,
    head: int,
}

impl Queue {
    pub fn new() -> Queue {
        Queue { data: Vec::new(), head: 0 }
    }
    
    pub fn enqueue(&mut self, value: int) {
        self.data.push(value);
    }
    
    pub fn dequeue(&mut self) -> int {
        let value = self.data.get(self.head);
        self.head = self.head + 1;
        if self.head * 2 >= self.data.len() {
            // 压缩
            let mut new_data = Vec::new();
            let i = self.head;
            while i < self.data.len() {
                new_data.push(self.data.get(i));
                i = i + 1;
            }
            self.data = new_data;
            self.head = 0;
        }
        value
    }
    
    pub fn len(&self) -> int {
        self.data.len() - self.head
    }
    
    pub fn is_empty(&self) -> bool {
        self.data.len() == self.head
    }
}

// ==================== 链表节点 ====================
pub struct ListNode {
    value: int,
    next: int,  // 指向下一个节点的索引，-1表示无
}

pub struct LinkedList {
    nodes: [ListNode],
    head: int,
    tail: int,
    size: int,
}

impl LinkedList {
    pub fn new() -> LinkedList {
        LinkedList { nodes: [], head: -1, tail: -1, size: 0 }
    }
    
    pub fn push_front(&mut self, value: int) {
        let new_node = ListNode { value: value, next: self.head };
        self.nodes[self.size] = new_node;
        self.head = self.size;
        if self.tail == -1 {
            self.tail = self.size;
        }
        self.size = self.size + 1;
    }
    
    pub fn push_back(&mut self, value: int) {
        let new_node = ListNode { value: value, next: -1 };
        if self.tail != -1 {
            self.nodes[self.tail].next = self.size;
        }
        self.nodes[self.size] = new_node;
        self.tail = self.size;
        if self.head == -1 {
            self.head = self.size;
        }
        self.size = self.size + 1;
    }
    
    pub fn pop_front(&mut self) -> int {
        let value = self.nodes[self.head].value;
        self.head = self.nodes[self.head].next;
        if self.head == -1 {
            self.tail = -1;
        }
        self.size = self.size - 1;
        value
    }
    
    pub fn len(&self) -> int {
        self.size
    }
    
    pub fn is_empty(&self) -> bool {
        self.size == 0
    }
}
