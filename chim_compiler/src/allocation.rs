use crate::ast::{Expression, UnaryOperator};
use crate::semantic::EscapeAnalyzer;
use crate::memory_layout::MemoryLayoutAnalyzer;
// use crate::radix_pool::RadixMemoryPool;  // 暂时注释掉，未使用

/// 分配策略（增强版：支持基数树内存池）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AllocationStrategy {
    Stack,
    Heap,
    RadixPool,  // 新增：使用分层基数树内存池
}

/// 栈/堆/池分配决策器（增强版）
/// 
/// 根据以下规则决定变量应该分配在栈、堆还是内存池：
/// 1. 生命周期 < 100指令 -> 栈分配（超激进优化）
/// 2. 100-1000指令 && size <= 4KB -> 基数树内存池（新增！）
/// 3. 逃逸或 size > 4KB -> 堆分配
/// 4. 默认优先栈分配（性能最好）
pub struct AllocationDecider {
    escape_analyzer: EscapeAnalyzer,
    memory_layout: MemoryLayoutAnalyzer,
    /// 栈分配的大小阈值（字节）
    stack_threshold: usize,
    /// 内存池分配的大小阈值（字节）
    pool_threshold: usize,
    /// 内存池分配的生命周期阈值（指令数）
    pool_lifetime_threshold: usize,
}

impl AllocationDecider {
    pub fn new(escape_analyzer: EscapeAnalyzer, memory_layout: MemoryLayoutAnalyzer) -> Self {
        Self {
            escape_analyzer,
            memory_layout,
            stack_threshold: 4096,  // 提高到4KB（Rust仅1KB）
            pool_threshold: 4096,   // 内存池支持到4KB
            pool_lifetime_threshold: 1000,  // 中等生命周期
        }
    }
    
    /// 设置栈分配阈值
    pub fn set_stack_threshold(&mut self, threshold: usize) {
        self.stack_threshold = threshold;
    }
    
    /// 设置内存池阈值
    pub fn set_pool_threshold(&mut self, threshold: usize) {
        self.pool_threshold = threshold;
    }
    
    /// 决定变量应该分配在栈、堆还是内存池（增强版）
    pub fn decide_with_pool(
        &self, 
        name: &str, 
        ty: &str, 
        initializer: &Expression, 
        context: &str,
        lifetime_instructions: usize,  // 新增：生命周期长度（指令数）
    ) -> AllocationStrategy {
        let size = self.get_type_size(ty);
        
        // 规则1：超短生命周期（<100指令）-> 栈分配
        if lifetime_instructions < 100 {
            return AllocationStrategy::Stack;
        }
        
        // 规则2：中等生命周期（100-1000指令）且大小适中 -> 内存池
        if lifetime_instructions >= 100 
            && lifetime_instructions < self.pool_lifetime_threshold 
            && size <= self.pool_threshold 
        {
            // 不逃逸才能用内存池
            if !self.escape_analyzer.should_allocate_on_heap(name, context) {
                return AllocationStrategy::RadixPool;
            }
        }
        
        // 规则3：如果变量逃逸，分配在堆
        if self.escape_analyzer.should_allocate_on_heap(name, context) {
            return AllocationStrategy::Heap;
        }
        
        // 规则4：大型结构体（超过阈值）分配在堆
        if self.is_large_type(ty) {
            return AllocationStrategy::Heap;
        }
        
        // 规则5：取地址操作需要堆分配
        if self.has_address_taken(initializer) {
            return AllocationStrategy::Heap;
        }
        
        // 规则6：递归类型必须堆分配
        if self.is_recursive_type(ty) {
            return AllocationStrategy::Heap;
        }
        
        // 默认栈分配
        AllocationStrategy::Stack
    }
    
    /// 检查类型是否过大
    fn is_large_type(&self, ty: &str) -> bool {
        // 检查基本类型
        match ty {
            "int" | "float" | "bool" => false,
            "string" => false, // string是指针+长度，不算大
            _ => {
                // 检查结构体大小
                if let Some(layout) = self.memory_layout.get_layout(ty) {
                    layout.size > self.stack_threshold
                } else {
                    // 未知类型，保守处理
                    false
                }
            }
        }
    }
    
    /// 检查是否有取地址操作
    fn has_address_taken(&self, expr: &Expression) -> bool {
        match expr {
            Expression::UnaryOp { op: UnaryOperator::Ref, .. } => true,
            Expression::Block(stmts) => {
                // 检查块中的语句
                stmts.iter().any(|stmt| {
                    if let crate::ast::Statement::Expression(e) = stmt {
                        self.has_address_taken(e)
                    } else {
                        false
                    }
                })
            },
            Expression::BinaryOp { left, right, .. } => {
                self.has_address_taken(left) || self.has_address_taken(right)
            },
            _ => false,
        }
    }
    
    /// 检查是否为递归类型
    fn is_recursive_type(&self, _ty: &str) -> bool {
        // 简化实现：未来可以添加递归检测
        // 例如：struct Node { next: Node } 必须用 next: Box<Node>
        false
    }
    
    /// 获取类型的实际大小
    pub fn get_type_size(&self, ty: &str) -> usize {
        if let Some(layout) = self.memory_layout.get_layout(ty) {
            layout.size
        } else {
            // 基本类型大小
            match ty {
                "int" | "float" => 4,
                "bool" => 1,
                "string" => 16,
                _ => 8, // 默认指针大小
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{Expression, Literal};
    
    #[test]
    fn test_stack_allocation() {
        let escape_analyzer = EscapeAnalyzer::new();
        let memory_layout = MemoryLayoutAnalyzer::new();
        let decider = AllocationDecider::new(escape_analyzer, memory_layout);
        
        // 简单类型应该栈分配
        let expr = Expression::Literal(Literal::Integer(42));
        let strategy = decider.decide("x", "int", &expr, "test");
        
        assert_eq!(strategy, AllocationStrategy::Stack);
    }
    
    #[test]
    fn test_heap_allocation_on_address_taken() {
        let escape_analyzer = EscapeAnalyzer::new();
        let memory_layout = MemoryLayoutAnalyzer::new();
        let decider = AllocationDecider::new(escape_analyzer, memory_layout);
        
        // 取地址操作应该堆分配
        let expr = Expression::UnaryOp {
            op: UnaryOperator::Ref,
            expr: Box::new(Expression::Identifier("x".to_string())),
        };
        
        let strategy = decider.decide("x", "int", &expr, "test");
        assert_eq!(strategy, AllocationStrategy::Heap);
    }
    
    #[test]
    fn test_stack_threshold() {
        let escape_analyzer = EscapeAnalyzer::new();
        let mut memory_layout = MemoryLayoutAnalyzer::new();
        let mut decider = AllocationDecider::new(escape_analyzer, memory_layout);
        
        // 设置小阈值
        decider.set_stack_threshold(10);
        
        // 即使是小类型，如果超过阈值也应该堆分配
        // (实际测试需要有结构体定义)
    }
    
    #[test]
    fn test_get_type_size() {
        let escape_analyzer = EscapeAnalyzer::new();
        let memory_layout = MemoryLayoutAnalyzer::new();
        let decider = AllocationDecider::new(escape_analyzer, memory_layout);
        
        assert_eq!(decider.get_type_size("int"), 4);
        assert_eq!(decider.get_type_size("float"), 4);
        assert_eq!(decider.get_type_size("bool"), 1);
        assert_eq!(decider.get_type_size("string"), 16);
    }
}
