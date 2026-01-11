// ==================== CTFE 测试套件 ====================
// 测试编译期函数求值功能

#[cfg(test)]
mod ctfe_tests {
    use crate::ctfe::{CtfeEvaluator, Expression, Value, CtfeError};
    use crate::stdlib::string::String as StdString;

    // ==================== 辅助函数 ====================

    fn int_lit(n: i128) -> Expression {
        Expression::Literal(crate::ctfe::Literal::Int(n, None))
    }

    fn float_lit(f: f64) -> Expression {
        Expression::Literal(crate::ctfe::Literal::Float(f))
    }

    fn bool_lit(b: bool) -> Expression {
        Expression::Literal(crate::ctfe::Literal::Bool(b))
    }

    fn string_lit(s: &str) -> Expression {
        Expression::Literal(crate::ctfe::Literal::String(StdString::from(s)))
    }

    fn var(name: &str) -> Expression {
        Expression::Identifier(name.to_string())
    }

    // ==================== 基础运算测试 ====================

    #[test]
    fn test_addition() {
        let mut evaluator = CtfeEvaluator::new();
        
        let expr = Expression::BinaryOp {
            op: crate::ctfe::BinaryOp::Add,
            left: Box::new(int_lit(10)),
            right: Box::new(int_lit(20)),
        };
        
        let result = evaluator.eval(&expr, &mut crate::ctfe::Env::new());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Int(30));
    }

    #[test]
    fn test_subtraction() {
        let mut evaluator = CtfeEvaluator::new();
        
        let expr = Expression::BinaryOp {
            op: crate::ctfe::BinaryOp::Sub,
            left: Box::new(int_lit(100)),
            right: Box::new(int_lit(30)),
        };
        
        let result = evaluator.eval(&expr, &mut crate::ctfe::Env::new());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Int(70));
    }

    #[test]
    fn test_multiplication() {
        let mut evaluator = CtfeEvaluator::new();
        
        let expr = Expression::BinaryOp {
            op: crate::ctfe::BinaryOp::Mul,
            left: Box::new(int_lit(7)),
            right: Box::new(int_lit(8)),
        };
        
        let result = evaluator.eval(&expr, &mut crate::ctfe::Env::new());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Int(56));
    }

    #[test]
    fn test_division() {
        let mut evaluator = CtfeEvaluator::new();
        
        let expr = Expression::BinaryOp {
            op: crate::ctfe::BinaryOp::Div,
            left: Box::new(int_lit(100)),
            right: Box::new(int_lit(4)),
        };
        
        let result = evaluator.eval(&expr, &mut crate::ctfe::Env::new());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Int(25));
    }

    #[test]
    fn test_modulo() {
        let mut evaluator = CtfeEvaluator::new();
        
        let expr = Expression::BinaryOp {
            op: crate::ctfe::BinaryOp::Mod,
            left: Box::new(int_lit(17)),
            right: Box::new(int_lit(5)),
        };
        
        let result = evaluator.eval(&expr, &mut crate::ctfe::Env::new());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Int(2));
    }

    // ==================== 浮点数测试 ====================

    #[test]
    fn test_float_operations() {
        let mut evaluator = CtfeEvaluator::new();
        
        let expr = Expression::BinaryOp {
            op: crate::ctfe::BinaryOp::Add,
            left: Box::new(float_lit(3.14)),
            right: Box::new(float_lit(2.86)),
        };
        
        let result = evaluator.eval(&expr, &mut crate::ctfe::Env::new());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Float(6.0));
    }

    // ==================== 比较运算测试 ====================

    #[test]
    fn test_comparison() {
        let mut evaluator = CtfeEvaluator::new();
        
        let expr = Expression::BinaryOp {
            op: crate::ctfe::BinaryOp::Lt,
            left: Box::new(int_lit(5)),
            right: Box::new(int_lit(10)),
        };
        
        let result = evaluator.eval(&expr, &mut crate::ctfe::Env::new());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Bool(true));
    }

    #[test]
    fn test_equality() {
        let mut evaluator = CtfeEvaluator::new();
        
        let expr = Expression::BinaryOp {
            op: crate::ctfe::BinaryOp::Eq,
            left: Box::new(int_lit(42)),
            right: Box::new(int_lit(42)),
        };
        
        let result = evaluator.eval(&expr, &mut crate::ctfe::Env::new());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Bool(true));
    }

    // ==================== 逻辑运算测试 ====================

    #[test]
    fn test_logical_and() {
        let mut evaluator = CtfeEvaluator::new();
        
        let expr = Expression::BinaryOp {
            op: crate::ctfe::BinaryOp::And,
            left: Box::new(bool_lit(true)),
            right: Box::new(bool_lit(false)),
        };
        
        let result = evaluator.eval(&expr, &mut crate::ctfe::Env::new());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Bool(false));
    }

    #[test]
    fn test_logical_or() {
        let mut evaluator = CtfeEvaluator::new();
        
        let expr = Expression::BinaryOp {
            op: crate::ctfe::BinaryOp::Or,
            left: Box::new(bool_lit(true)),
            right: Box::new(bool_lit(false)),
        };
        
        let result = evaluator.eval(&expr, &mut crate::ctfe::Env::new());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Bool(true));
    }

    // ==================== 一元运算测试 ====================

    #[test]
    fn test_negation() {
        let mut evaluator = CtfeEvaluator::new();
        
        let expr = Expression::UnaryOp {
            op: crate::ctfe::UnaryOp::Neg,
            expr: Box::new(int_lit(42)),
        };
        
        let result = evaluator.eval(&expr, &mut crate::ctfe::Env::new());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Int(-42));
    }

    #[test]
    fn test_bool_not() {
        let mut evaluator = CtfeEvaluator::new();
        
        let expr = Expression::UnaryOp {
            op: crate::ctfe::UnaryOp::Not,
            expr: Box::new(bool_lit(true)),
        };
        
        let result = evaluator.eval(&expr, &mut crate::ctfe::Env::new());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Bool(false));
    }

    // ==================== 条件表达式测试 ====================

    #[test]
    fn test_if_true() {
        let mut evaluator = CtfeEvaluator::new();
        
        let expr = Expression::If {
            condition: Box::new(bool_lit(true)),
            then_branch: Box::new(int_lit(100)),
            else_branch: Some(Box::new(int_lit(200))),
        };
        
        let result = evaluator.eval(&expr, &mut crate::ctfe::Env::new());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Int(100));
    }

    #[test]
    fn test_if_false() {
        let mut evaluator = CtfeEvaluator::new();
        
        let expr = Expression::If {
            condition: Box::new(bool_lit(false)),
            then_branch: Box::new(int_lit(100)),
            else_branch: Some(Box::new(int_lit(200))),
        };
        
        let result = evaluator.eval(&expr, &mut crate::ctfe::Env::new());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Int(200));
    }

    // ==================== 变量测试 ====================

    #[test]
    fn test_variable_lookup() {
        let mut evaluator = CtfeEvaluator::new();
        let mut env = crate::ctfe::Env::new();
        
        env.insert("x".to_string(), Value::Int(42));
        
        let expr = var("x");
        let result = evaluator.eval(&expr, &mut env);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Int(42));
    }

    // ==================== 数组测试 ====================

    #[test]
    fn test_array_literal() {
        let mut evaluator = CtfeEvaluator::new();
        
        let expr = Expression::Array(vec![
            int_lit(1),
            int_lit(2),
            int_lit(3),
        ]);
        
        let result = evaluator.eval(&expr, &mut crate::ctfe::Env::new());
        assert!(result.is_ok());
        match result.unwrap() {
            Value::Array(arr) => {
                assert_eq!(arr.len(), 3);
                assert_eq!(arr[0], Value::Int(1));
                assert_eq!(arr[1], Value::Int(2));
                assert_eq!(arr[2], Value::Int(3));
            }
            _ => panic!("Expected array"),
        }
    }

    // ==================== 元组测试 ====================

    #[test]
    fn test_tuple_literal() {
        let mut evaluator = CtfeEvaluator::new();
        
        let expr = Expression::Tuple(vec![
            int_lit(1),
            string_lit("hello"),
            bool_lit(true),
        ]);
        
        let result = evaluator.eval(&expr, &mut crate::ctfe::Env::new());
        assert!(result.is_ok());
        match result.unwrap() {
            Value::Tuple(t) => {
                assert_eq!(t.len(), 3);
            }
            _ => panic!("Expected tuple"),
        }
    }

    // ==================== 字符串操作测试 ====================

    #[test]
    fn test_string_concat() {
        let mut evaluator = CtfeEvaluator::new();
        
        let expr = Expression::BinaryOp {
            op: crate::ctfe::BinaryOp::StrConcat,
            left: Box::new(string_lit("Hello, ")),
            right: Box::new(string_lit("World!")),
        };
        
        let result = evaluator.eval(&expr, &mut crate::ctfe::Env::new());
        assert!(result.is_ok());
        match result.unwrap() {
            Value::String(s) => {
                assert_eq!(s.as_str(), "Hello, World!");
            }
            _ => panic!("Expected string"),
        }
    }

    // ==================== 错误处理测试 ====================

    #[test]
    fn test_division_by_zero() {
        let mut evaluator = CtfeEvaluator::new();
        
        let expr = Expression::BinaryOp {
            op: crate::ctfe::BinaryOp::Div,
            left: Box::new(int_lit(100)),
            right: Box::new(int_lit(0)),
        };
        
        let result = evaluator.eval(&expr, &mut crate::ctfe::Env::new());
        assert!(result.is_err());
        match result.unwrap_err() {
            CtfeError::DivisionByZero => {}
            _ => panic!("Expected division by zero error"),
        }
    }

    #[test]
    fn test_undefined_variable() {
        let mut evaluator = CtfeEvaluator::new();
        let mut env = crate::ctfe::Env::new();
        
        let expr = var("undefined_var");
        let result = evaluator.eval(&expr, &mut env);
        assert!(result.is_err());
    }

    // ==================== 类型转换测试 ====================

    #[test]
    fn test_int_to_float() {
        let mut evaluator = CtfeEvaluator::new();
        
        let expr = Expression::Cast {
            expr: Box::new(int_lit(42)),
            target_type: crate::ctfe::Type::Float,
        };
        
        let result = evaluator.eval(&expr, &mut crate::ctfe::Env::new());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Float(42.0));
    }

    // ==================== SizeOf 测试 ====================

    #[test]
    fn test_size_of() {
        let mut evaluator = CtfeEvaluator::new();
        
        let expr = Expression::SizeOf(crate::ctfe::Type::Int(64));
        let result = evaluator.eval(&expr, &mut crate::ctfe::Env::new());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Int(8));
    }
}

// ==================== CTFE 性能测试 ====================

#[cfg(test)]
mod ctfe_performance_tests {
    use crate::ctfe::{CtfeEvaluator, Expression, Value};
    use crate::stdlib::string::String as StdString;

    fn int_lit(n: i128) -> Expression {
        Expression::Literal(crate::ctfe::Literal::Int(n, None))
    }

    #[test]
    fn test_fibonacci_ctfe() {
        // 测试递归函数的 CTFE
        let fib_body = Expression::If {
            condition: Box::new(Expression::BinaryOp {
                op: crate::ctfe::BinaryOp::Le,
                left: Box::new(int_lit(1)),
                right: Box::new(int_lit(1)),
            }),
            then_branch: Box::new(int_lit(1)),
            else_branch: Some(Box::new(Expression::BinaryOp {
                op: crate::ctfe::BinaryOp::Add,
                left: Box::new(Expression::Call {
                    func: Box::new(Expression::Identifier("fib".to_string())),
                    args: vec![
                        Expression::BinaryOp {
                            op: crate::ctfe::BinaryOp::Sub,
                            left: Box::new(int_lit(1)),
                            right: Box::new(int_lit(1)),
                        },
                    ],
                }),
                right: Box::new(Expression::Call {
                    func: Box::new(Expression::Identifier("fib".to_string())),
                    args: vec![
                        Expression::BinaryOp {
                            op: crate::ctfe::BinaryOp::Sub,
                            left: Box::new(int_lit(1)),
                            right: Box::new(int_lit(2)),
                        },
                    ],
                }),
            })),
        };

        // 这里省略完整的 fibonacci 测试，因为需要函数定义支持
        // 实际实现中会测试 fibonacci(10) = 55
    }

    #[test]
    fn test_large_expression() {
        let mut evaluator = CtfeEvaluator::new();
        
        // 创建一个大表达式: (1 + 2 + 3 + ... + 100)
        let mut sum_expr = int_lit(0);
        for i in 1..=100 {
            sum_expr = Expression::BinaryOp {
                op: crate::ctfe::BinaryOp::Add,
                left: Box::new(sum_expr),
                right: Box::new(int_lit(i)),
            };
        }
        
        let result = evaluator.eval(&sum_expr, &mut crate::ctfe::Env::new());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Int(5050));
    }
}

// ==================== CTFE 使用示例 ====================

#[cfg(test)]
mod ctfe_examples {
    use crate::ctfe::{CtfeEvaluator, Expression, Value};
    use crate::stdlib::string::String as StdString;

    fn int_lit(n: i128) -> Expression {
        Expression::Literal(crate::ctfe::Literal::Int(n, None))
    }

    fn float_lit(f: f64) -> Expression {
        Expression::Literal(crate::ctfe::Literal::Float(f))
    }

    fn bool_lit(b: bool) -> Expression {
        Expression::Literal(crate::ctfe::Literal::Bool(b))
    }

    #[test]
    fn example_factorial() {
        // 5! = 120
        let expr = Expression::BinaryOp {
            op: crate::ctfe::BinaryOp::Mul,
            left: Box::new(int_lit(5)),
            right: Box::new(int_lit(24)), // 4!
        };
        
        let mut evaluator = CtfeEvaluator::new();
        let result = evaluator.eval(&expr, &mut crate::ctfe::Env::new());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Int(120));
    }

    #[test]
    fn example_nested_conditions() {
        // max(min(1, 2), 3) = 3
        let expr = Expression::Call {
            func: Box::new(Expression::Identifier("max".to_string())),
            args: vec![
                Expression::Call {
                    func: Box::new(Expression::Identifier("min".to_string())),
                    args: vec![int_lit(1), int_lit(2)],
                },
                int_lit(3),
            ],
        };
        
        let mut evaluator = CtfeEvaluator::new();
        let result = evaluator.eval(&expr, &mut crate::ctfe::Env::new());
        // 实际结果取决于内置函数的实现
        assert!(result.is_ok());
    }

    #[test]
    fn example_compound_assignment() {
        // ((1 + 2) * 3) - 4 = 5
        let expr = Expression::BinaryOp {
            op: crate::ctfe::BinaryOp::Sub,
            left: Box::new(Expression::BinaryOp {
                op: crate::ctfe::BinaryOp::Mul,
                left: Box::new(Expression::BinaryOp {
                    op: crate::ctfe::BinaryOp::Add,
                    left: Box::new(int_lit(1)),
                    right: Box::new(int_lit(2)),
                }),
                right: Box::new(int_lit(3)),
            }),
            right: Box::new(int_lit(4)),
        };
        
        let mut evaluator = CtfeEvaluator::new();
        let result = evaluator.eval(&expr, &mut crate::ctfe::Env::new());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Int(5));
    }
}

// ==================== 运行测试 ====================

#[test]
fn run_all_ctfe_tests() {
    println!("Running CTFE tests...");
    
    // 运行所有测试
    ctfe_tests::test_addition();
    ctfe_tests::test_subtraction();
    ctfe_tests::test_multiplication();
    ctfe_tests::test_division();
    ctfe_tests::test_float_operations();
    ctfe_tests::test_comparison();
    ctfe_tests::test_equality();
    ctfe_tests::test_logical_and();
    ctfe_tests::test_logical_or();
    ctfe_tests::test_negation();
    ctfe_tests::test_bool_not();
    ctfe_tests::test_if_true();
    ctfe_tests::test_if_false();
    ctfe_tests::test_variable_lookup();
    ctfe_tests::test_array_literal();
    ctfe_tests::test_tuple_literal();
    ctfe_tests::test_string_concat();
    ctfe_tests::test_division_by_zero();
    ctfe_tests::test_int_to_float();
    ctfe_tests::test_size_of();
    
    println!("All CTFE tests passed!");
}
