// ==================== Const Fn 支持模块 ====================
// 为 Chim 语言添加 const fn（编译期函数）支持

pub mod const_fn {
    use crate::ctfe::{self, CtfeEvaluator, Expression, Value, Env, Type};
    use crate::stdlib::prelude::{Option, Result, Vec, HashMap, String};

    // ==================== Const Fn 注解 ====================

    #[derive(Debug, Clone)]
    pub struct ConstFn {
        pub name: string,
        pub params: Vec<ConstParam>,
        pub return_type: Type,
        pub body: Expression,
        pub inline: bool,
        pub deprecated: Option<string>,
    }

    #[derive(Debug, Clone)]
    pub struct ConstParam {
        pub name: string,
        pub ty: Type,
        pub default: Option<Value>,
    }

    // ==================== Const 表达式 ====================

    #[derive(Debug, Clone)]
    pub struct ConstExpr {
        pub value: Value,
        pub evaluated_at_compile_time: bool,
        pub ctfe_result: Option<Result<Value, ctfe::CtfeError>>,
    }

    impl ConstExpr {
        pub fn new(value: Value) -> Self {
            ConstExpr {
                value,
                evaluated_at_compile_time: false,
                ctfe_result: None,
            }
        }

        pub fn ctfe(value: Value) -> Self {
            let mut expr = ConstExpr::new(value);
            expr.evaluated_at_compile_time = true;
            expr
        }
    }

    // ==================== Const 求值器 ====================

    pub struct ConstEvaluator {
        ctfe: CtfeEvaluator,
        const_table: HashMap<string, ConstExpr>,
    }

    impl ConstEvaluator {
        pub fn new() -> Self {
            ConstEvaluator {
                ctfe: CtfeEvaluator::new(),
                const_table: HashMap::new(),
            }
        }

        // 求值 const 表达式
        pub fn eval_const_expr(&mut self, expr: &Expression) -> Result<ConstExpr, ctfe::CtfeError> {
            let value = self.ctfe.eval(expr, &mut Env::new())?;
            let mut const_expr = ConstExpr::new(value);
            const_expr.evaluated_at_compile_time = true;
            const_expr.ctfe_result = Some(Ok(const_expr.value.clone()));
            Ok(const_expr)
        }

        // 注册 const 值
        pub fn register_const(&mut self, name: string, value: Value) {
            self.const_table.insert(name, ConstExpr::new(value));
        }

        // 查找 const 值
        pub fn lookup_const(&self, name: &string) -> Option<&ConstExpr> {
            self.const_table.get(name)
        }

        // 检查是否是 const 表达式
        pub fn is_const_expr(&self, expr: &Expression) -> bool {
            self.can_evaluate_at_compile_time(expr)
        }

        fn can_evaluate_at_compile_time(&self, expr: &Expression) -> bool {
            match expr {
                Expression::Literal(_) => true,
                Expression::Identifier(name) => self.const_table.contains_key(name),
                Expression::BinaryOp { left, right, .. } => {
                    self.can_evaluate_at_compile_time(left) && 
                    self.can_evaluate_at_compile_time(right)
                }
                Expression::UnaryOp { expr, .. } => self.can_evaluate_at_compile_time(expr),
                Expression::Call { func, args } => {
                    // 检查是否是 const fn
                    if let Expression::Identifier(name) = func.as_ref() {
                        if name.starts_with("__ctfe_") || self.is_const_fn(name) {
                            args.iter().all(|a| self.can_evaluate_at_compile_time(a))
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                }
                Expression::If { condition, then_branch, else_branch } => {
                    self.can_evaluate_at_compile_time(condition) &&
                    self.can_evaluate_at_compile_time(then_branch) &&
                    else_branch.as_ref().map_or(true, |e| self.can_evaluate_at_compile_time(e))
                }
                Expression::Cast { expr, .. } => self.can_evaluate_at_compile_time(expr),
                Expression::Tuple(elements) | Expression::Array(elements) => {
                    elements.iter().all(|e| self.can_evaluate_at_compile_time(e))
                }
                Expression::Struct { fields, .. } => {
                    fields.iter().all(|(_, e)| self.can_evaluate_at_compile_time(e))
                }
                Expression::SizeOf(_) => true,
                _ => false,
            }
        }

        fn is_const_fn(&self, name: &string) -> bool {
            // 检查是否是已知的 const fn
            const_fns().contains(name)
        }
    }

    // ==================== 内置 Const 函数 ====================

    fn const_fns() -> Vec<&'static str> {
        vec![
            // 数学函数
            "abs",
            "min",
            "max",
            "clamp",
            "sqrt",
            "pow",
            "sin",
            "cos",
            "tan",
            "floor",
            "ceil",
            "round",
            "trunc",
            "fract",
            
            // 字符串函数
            "len",
            "concat",
            "split",
            "trim",
            "uppercase",
            "lowercase",
            "replace",
            
            // 类型转换
            "int",
            "float",
            "string",
            "char",
            "bool",
            
            // 数组函数
            "array_len",
            "first",
            "last",
            
            // 其他
            "size_of",
            "type_of",
            "type_name",
        ]
    }

    // ==================== CTFE 内置函数实现 ====================

    pub struct CtfeBuiltins;

    impl CtfeBuiltins {
        pub fn add_builtins(env: &mut Env) {
            // 数学常量
            env.insert("PI".to_string(), Value::Float(std::f64::consts::PI));
            env.insert("E".to_string(), Value::Float(std::f64::consts::E));
            env.insert("TAU".to_string(), Value::Float(std::f64::consts::TAU));
            
            // 内置函数作为闭包或直接处理
        }

        // 编译期可用的 len 函数
        pub fn len(value: &Value) -> Result<Value, ctfe::CtfeError> {
            match value {
                Value::String(s) => Ok(Value::Int(s.len() as i128)),
                Value::Array(arr) => Ok(Value::Int(arr.len() as i128)),
                Value::Tuple(t) => Ok(Value::Int(t.len() as i128)),
                _ => Err(ctfe::CtfeError::TypeError {
                    expected: "string, array, or tuple".to_string(),
                    found: format!("{:?}", value),
                }),
            }
        }

        pub fn abs(value: &Value) -> Result<Value, ctfe::CtfeError> {
            match value {
                Value::Int(i) => Ok(Value::Int(i.abs())),
                Value::Float(f) => Ok(Value::Float(f.abs())),
                _ => Err(ctfe::CtfeError::TypeError {
                    expected: "numeric".to_string(),
                    found: format!("{:?}", value),
                }),
            }
        }

        pub fn sqrt(value: &Value) -> Result<Value, ctfe::CtfeError> {
            match value {
                Value::Int(i) => Ok(Value::Float((*i as f64).sqrt())),
                Value::Float(f) => Ok(Value::Float(f.sqrt())),
                _ => Err(ctfe::CtfeError::TypeError {
                    expected: "numeric".to_string(),
                    found: format!("{:?}", value),
                }),
            }
        }

        pub fn pow(base: &Value, exp: &Value) -> Result<Value, ctfe::CtfeError> {
            match (base, exp) {
                (Value::Int(i), Value::Int(e)) => {
                    if *e < 0 {
                        Err(ctfe::CtfeError::NegativeExponent)
                    } else {
                        Ok(Value::Int(i.pow(*e as u32)))
                    }
                }
                (Value::Float(f), Value::Int(e)) => {
                    Ok(Value::Float(f.powi(*e as i32)))
                }
                (Value::Float(f), Value::Float(e)) => {
                    Ok(Value::Float(f.powf(*e)))
                }
                _ => Err(ctfe::CtfeError::TypeError {
                    expected: "numeric".to_string(),
                    found: format!("pow({:?}, {:?})", base, exp),
                }),
            }
        }

        pub fn min(a: &Value, b: &Value) -> Result<Value, ctfe::CtfeError> {
            Ok(if a <= b { a.clone() } else { b.clone() })
        }

        pub fn max(a: &Value, b: &Value) -> Result<Value, ctfe::CtfeError> {
            Ok(if a >= b { a.clone() } else { b.clone() })
        }

        pub fn clamp(value: &Value, min: &Value, max: &Value) -> Result<Value, ctfe::CtfeError> {
            if value < min {
                Ok(min.clone())
            } else if value > max {
                Ok(max.clone())
            } else {
                Ok(value.clone())
            }
        }

        pub fn int_value(value: &Value) -> Result<Value, ctfe::CtfeError> {
            match value {
                Value::Int(i) => Ok(Value::Int(*i)),
                Value::Float(f) => Ok(Value::Int(*f as i128)),
                Value::Bool(b) => Ok(Value::Int(if *b { 1 } else { 0 })),
                Value::Char(c) => Ok(Value::Int(*c as i128)),
                Value::String(s) => {
                    if let Ok(n) = s.as_str().parse::<i128>() {
                        Ok(Value::Int(n))
                    } else {
                        Err(ctfe::CtfeError::TypeError {
                            expected: "parsable string".to_string(),
                            found: format!("int(\"{}\")", s),
                        })
                    }
                }
                _ => Err(ctfe::CtfeError::TypeError {
                    expected: "convertible to int".to_string(),
                    found: format!("int({:?})", value),
                }),
            }
        }

        pub fn float_value(value: &Value) -> Result<Value, ctfe::CtfeError> {
            match value {
                Value::Float(f) => Ok(Value::Float(*f)),
                Value::Int(i) => Ok(Value::Float(*i as f64)),
                Value::String(s) => {
                    if let Ok(n) = s.as_str().parse::<f64>() {
                        Ok(Value::Float(n))
                    } else {
                        Err(ctfe::CtfeError::TypeError {
                            expected: "parsable string".to_string(),
                            found: format!("float(\"{}\")", s),
                        })
                    }
                }
                _ => Err(ctfe::CtfeError::TypeError {
                    expected: "convertible to float".to_string(),
                    found: format!("float({:?})", value),
                }),
            }
        }

        pub fn string_value(value: &Value) -> Result<Value, ctfe::CtfeError> {
            Ok(Value::String(value.to_string()))
        }
    }

    // ==================== Const 折叠优化 ====================

    pub struct ConstFolder;

    impl ConstFolder {
        pub fn fold(expr: &Expression) -> Expression {
            match expr {
                Expression::BinaryOp { op, left, right } => {
                    let l = Self::fold(left);
                    let r = Self::fold(right);
                    
                    if let (Some(lv), Some(rv)) = (Self::eval_if_const(&l), Self::eval_if_const(&r)) {
                        // 尝试在编译期计算
                        if let Ok(value) = Self::compute_binary_op(op, &lv, &rv) {
                            return Expression::Literal(Self::value_to_literal(&value));
                        }
                    }
                    
                    Expression::BinaryOp {
                        op: op.clone(),
                        left: Box::new(l),
                        right: Box::new(r),
                    }
                }
                
                Expression::UnaryOp { op, expr } => {
                    let e = Self::fold(expr);
                    if let Some(v) = Self::eval_if_const(&e) {
                        if let Ok(value) = Self::compute_unary_op(op, &v) {
                            return Expression::Literal(Self::value_to_literal(&value));
                        }
                    }
                    
                    Expression::UnaryOp {
                        op: op.clone(),
                        expr: Box::new(e),
                    }
                }
                
                Expression::If { condition, then_branch, else_branch } => {
                    let c = Self::fold(condition);
                    if let Some(Value::Bool(b)) = Self::eval_if_const(&c) {
                        if b {
                            Self::fold(then_branch)
                        } else {
                            match else_branch {
                                Some(e) => Self::fold(e),
                                None => Expression::Literal(Literal::Unit),
                            }
                        }
                    } else {
                        Expression::If {
                            condition: Box::new(c),
                            then_branch: Box::new(Self::fold(then_branch)),
                            else_branch: else_branch.as_ref().map(|e| Box::new(Self::fold(e))),
                        }
                    }
                }
                
                _ => expr.clone(),
            }
        }

        fn eval_if_const(expr: &Expression) -> Option<Value> {
            match expr {
                Expression::Literal(lit) => Some(Self::literal_to_value(lit)),
                Expression::Identifier(name) => {
                    // 假设已经在 const 环境中
                    None
                }
                _ => None,
            }
        }

        fn compute_binary_op(op: &ctfe::BinaryOp, a: &Value, b: &Value) -> Result<Value, String> {
            let mut evaluator = CtfeEvaluator::new();
            evaluator.eval_binary_op(op, a, b).map_err(|e| format!("{}", e))
        }

        fn compute_unary_op(op: &ctfe::UnaryOp, v: &Value) -> Result<Value, String> {
            let mut evaluator = CtfeEvaluator::new();
            evaluator.eval_unary_op(op, v).map_err(|e| format!("{}", e))
        }

        fn literal_to_value(lit: &Literal) -> Value {
            match lit {
                Literal::Unit => Value::Unit,
                Literal::Bool(b) => Value::Bool(*b),
                Literal::Int(i, _) => Value::Int(*i),
                Literal::Uint(u, _) => Value::Uint(*u),
                Literal::Float(f) => Value::Float(*f),
                Literal::Char(c) => Value::Char(*c),
                Literal::String(s) => Value::String(s.clone()),
                Literal::Bytes(b) => Value::Bytes(b.clone()),
            }
        }

        fn value_to_literal(value: &Value) -> Literal {
            match value {
                Value::Unit => Literal::Unit,
                Value::Bool(b) => Literal::Bool(*b),
                Value::Int(i) => Literal::Int(*i, None),
                Value::Uint(u) => Literal::Uint(*u, None),
                Value::Float(f) => Literal::Float(*f),
                Value::Char(c) => Literal::Char(*c),
                Value::String(s) => Literal::String(s.clone()),
                Value::Bytes(b) => Literal::Bytes(b.clone()),
                _ => Literal::Unit,
            }
        }
    }

    use crate::ctfe::Literal;
}

// ==================== 编译器前端集成 ====================

pub mod compiler_integration {
    use crate::ctfe::{self, CtfeEvaluator, Expression, Value, Env};
    use crate::const_fn::{ConstEvaluator, ConstExpr};
    use crate::stdlib::prelude::*;

    // ==================== 编译器修改 ====================

    pub struct CtfePass {
        const_evaluator: ConstEvaluator,
    }

    impl CtfePass {
        pub fn new() -> Self {
            CtfePass {
                const_evaluator: ConstEvaluator::new(),
            }
        }

        // 处理 const fn
        pub fn process_const_fn(&mut self, fn_decl: &Function) -> Result<(), CtfeError> {
            let mut env = Env::new();
            
            // 添加参数到环境
            for param in &fn_decl.params {
                env.insert(param.name.clone(), param.ty.default_value());
            }
            
            // 尝试 CTFE 求值
            if self.const_evaluator.is_const_expr(&fn_decl.body) {
                match self.const_evaluator.eval_const_expr(&fn_decl.body) {
                    Ok(const_expr) => {
                        // 函数可以在编译期求值
                        self.register_ctfe_result(&fn_decl.name, const_expr);
                        Ok(())
                    }
                    Err(e) => Err(CtfeError::from(e)),
                }
            } else {
                Ok(()) // 非 const 函数，不处理
            }
        }

        fn register_ctfe_result(&mut self, name: &str, result: ConstExpr) {
            // 注册 CTFE 结果
        }

        // 处理 const 变量
        pub fn process_const_var(&mut self, var_decl: &VariableDecl) -> Result<Option<Value>, CtfeError> {
            if let Some(initializer) = &var_decl.initializer {
                if self.const_evaluator.is_const_expr(initializer) {
                    let const_expr = self.const_evaluator.eval_const_expr(initializer)?;
                    self.const_evaluator.register_const(var_decl.name.clone(), const_expr.value.clone());
                    Ok(Some(const_expr.value))
                } else {
                    Ok(None)
                }
            } else {
                Ok(None)
            }
        }

        // 折叠 const 表达式
        pub fn fold_const_expr(&self, expr: &Expression) -> Expression {
            crate::const_fn::ConstFolder::fold(expr)
        }
    }

    // ==================== 错误类型 ====================

    #[derive(Debug, Clone)]
    pub enum CtfeError {
        CtfeError(ctfe::CtfeError),
        NotConstExpr(String),
        CtfeNotEnabled,
    }

    impl From<ctfe::CtfeError> for CtfeError {
        fn from(e: ctfe::CtfeError) -> Self {
            CtfeError::CtfeError(e)
        }
    }

    impl std::fmt::Display for CtfeError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                CtfeError::CtfeError(e) => write!(f, "CTFE error: {}", e),
                CtfeError::NotConstExpr(msg) => write!(f, "not a const expression: {}", msg),
                CtfeError::CtfeNotEnabled => write!(f, "CTFE not enabled"),
            }
        }
    }
}

// ==================== 导出 ====================

pub use self::ctfe::{CtfeEvaluator, Expression, Value, Env, Type, CtfeError as Error};
pub use self::const_fn::{ConstEvaluator, ConstExpr, ConstFolder};
pub use self::compiler_integration::CtfePass;
