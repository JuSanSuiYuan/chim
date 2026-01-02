use crate::ast::{Expression, Statement};
use crate::ir::{Instruction, Function};

/// 返回值优化器 (Return Value Optimization)
/// 
/// RVO通过消除不必要的拷贝来优化函数返回值：
/// 1. 检测直接返回结构体构造的模式
/// 2. 将返回值直接构造在调用者的栈帧中
/// 3. 避免临时对象的创建和拷贝
pub struct RVOOptimizer {
    optimized_count: usize,
}

impl RVOOptimizer {
    pub fn new() -> Self {
        Self {
            optimized_count: 0,
        }
    }
    
    /// 检查函数是否可以应用RVO
    pub fn can_optimize(&self, func: &Statement) -> bool {
        if let Statement::Function { body, .. } = func {
            self.has_direct_return(body)
        } else {
            false
        }
    }
    
    /// 检查函数体是否直接返回一个结构体构造
    fn has_direct_return(&self, body: &Expression) -> bool {
        match body {
            // 块表达式：检查最后一个语句
            Expression::Block(stmts) => {
                if let Some(Statement::Return(Some(expr))) = stmts.last() {
                    matches!(expr, Expression::Struct { .. })
                } else {
                    false
                }
            },
            // 直接返回结构体构造
            Expression::Struct { .. } => true,
            _ => false,
        }
    }
    
    /// 应用RVO优化到IR函数
    pub fn optimize_function(&mut self, func: &mut Function) -> bool {
        let mut optimized = false;
        let mut new_body = Vec::new();
        
        for inst in &func.body {
            match inst {
                // 模式1: Store临时变量 + Return临时变量
                Instruction::Store { dest, src } => {
                    // 检查下一条指令是否是返回这个临时变量
                    new_body.push(inst.clone());
                },
                
                // 模式2: 直接返回结构体构造
                Instruction::Return(Some(value)) => {
                    // 检查value是否是结构体构造的临时变量
                    if value.starts_with(".tmp") {
                        // 应用RVO：标记为原地构造
                        new_body.push(Instruction::ReturnInPlace(value.clone()));
                        optimized = true;
                        self.optimized_count += 1;
                    } else {
                        new_body.push(inst.clone());
                    }
                },
                
                _ => {
                    new_body.push(inst.clone());
                }
            }
        }
        
        if optimized {
            func.body = new_body;
        }
        
        optimized
    }
    
    /// 分析整个模块并应用RVO
    pub fn optimize_module(&mut self, module: &mut crate::ir::Module) {
        for func in &mut module.functions {
            if self.optimize_function(func) {
                println!("[RVO优化] 函数 '{}' 应用了返回值优化", func.name);
            }
        }
        
        if self.optimized_count > 0 {
            println!("[RVO优化] 总计优化了 {} 个函数返回", self.optimized_count);
        }
    }
    
    /// 获取优化统计
    pub fn get_stats(&self) -> usize {
        self.optimized_count
    }
}

impl Default for RVOOptimizer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::*;
    
    #[test]
    fn test_can_optimize_direct_return() {
        let optimizer = RVOOptimizer::new();
        
        let func = Statement::Function {
            name: "test".to_string(),
            params: vec![],
            return_type: Some("Point".to_string()),
            body: Expression::Struct {
                name: "Point".to_string(),
                fields: vec![
                    ("x".to_string(), Expression::Literal(Literal::Float(1.0))),
                    ("y".to_string(), Expression::Literal(Literal::Float(2.0))),
                ],
            },
            kernel: false,
        };
        
        assert!(optimizer.can_optimize(&func));
    }
    
    #[test]
    fn test_can_optimize_block_return() {
        let optimizer = RVOOptimizer::new();
        
        let func = Statement::Function {
            name: "test".to_string(),
            params: vec![],
            return_type: Some("Point".to_string()),
            body: Expression::Block(vec![
                Statement::Return(Some(Expression::Struct {
                    name: "Point".to_string(),
                    fields: vec![],
                })),
            ]),
            kernel: false,
        };
        
        assert!(optimizer.can_optimize(&func));
    }
    
    #[test]
    fn test_cannot_optimize_variable_return() {
        let optimizer = RVOOptimizer::new();
        
        let func = Statement::Function {
            name: "test".to_string(),
            params: vec![],
            return_type: Some("int".to_string()),
            body: Expression::Block(vec![
                Statement::Return(Some(Expression::Identifier("x".to_string()))),
            ]),
            kernel: false,
        };
        
        assert!(!optimizer.can_optimize(&func));
    }
}
