// ==================== CTFE 核心模块 ====================
// Compile-Time Function Evaluation (编译期函数求值)
// 参考 D 语言 CTFE 设计，支持在编译期执行普通函数

pub mod ctfe {
    use crate::stdlib::prelude::{Option, Result, Vec, HashMap, String, StringBuilder};
    use crate::stdlib::string::String as StdString;
    use crate::stdlib::conv::ToString;

    // ==================== CTFE 值类型 ====================

    #[derive(Debug, Clone, PartialEq)]
    pub enum Value {
        Unit,
        Bool(bool),
        Int(i128),
        Uint(u128),
        Float(f64),
        Char(char),
        String(StdString),
        Bytes(Vec<u8>),
        
        // 聚合类型
        Array(Vec<Value>),
        Tuple(Vec<Value>),
        Struct(StructValue),
        Enum(EnumValue),
        
        // 函数
        Function(FnValue),
        Closure(ClosureValue),
        
        // 指针
        Pointer(PointerValue),
        
        // 错误标记
        Error(ErrorValue),
        
        // 未初始化
        Uninitialized,
    }

    #[derive(Debug, Clone, PartialEq)]
    pub struct StructValue {
        pub name: string,
        pub fields: HashMap<string, Value>,
    }

    #[derive(Debug, Clone, PartialEq)]
    pub struct EnumValue {
        pub name: string,
        pub variant: string,
        pub value: Option<Value>,
    }

    #[derive(Debug, Clone, PartialEq)]
    pub struct FnValue {
        pub name: string,
        pub params: Vec<ParamInfo>,
        pub body: Expression,
        pub env: Env,
    }

    #[derive(Debug, Clone, PartialEq)]
    pub struct ParamInfo {
        pub name: string,
        pub ty: Type,
        pub default: Option<Value>,
    }

    #[derive(Debug, Clone, PartialEq)]
    pub struct ClosureValue {
        pub fn_value: FnValue,
        pub captures: HashMap<string, Value>,
    }

    #[derive(Debug, Clone, PartialEq)]
    pub struct PointerValue {
        pub addr: usize,
        pub ty: Type,
        pub data: Option<Box<Value>>,
    }

    #[derive(Debug, Clone, PartialEq)]
    pub struct ErrorValue {
        pub message: string,
        pub location: Option<Location>,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct Location {
        pub file: string,
        pub line: int,
        pub column: int,
    }

    // ==================== CTFE 类型系统 ====================

    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub enum Type {
        Unit,
        Bool,
        Int(usize),
        Uint(usize),
        Float,
        Char,
        String,
        Bytes,
        Array(Box<Type>, Option<usize>),
        Tuple(Vec<Type>),
        Struct(string),
        Enum(string, string),
        Function(Vec<Type>, Box<Type>),
        Pointer(Box<Type>),
        Never,
    }

    impl Type {
        pub fn is_ctfe_compatible(&self) -> bool {
            match self {
                Type::Pointer(_) => false,  // 指针不能在 CTFE 中创建
                Type::Function(_, _) => false,
                _ => true,
            }
        }

        pub fn default_value(&self) -> Value {
            match self {
                Type::Unit => Value::Unit,
                Type::Bool => Value::Bool(false),
                Type::Int(_) => Value::Int(0),
                Type::Uint(_) => Value::Uint(0),
                Type::Float => Value::Float(0.0),
                Type::Char => Value::Char('\0'),
                Type::String => Value::String(StdString::new()),
                Type::Bytes => Value::Bytes(Vec::new()),
                Type::Array(t, _) => Value::Array(vec![t.default_value()]),
                Type::Tuple(types) => Value::Tuple(types.iter().map(|t| t.default_value()).collect()),
                _ => Value::Uninitialized,
            }
        }
    }

    // ==================== CTFE 表达式 ====================

    #[derive(Debug, Clone, PartialEq)]
    pub enum Expression {
        // 字面量
        Literal(Literal),
        
        // 标识符
        Identifier(string),
        
        // 二元运算
        BinaryOp {
            op: BinaryOp,
            left: Box<Expression>,
            right: Box<Expression>,
        },
        
        // 一元运算
        UnaryOp {
            op: UnaryOp,
            expr: Box<Expression>,
        },
        
        // 函数调用
        Call {
            func: Box<Expression>,
            args: Vec<Expression>,
        },
        
        // 条件
        If {
            condition: Box<Expression>,
            then_branch: Box<Expression>,
            else_branch: Option<Box<Expression>>,
        },
        
        // 循环
        Loop {
            body: Box<Expression>,
            label: Option<string>,
        },
        
        // Break
        Break {
            value: Option<Box<Expression>>,
            label: Option<string>,
        },
        
        // Continue
        Continue(Option<string>),
        
        // Return
        Return(Option<Box<Expression>>),
        
        // Block
        Block(Vec<Statement>, Option<Box<Expression>>),
        
        // 变量声明
        VarDecl {
            name: string,
            ty: Option<Type>,
            initializer: Option<Box<Expression>>,
            mutable: bool,
        },
        
        // 赋值
        Assign {
            target: Box<Expression>,
            value: Box<Expression>,
        },
        
        // 成员访问
        Member {
            expr: Box<Expression>,
            member: string,
        },
        
        // 索引访问
        Index {
            expr: Box<Expression>,
            index: Box<Expression>,
        },
        
        // 类型转换
        Cast {
            expr: Box<Expression>,
            target_type: Type,
        },
        
        // Tuple 构造
        Tuple(Vec<Expression>),
        
        // Array 构造
        Array(Vec<Expression>),
        
        // Struct 构造
        Struct {
            name: string,
            fields: Vec<(string, Expression)>,
        },
        
        // Enum 构造
        Enum {
            enum_name: string,
            variant: string,
            value: Option<Box<Expression>>,
        },
        
        // Lambda
        Lambda {
            params: Vec<ParamInfo>,
            body: Box<Expression>,
        },
        
        // SizeOf
        SizeOf(Type),
        
        // TypeOf
        TypeOf(Box<Expression>),
        
        // 错误传播
        Unreachable,
    }

    #[derive(Debug, Clone, PartialEq)]
    pub enum Literal {
        Unit,
        Bool(bool),
        Int(i128, Option<Type>),
        Uint(u128, Option<Type>),
        Float(f64),
        Char(char),
        String(StdString),
        Bytes(Vec<u8>),
    }

    #[derive(Debug, Clone, PartialEq)]
    pub enum BinaryOp {
        // 算术
        Add, Sub, Mul, Div, Mod,
        Pow,   // 指数运算
        
        // 位运算
        BitAnd, BitOr, BitXor,
        Shl, Shr,
        
        // 比较
        Eq, Ne, Lt, Le, Gt, Ge,
        
        // 逻辑
        And, Or,
        
        // 字符串
        StrConcat,
    }

    #[derive(Debug, Clone, PartialEq)]
    pub enum UnaryOp {
        Neg,     // -x
        Not,     // !x / ~x
        BitNot,  // ~x
    }

    #[derive(Debug, Clone, PartialEq)]
    pub enum Statement {
        Expression(Expression),
        Let {
            pattern: Pattern,
            ty: Option<Type>,
            initializer: Option<Expression>,
            mutable: bool,
        },
        While {
            condition: Expression,
            body: Expression,
            label: Option<string>,
        },
        For {
            init: Option<Box<Statement>>,
            condition: Option<Expression>,
            update: Option<Expression>,
            body: Expression,
            label: Option<string>,
        },
        If {
            condition: Expression,
            then_branch: Expression,
            else_branch: Option<Expression>,
        },
        Break(Option<string>),
        Continue(Option<string>),
        Return(Option<Expression>),
        Block(Expression),
        Empty,
    }

    #[derive(Debug, Clone, PartialEq)]
    pub enum Pattern {
        Identifier(string),
        Wildcard,
        Tuple(Vec<Pattern>),
        Struct {
            name: string,
            fields: Vec<(string, Pattern)>,
        },
        Enum {
            enum_name: string,
            variant: string,
            pattern: Option<Box<Pattern>>,
        },
        Literal(Literal),
        Or(Vec<Pattern>),
    }

    // ==================== CTFE 环境 ====================

    #[derive(Debug, Clone)]
    pub struct Env {
        parent: Option<Box<Env>>,
        bindings: HashMap<string, Value>,
        functions: HashMap<string, FnValue>,
        types: HashMap<string, Type>,
        labels: HashMap<string, usize>,  // 循环深度
    }

    impl Env {
        pub fn new() -> Env {
            Env {
                parent: None,
                bindings: HashMap::new(),
                functions: HashMap::new(),
                types: HashMap::new(),
                labels: HashMap::new(),
            }
        }

        pub fn with_parent(parent: Env) -> Env {
            Env {
                parent: Some(Box::new(parent)),
                bindings: HashMap::new(),
                functions: HashMap::new(),
                types: HashMap::new(),
                labels: HashMap::new(),
            }
        }

        pub fn lookup(&self, name: &string) -> Option<Value> {
            if let Some(v) = self.bindings.get(name) {
                return Some(v.clone());
            }
            if let Some(ref parent) = self.parent {
                return parent.lookup(name);
            }
            None
        }

        pub fn insert(&mut self, name: string, value: Value) {
            self.bindings.insert(name, value);
        }

        pub fn insert_fn(&mut self, name: string, fn_value: FnValue) {
            self.functions.insert(name, fn_value);
        }

        pub fn lookup_fn(&self, name: &string) -> Option<FnValue> {
            if let Some(f) = self.functions.get(name) {
                return Some(f.clone());
            }
            if let Some(ref parent) = self.parent {
                return parent.lookup_fn(name);
            }
            None
        }

        pub fn enter_scope(&mut self) {
            let new_env = Env::with_parent(self.clone());
            *self = new_env;
        }

        pub fn exit_scope(&mut self) {
            if let Some(parent) = self.parent.take() {
                *self = *parent;
            }
        }
    }

    // ==================== CTFE 求值器 ====================

    pub struct CtfeEvaluator {
        max_steps: usize,
        current_step: usize,
        errors: Vec<CtfeError>,
    }

    impl CtfeEvaluator {
        pub fn new() -> Self {
            CtfeEvaluator {
                max_steps: 1_000_000,  // 默认最大步数
                current_step: 0,
                errors: Vec::new(),
            }
        }

        pub fn with_max_steps(mut self, steps: usize) -> Self {
            self.max_steps = steps;
            self
        }

        pub fn eval(&mut self, expr: &Expression, env: &mut Env) -> Result<Value, CtfeError> {
            self.current_step = 0;
            self.eval_internal(expr, env)
        }

        fn eval_internal(&mut self, expr: &Expression, env: &mut Env) -> Result<Value, CtfeError> {
            self.current_step += 1;
            if self.current_step > self.max_steps {
                return Err(CtfeError::TooManySteps {
                    limit: self.max_steps,
                });
            }

            match expr {
                Expression::Literal(lit) => self.eval_literal(lit),
                
                Expression::Identifier(name) => {
                    if let Some(value) = env.lookup(name) {
                        Ok(value)
                    } else {
                        Err(CtfeError::UndefinedVariable {
                            name: name.clone(),
                        })
                    }
                }
                
                Expression::BinaryOp { op, left, right } => {
                    let l = self.eval_internal(left, env)?;
                    let r = self.eval_internal(right, env)?;
                    self.eval_binary_op(op, &l, &r)
                }
                
                Expression::UnaryOp { op, expr } => {
                    let v = self.eval_internal(expr, env)?;
                    self.eval_unary_op(op, &v)
                }
                
                Expression::Call { func, args } => {
                    self.eval_call(func, args, env)
                }
                
                Expression::If { condition, then_branch, else_branch } => {
                    let cond = self.eval_internal(condition, env)?;
                    match cond {
                        Value::Bool(true) => self.eval_internal(then_branch, env),
                        Value::Bool(false) => {
                            match else_branch {
                                Some(else_br) => self.eval_internal(else_br, env),
                                None => Ok(Value::Unit),
                            }
                        }
                        _ => Err(CtfeError::TypeError {
                            expected: "bool".to_string(),
                            found: format!("{:?}", cond),
                        }),
                    }
                }
                
                Expression::Loop { body, label } => {
                    self.eval_loop(body, env, label.clone())
                }
                
                Expression::Break { value, label } => {
                    let v = match value {
                        Some(val) => self.eval_internal(val, env)?,
                        None => Value::Unit,
                    };
                    Err(CtfeError::Break {
                        value: v,
                        label: label.clone(),
                    })
                }
                
                Expression::Return(value) => {
                    let v = match value {
                        Some(val) => self.eval_internal(val, env)?,
                        None => Value::Unit,
                    };
                    Err(CtfeError::Return(v))
                }
                
                Expression::Block(stmts, last) => {
                    self.eval_block(stmts, last.as_ref(), env)
                }
                
                Expression::VarDecl { name, ty: _, initializer, mutable: _ } => {
                    if let Some(init) = initializer {
                        let value = self.eval_internal(init, env)?;
                        env.insert(name.clone(), value.clone());
                        Ok(value)
                    } else {
                        let value = Value::Uninitialized;
                        env.insert(name.clone(), value.clone());
                        Ok(value)
                    }
                }
                
                Expression::Assign { target, value } => {
                    Err(CtfeError::ImmutableAssignment {
                        target: format!("{:?}", target),
                    })
                }
                
                Expression::Member { expr, member } => {
                    let obj = self.eval_internal(expr, env)?;
                    match obj {
                        Value::Struct(s) => {
                            if let Some(value) = s.fields.get(member) {
                                Ok(value.clone())
                            } else {
                                Err(CtfeError::FieldNotFound {
                                    struct_name: s.name,
                                    field: member.clone(),
                                })
                            }
                        }
                        _ => Err(CtfeError::TypeError {
                            expected: "struct".to_string(),
                            found: format!("{:?}", obj),
                        }),
                    }
                }
                
                Expression::Index { expr, index } => {
                    let arr = self.eval_internal(expr, env)?;
                    let idx = self.eval_internal(index, env)?;
                    match (arr, idx) {
                        (Value::Array(elements), Value::Int(i)) => {
                            if i >= 0 && (i as usize) < elements.len() {
                                Ok(elements[i as usize].clone())
                            } else {
                                Err(CtfeError::IndexOutOfBounds {
                                    len: elements.len(),
                                    index: i,
                                })
                            }
                        }
                        _ => Err(CtfeError::TypeError {
                            expected: "array".to_string(),
                            found: format!("{:?}", arr),
                        }),
                    }
                }
                
                Expression::Cast { expr, target_type } => {
                    let v = self.eval_internal(expr, env)?;
                    self.eval_cast(&v, target_type)
                }
                
                Expression::Tuple(elements) => {
                    let values: Result<Vec<Value>, CtfeError> = elements
                        .iter()
                        .map(|e| self.eval_internal(e, env))
                        .collect();
                    Ok(Value::Tuple(values?))
                }
                
                Expression::Array(elements) => {
                    let values: Result<Vec<Value>, CtfeError> = elements
                        .iter()
                        .map(|e| self.eval_internal(e, env))
                        .collect();
                    Ok(Value::Array(values?))
                }
                
                Expression::Struct { name, fields } => {
                    let mut field_map = HashMap::new();
                    for (field_name, field_expr) in fields {
                        let value = self.eval_internal(field_expr, env)?;
                        field_map.insert(field_name.clone(), value);
                    }
                    Ok(Value::Struct(StructValue {
                        name: name.clone(),
                        fields: field_map,
                    }))
                }
                
                Expression::Enum { enum_name, variant, value } => {
                    let v = match value {
                        Some(expr) => Some(self.eval_internal(expr, env)?),
                        None => None,
                    };
                    Ok(Value::Enum(EnumValue {
                        name: enum_name.clone(),
                        variant: variant.clone(),
                        value: v,
                    }))
                }
                
                Expression::SizeOf(ty) => {
                    let size = self.size_of(ty);
                    Ok(Value::Int(size as i128))
                }
                
                Expression::String(s) => Ok(Value::String(StdString::from(s.as_str()))),
                
                Expression::Tuple(vec) => {
                    let values: Vec<Value> = vec.iter()
                        .map(|e| self.eval_internal(e, env))
                        .collect::<Result<Vec<_>, _>>()?;
                    Ok(Value::Tuple(values))
                }
                
                Expression::Array(vec) => {
                    let values: Vec<Value> = vec.iter()
                        .map(|e| self.eval_internal(e, env))
                        .collect::<Result<Vec<_>, _>>()?;
                    Ok(Value::Array(values))
                }
                
                Expression::Identifier(name) => {
                    if let Some(value) = env.lookup(name) {
                        Ok(value)
                    } else {
                        Err(CtfeError::UndefinedVariable {
                            name: name.clone(),
                        })
                    }
                }
                
                _ => Err(CtfeError::UnsupportedExpression {
                    expr: format!("{:?}", expr),
                }),
            }
        }

        // ==================== 字面量求值 ====================

        fn eval_literal(&self, lit: &Literal) -> Result<Value, CtfeError> {
            match lit {
                Literal::Unit => Ok(Value::Unit),
                Literal::Bool(b) => Ok(Value::Bool(*b)),
                Literal::Int(n, _) => Ok(Value::Int(*n)),
                Literal::Uint(n, _) => Ok(Value::Uint(*n)),
                Literal::Float(f) => Ok(Value::Float(*f)),
                Literal::Char(c) => Ok(Value::Char(*c)),
                Literal::String(s) => Ok(Value::String(StdString::from(s.as_str()))),
                Literal::Bytes(b) => Ok(Value::Bytes(b.clone())),
            }
        }

        // ==================== 二元运算求值 ====================

        fn eval_binary_op(&self, op: &BinaryOp, left: &Value, right: &Value) -> Result<Value, CtfeError> {
            match op {
                // 算术运算
                BinaryOp::Add => self.eval_add(left, right),
                BinaryOp::Sub => self.eval_sub(left, right),
                BinaryOp::Mul => self.eval_mul(left, right),
                BinaryOp::Div => self.eval_div(left, right),
                BinaryOp::Mod => self.eval_mod(left, right),
                BinaryOp::Pow => self.eval_pow(left, right),
                
                // 位运算
                BinaryOp::BitAnd => self.eval_bitand(left, right),
                BinaryOp::BitOr => self.eval_bitor(left, right),
                BinaryOp::BitXor => self.eval_bitxor(left, right),
                BinaryOp::Shl => self.eval_shl(left, right),
                BinaryOp::Shr => self.eval_shr(left, right),
                
                // 比较运算
                BinaryOp::Eq => self.eval_eq(left, right),
                BinaryOp::Ne => self.eval_ne(left, right),
                BinaryOp::Lt => self.eval_lt(left, right),
                BinaryOp::Le => self.eval_le(left, right),
                BinaryOp::Gt => self.eval_gt(left, right),
                BinaryOp::Ge => self.eval_ge(left, right),
                
                // 逻辑运算
                BinaryOp::And => self.eval_and(left, right),
                BinaryOp::Or => self.eval_or(left, right),
                
                // 字符串运算
                BinaryOp::StrConcat => self.eval_str_concat(left, right),
            }
        }

        fn eval_add(&self, a: &Value, b: &Value) -> Result<Value, CtfeError> {
            match (a, b) {
                (Value::Int(i), Value::Int(j)) => Ok(Value::Int(i + j)),
                (Value::Float(f), Value::Float(g)) => Ok(Value::Float(f + g)),
                (Value::Int(i), Value::Float(f)) => Ok(Value::Float(*i as f64 + f)),
                (Value::Float(f), Value::Int(i)) => Ok(Value::Float(f + *i as f64)),
                _ => Err(CtfeError::TypeError {
                    expected: "numeric".to_string(),
                    found: format!("{:?} + {:?}", a, b),
                }),
            }
        }

        fn eval_sub(&self, a: &Value, b: &Value) -> Result<Value, CtfeError> {
            match (a, b) {
                (Value::Int(i), Value::Int(j)) => Ok(Value::Int(i - j)),
                (Value::Float(f), Value::Float(g)) => Ok(Value::Float(f - g)),
                (Value::Int(i), Value::Float(f)) => Ok(Value::Float(*i as f64 - f)),
                (Value::Float(f), Value::Int(i)) => Ok(Value::Float(f - *i as f64)),
                _ => Err(CtfeError::TypeError {
                    expected: "numeric".to_string(),
                    found: format!("{:?} - {:?}", a, b),
                }),
            }
        }

        fn eval_mul(&self, a: &Value, b: &Value) -> Result<Value, CtfeError> {
            match (a, b) {
                (Value::Int(i), Value::Int(j)) => Ok(Value::Int(i * j)),
                (Value::Float(f), Value::Float(g)) => Ok(Value::Float(f * g)),
                (Value::Int(i), Value::Float(f)) => Ok(Value::Float(*i as f64 * f)),
                (Value::Float(f), Value::Int(i)) => Ok(Value::Float(f * *i as f64)),
                _ => Err(CtfeError::TypeError {
                    expected: "numeric".to_string(),
                    found: format!("{:?} * {:?}", a, b),
                }),
            }
        }

        fn eval_div(&self, a: &Value, b: &Value) -> Result<Value, CtfeError> {
            match (a, b) {
                (Value::Int(i), Value::Int(j)) => {
                    if *j == 0 {
                        Err(CtfeError::DivisionByZero)
                    } else {
                        Ok(Value::Int(i / j))
                    }
                }
                (Value::Float(f), Value::Float(g)) => {
                    if *g == 0.0 {
                        Err(CtfeError::DivisionByZero)
                    } else {
                        Ok(Value::Float(f / g))
                    }
                }
                _ => Err(CtfeError::TypeError {
                    expected: "numeric".to_string(),
                    found: format!("{:?} / {:?}", a, b),
                }),
            }
        }

        fn eval_mod(&self, a: &Value, b: &Value) -> Result<Value, CtfeError> {
            match (a, b) {
                (Value::Int(i), Value::Int(j)) => {
                    if *j == 0 {
                        Err(CtfeError::DivisionByZero)
                    } else {
                        Ok(Value::Int(i % j))
                    }
                }
                _ => Err(CtfeError::TypeError {
                    expected: "int".to_string(),
                    found: format!("{:?} % {:?}", a, b),
                }),
            }
        }

        fn eval_pow(&self, a: &Value, b: &Value) -> Result<Value, CtfeError> {
            match (a, b) {
                (Value::Int(i), Value::Int(j)) => {
                    if *j < 0 {
                        Err(CtfeError::NegativeExponent)
                    } else {
                        Ok(Value::Int(i.pow(j as u32)))
                    }
                }
                (Value::Float(f), Value::Float(g)) => Ok(Value::Float(f.powf(*g))),
                _ => Err(CtfeError::TypeError {
                    expected: "numeric".to_string(),
                    found: format!("{:?} ** {:?}", a, b),
                }),
            }
        }

        fn eval_bitand(&self, a: &Value, b: &Value) -> Result<Value, CtfeError> {
            match (a, b) {
                (Value::Int(i), Value::Int(j)) => Ok(Value::Int(i & j)),
                (Value::Uint(i), Value::Uint(j)) => Ok(Value::Uint(i & j)),
                _ => Err(CtfeError::TypeError {
                    expected: "integer".to_string(),
                    found: format!("{:?} & {:?}", a, b),
                }),
            }
        }

        fn eval_bitor(&self, a: &Value, b: &Value) -> Result<Value, CtfeError> {
            match (a, b) {
                (Value::Int(i), Value::Int(j)) => Ok(Value::Int(i | j)),
                (Value::Uint(i), Value::Uint(j)) => Ok(Value::Uint(i | j)),
                _ => Err(CtfeError::TypeError {
                    expected: "integer".to_string(),
                    found: format!("{:?} | {:?}", a, b),
                }),
            }
        }

        fn eval_bitxor(&self, a: &Value, b: &Value) -> Result<Value, CtfeError> {
            match (a, b) {
                (Value::Int(i), Value::Int(j)) => Ok(Value::Int(i ^ j)),
                (Value::Uint(i), Value::Uint(j)) => Ok(Value::Uint(i ^ j)),
                _ => Err(CtfeError::TypeError {
                    expected: "integer".to_string(),
                    found: format!("{:?} ^ {:?}", a, b),
                }),
            }
        }

        fn eval_shl(&self, a: &Value, b: &Value) -> Result<Value, CtfeError> {
            match (a, b) {
                (Value::Int(i), Value::Int(j)) => {
                    if *j < 0 || *j >= 128 {
                        Err(CtfeError::InvalidShiftAmount)
                    } else {
                        Ok(Value::Int(i << j))
                    }
                }
                (Value::Uint(i), Value::Uint(j)) => {
                    if *j >= 128 {
                        Err(CtfeError::InvalidShiftAmount)
                    } else {
                        Ok(Value::Uint(i << j))
                    }
                }
                _ => Err(CtfeError::TypeError {
                    expected: "integer".to_string(),
                    found: format!("{:?} << {:?}", a, b),
                }),
            }
        }

        fn eval_shr(&self, a: &Value, b: &Value) -> Result<Value, CtfeError> {
            match (a, b) {
                (Value::Int(i), Value::Int(j)) => {
                    if *j < 0 || *j >= 128 {
                        Err(CtfeError::InvalidShiftAmount)
                    } else {
                        Ok(Value::Int(i >> j))
                    }
                }
                (Value::Uint(i), Value::Uint(j)) => {
                    if *j >= 128 {
                        Err(CtfeError::InvalidShiftAmount)
                    } else {
                        Ok(Value::Uint(i >> j))
                    }
                }
                _ => Err(CtfeError::TypeError {
                    expected: "integer".to_string(),
                    found: format!("{:?} >> {:?}", a, b),
                }),
            }
        }

        fn eval_eq(&self, a: &Value, b: &Value) -> Result<Value, CtfeError> {
            Ok(Value::Bool(a == b))
        }

        fn eval_ne(&self, a: &Value, b: &Value) -> Result<Value, CtfeError> {
            Ok(Value::Bool(a != b))
        }

        fn eval_lt(&self, a: &Value, b: &Value) -> Result<Value, CtfeError> {
            Ok(Value::Bool(a < b))
        }

        fn eval_le(&self, a: &Value, b: &Value) -> Result<Value, CtfeError> {
            Ok(Value::Bool(a <= b))
        }

        fn eval_gt(&self, a: &Value, b: &Value) -> Result<Value, CtfeError> {
            Ok(Value::Bool(a > b))
        }

        fn eval_ge(&self, a: &Value, b: &Value) -> Result<Value, CtfeError> {
            Ok(Value::Bool(a >= b))
        }

        fn eval_and(&self, a: &Value, b: &Value) -> Result<Value, CtfeError> {
            match (a, b) {
                (Value::Bool(x), Value::Bool(y)) => Ok(Value::Bool(*x && *y)),
                _ => Err(CtfeError::TypeError {
                    expected: "bool".to_string(),
                    found: format!("{:?} && {:?}", a, b),
                }),
            }
        }

        fn eval_or(&self, a: &Value, b: &Value) -> Result<Value, CtfeError> {
            match (a, b) {
                (Value::Bool(x), Value::Bool(y)) => Ok(Value::Bool(*x || *y)),
                _ => Err(CtfeError::TypeError {
                    expected: "bool".to_string(),
                    found: format!("{:?} || {:?}", a, b),
                }),
            }
        }

        fn eval_str_concat(&self, a: &Value, b: &Value) -> Result<Value, CtfeError> {
            match (a, b) {
                (Value::String(s1), Value::String(s2)) => {
                    let mut result = StdString::new();
                    result.push_str(s1.as_str());
                    result.push_str(s2.as_str());
                    Ok(Value::String(result))
                }
                _ => Err(CtfeError::TypeError {
                    expected: "string".to_string(),
                    found: format!("{:?} ~ {:?}", a, b),
                }),
            }
        }

        // ==================== 一元运算求值 ====================

        fn eval_unary_op(&self, op: &UnaryOp, v: &Value) -> Result<Value, CtfeError> {
            match op {
                UnaryOp::Neg => {
                    match v {
                        Value::Int(i) => Ok(Value::Int(-i)),
                        Value::Float(f) => Ok(Value::Float(-f)),
                        _ => Err(CtfeError::TypeError {
                            expected: "numeric".to_string(),
                            found: format!("-{:?}", v),
                        }),
                    }
                }
                UnaryOp::Not => {
                    match v {
                        Value::Bool(b) => Ok(Value::Bool(!b)),
                        _ => Err(CtfeError::TypeError {
                            expected: "bool".to_string(),
                            found: format!("!{:?}", v),
                        }),
                    }
                }
                UnaryOp::BitNot => {
                    match v {
                        Value::Int(i) => Ok(Value::Int(!i)),
                        Value::Uint(u) => Ok(Value::Uint(!u)),
                        _ => Err(CtfeError::TypeError {
                            expected: "integer".to_string(),
                            found: format!("~{:?}", v),
                        }),
                    }
                }
            }
        }

        // ==================== 函数调用求值 ====================

        fn eval_call(&mut self, func: &Expression, args: &[Expression], env: &mut Env) -> Result<Value, CtfeError> {
            match func {
                Expression::Identifier(name) => {
                    // 内置函数
                    if let Some(result) = self.eval_builtin(name, args, env)? {
                        return result;
                    }
                    
                    // 用户定义函数
                    if let Some(fn_value) = env.lookup_fn(name) {
                        self.eval_function_call(&fn_value, args, env)
                    } else {
                        Err(CtfeError::UndefinedFunction {
                            name: name.clone(),
                        })
                    }
                }
                _ => Err(CtfeError::UnsupportedExpression {
                    expr: "function call on non-identifier".to_string(),
                }),
            }
        }

        fn eval_builtin(&mut self, name: &string, args: &[Expression], env: &mut Env) -> Result<Option<Result<Value, CtfeError>>, CtfeError> {
            match name.as_str() {
                // 字符串函数
                "len" | "length" => {
                    let arg = self.eval_internal(&args[0], env)?;
                    match arg {
                        Value::String(s) => Ok(Some(Ok(Value::Int(s.len() as i128)))),
                        Value::Array(arr) => Ok(Some(Ok(Value::Int(arr.len() as i128)))),
                        _ => Ok(Some(Err(CtfeError::TypeError {
                            expected: "string or array".to_string(),
                            found: format!("len({:?})", arg),
                        }))),
                    }
                }
                
                // 数学函数
                "abs" => {
                    let arg = self.eval_internal(&args[0], env)?;
                    match arg {
                        Value::Int(i) => Ok(Some(Ok(Value::Int(i.abs()))),
                        Value::Float(f) => Ok(Some(Ok(Value::Float(f.abs()))),
                        _ => Ok(Some(Err(CtfeError::TypeError {
                            expected: "numeric".to_string(),
                            found: format!("abs({:?})", arg),
                        }))),
                    }
                }
                
                "sqrt" => {
                    let arg = self.eval_internal(&args[0], env)?;
                    match arg {
                        Value::Float(f) => Ok(Some(Ok(Value::Float(f.sqrt())))),
                        Value::Int(i) => Ok(Some(Ok(Value::Float((*i as f64).sqrt()))),
                        _ => Ok(Some(Err(CtfeError::TypeError {
                            expected: "numeric".to_string(),
                            found: format!("sqrt({:?})", arg),
                        }))),
                    }
                }
                
                "pow" => {
                    let base = self.eval_internal(&args[0], env)?;
                    let exp = self.eval_internal(&args[1], env)?;
                    self.eval_binary_op(&BinaryOp::Pow, &base, &exp).map(Some)
                }
                
                "min" => {
                    let a = self.eval_internal(&args[0], env)?;
                    let b = self.eval_internal(&args[1], env)?;
                    Ok(Some(Ok(if a <= b { a } else { b })))
                }
                
                "max" => {
                    let a = self.eval_internal(&args[0], env)?;
                    let b = self.eval_internal(&args[1], env)?;
                    Ok(Some(Ok(if a >= b { a } else { b })))
                }
                
                // 类型转换
                "int" => {
                    let arg = self.eval_internal(&args[0], env)?;
                    match arg {
                        Value::Float(f) => Ok(Some(Ok(Value::Int(f as i128)))),
                        Value::String(s) => {
                            if let Ok(n) = s.as_str().parse::<i128>() {
                                Ok(Some(Ok(Value::Int(n))))
                            } else {
                                Ok(Some(Err(CtfeError::TypeError {
                                    expected: "parsable string".to_string(),
                                    found: format!("int(\"{}\")", s),
                                })))
                            }
                        }
                        _ => Ok(Some(Err(CtfeError::TypeError {
                            expected: "convertible to int".to_string(),
                            found: format!("int({:?})", arg),
                        }))),
                    }
                }
                
                "float" => {
                    let arg = self.eval_internal(&args[0], env)?;
                    match arg {
                        Value::Int(i) => Ok(Some(Ok(Value::Float(*i as f64)))),
                        Value::String(s) => {
                            if let Ok(f) = s.as_str().parse::<f64>() {
                                Ok(Some(Ok(Value::Float(f))))
                            } else {
                                Ok(Some(Err(CtfeError::TypeError {
                                    expected: "parsable string".to_string(),
                                    found: format!("float(\"{}\")", s),
                                })))
                            }
                        }
                        _ => Ok(Some(Err(CtfeError::TypeError {
                            expected: "convertible to float".to_string(),
                            found: format!("float({:?})", arg),
                        }))),
                    }
                }
                
                "string" => {
                    let arg = self.eval_internal(&args[0], env)?;
                    Ok(Some(Ok(Value::String(arg.to_string()))))
                }
                
                _ => Ok(None),  // 非内置函数
            }
        }

        fn eval_function_call(&mut self, fn_value: &FnValue, args: &[Expression], env: &mut Env) -> Result<Value, CtfeError> {
            // 评估参数
            let mut arg_values = Vec::new();
            for arg in args {
                arg_values.push(self.eval_internal(arg, env)?);
            }
            
            // 检查参数数量
            if arg_values.len() != fn_value.params.len() {
                return Err(CtfeError::ArgumentCountMismatch {
                    expected: fn_value.params.len(),
                    found: arg_values.len(),
                });
            }
            
            // 创建新环境
            let mut call_env = Env::new();
            
            // 复制父作用域的函数（但不是局部变量）
            if let Some(ref parent) = fn_value.env.parent {
                call_env.parent = Some(parent.clone());
            }
            
            // 绑定参数
            for (param, value) in fn_value.params.iter().zip(arg_values.iter()) {
                call_env.insert(param.name.clone(), value.clone());
            }
            
            // 执行函数体
            self.eval_internal(&fn_value.body, &mut call_env)
        }

        // ==================== 循环求值 ====================

        fn eval_loop(&mut self, body: &Expression, env: &mut Env, label: Option<string>) -> Result<Value, CtfeError> {
            loop {
                match self.eval_internal(body, env) {
                    Ok(_) => continue,
                    Err(CtfeError::Break { value, label: break_label }) => {
                        if label == break_label || (label.is_none() && break_label.is_none()) {
                            return Ok(value);
                        }
                        return Err(CtfeError::Break { value, label: break_label });
                    }
                    Err(CtfeError::Return(v)) => return Err(CtfeError::Return(v)),
                    Err(e) => return Err(e),
                }
            }
        }

        // ==================== Block 求值 ====================

        fn eval_block(&mut self, stmts: &[Statement], last: Option<&Expression>, env: &mut Env) -> Result<Value, CtfeError> {
            env.enter_scope();
            
            // 执行语句
            for stmt in stmts {
                match stmt {
                    Statement::Expression(expr) => {
                        self.eval_internal(expr, env)?;
                    }
                    Statement::Let { pattern, ty: _, initializer, mutable: _ } => {
                        if let Some(init) = initializer {
                            let value = self.eval_internal(init, env)?;
                            if let Pattern::Identifier(name) = pattern {
                                env.insert(name.clone(), value);
                            }
                        }
                    }
                    Statement::While { condition, body, label } => {
                        loop {
                            let cond = self.eval_internal(condition, env)?;
                            match cond {
                                Value::Bool(true) => {
                                    match self.eval_internal(body, env) {
                                        Ok(_) => continue,
                                        Err(CtfeError::Break { value, label: break_label }) => {
                                            if label == &break_label || (label.is_none() && break_label.is_none()) {
                                                env.exit_scope();
                                                return Ok(value);
                                            }
                                            return Err(CtfeError::Break { value, label: break_label });
                                        }
                                        Err(CtfeError::Return(v)) => {
                                            env.exit_scope();
                                            return Err(CtfeError::Return(v));
                                        }
                                        Err(e) => {
                                            env.exit_scope();
                                            return Err(e);
                                        }
                                    }
                                }
                                Value::Bool(false) => break,
                                _ => {
                                    env.exit_scope();
                                    return Err(CtfeError::TypeError {
                                        expected: "bool".to_string(),
                                        found: "while condition".to_string(),
                                    });
                                }
                            }
                        }
                    }
                    Statement::For { init: _, condition, update, body, label } => {
                        if let Some(cond) = condition {
                            loop {
                                let c = self.eval_internal(cond, env)?;
                                match c {
                                    Value::Bool(true) => {
                                        match self.eval_internal(body, env) {
                                            Ok(_) => {},
                                            Err(CtfeError::Break { value, label: break_label }) => {
                                                if label == &break_label || (label.is_none() && break_label.is_none()) {
                                                    env.exit_scope();
                                                    return Ok(value);
                                                }
                                                return Err(CtfeError::Break { value, label: break_label });
                                            }
                                            Err(CtfeError::Return(v)) => {
                                                env.exit_scope();
                                                return Err(CtfeError::Return(v));
                                            }
                                            Err(e) => {
                                                env.exit_scope();
                                                return Err(e);
                                            }
                                        }
                                        if let Some(upd) = update {
                                            self.eval_internal(upd, env)?;
                                        }
                                    }
                                    Value::Bool(false) => break,
                                    _ => {
                                        env.exit_scope();
                                        return Err(CtfeError::TypeError {
                                            expected: "bool".to_string(),
                                            found: "for condition".to_string(),
                                        });
                                    }
                                }
                            }
                        }
                    }
                    Statement::Break(l) => {
                        return Err(CtfeError::Break {
                            value: Value::Unit,
                            label: l.clone(),
                        });
                    }
                    Statement::Continue(l) => {
                        continue;
                    }
                    Statement::Return(value) => {
                        let v = match value {
                            Some(val) => self.eval_internal(val, env)?,
                            None => Value::Unit,
                        };
                        env.exit_scope();
                        return Err(CtfeError::Return(v));
                    }
                    Statement::Block(expr) => {
                        self.eval_internal(expr, env)?;
                    }
                    Statement::Empty => {}
                }
            }
            
            env.exit_scope();
            
            // 执行最后的表达式
            if let Some(last_expr) = last {
                self.eval_internal(last_expr, env)
            } else {
                Ok(Value::Unit)
            }
        }

        // ==================== 类型转换求值 ====================

        fn eval_cast(&self, value: &Value, target_type: &Type) -> Result<Value, CtfeError> {
            match (value, target_type) {
                (Value::Int(i), Type::Float) => Ok(Value::Float(*i as f64)),
                (Value::Float(f), Type::Int(_)) => Ok(Value::Int(*f as i128)),
                (Value::Float(f), Type::Float) => Ok(Value::Float(*f)),
                (Value::Bool(b), Type::Int(_)) => Ok(Value::Int(if *b { 1 } else { 0 })),
                (Value::Int(i), Type::Bool) => Ok(Value::Bool(*i != 0)),
                (Value::Char(c), Type::Int(_)) => Ok(Value::Int(*c as i128)),
                (Value::Int(i), Type::Char) => Ok(Value::Char(*i as u8 as char)),
                _ => Err(CtfeError::TypeError {
                    expected: format!("{:?}", target_type),
                    found: format!("{:?}", value),
                }),
            }
        }

        // ==================== SizeOf ====================

        fn size_of(&self, ty: &Type) -> usize {
            match ty {
                Type::Unit => 0,
                Type::Bool => 1,
                Type::Int(8) => 1,
                Type::Int(16) => 2,
                Type::Int(32) => 4,
                Type::Int(64) => 8,
                Type::Float => 8,
                Type::Char => 4,
                Type::String => 16,  // 指针 + 长度
                Type::Array(t, Some(n)) => self.size_of(t) * n,
                Type::Tuple(types) => types.iter().map(|t| self.size_of(t)).sum(),
                _ => 8,  // 默认指针大小
            }
        }
    }

    // ==================== 错误类型 ====================

    #[derive(Debug, Clone)]
    pub enum CtfeError {
        TooManySteps { limit: usize },
        DivisionByZero,
        NegativeExponent,
        InvalidShiftAmount,
        UndefinedVariable { name: string },
        UndefinedFunction { name: string },
        TypeError { expected: string, found: string },
        IndexOutOfBounds { len: usize, index: i128 },
        FieldNotFound { struct_name: string, field: string },
        ArgumentCountMismatch { expected: usize, found: usize },
        ImmutableAssignment { target: string },
        UnsupportedExpression { expr: string },
        Break { value: Value, label: Option<string> },
        Return(Value),
    }

    impl std::fmt::Display for CtfeError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                CtfeError::TooManySteps { limit } => write!(f, "CTFE exceeded {} steps limit", limit),
                CtfeError::DivisionByZero => write!(f, "division by zero"),
                CtfeError::NegativeExponent => write!(f, "negative exponent"),
                CtfeError::InvalidShiftAmount => write!(f, "invalid shift amount"),
                CtfeError::UndefinedVariable { name } => write!(f, "undefined variable '{}'", name),
                CtfeError::UndefinedFunction { name } => write!(f, "undefined function '{}'", name),
                CtfeError::TypeError { expected, found } => write!(f, "type error: expected {}, found {}", expected, found),
                CtfeError::IndexOutOfBounds { len, index } => write!(f, "index out of bounds: len={}, index={}", len, index),
                CtfeError::FieldNotFound { struct_name, field } => write!(f, "field '{}' not found in struct '{}'", field, struct_name),
                CtfeError::ArgumentCountMismatch { expected, found } => write!(f, "argument count mismatch: expected {}, found {}", expected, found),
                CtfeError::ImmutableAssignment { target } => write!(f, "cannot assign to immutable target: {}", target),
                CtfeError::UnsupportedExpression { expr } => write!(f, "unsupported expression: {}", expr),
                CtfeError::Break { value: _, label } => write!(f, "break{}", label.as_ref().map_or("".to_string(), |l| format!(" '{}'", l))),
                CtfeError::Return(v) => write!(f, "return {:?}", v),
            }
        }
    }
}

// ==================== Value Display 实现 ====================

impl std::fmt::Display for ctfe::Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ctfe::Value::Unit => write!(f, "()"),
            ctfe::Value::Bool(b) => write!(f, "{}", b),
            ctfe::Value::Int(i) => write!(f, "{}", i),
            ctfe::Value::Uint(u) => write!(f, "{}", u),
            ctfe::Value::Float(fl) => write!(f, "{}", fl),
            ctfe::Value::Char(c) => write!(f, "'{}'", c),
            ctfe::Value::String(s) => write!(f, "\"{}\"", s.as_str()),
            ctfe::Value::Bytes(b) => write!(f, "{:?}", b),
            ctfe::Value::Array(arr) => {
                write!(f, "[")?;
                for (i, v) in arr.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}", v)?;
                }
                write!(f, "]")
            }
            ctfe::Value::Tuple(t) => {
                write!(f, "(")?;
                for (i, v) in t.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}", v)?;
                }
                if t.len() == 1 { write!(f, ",")?; }
                write!(f, ")")
            }
            ctfe::Value::Struct(s) => {
                write!(f, "{} {{ ", s.name)?;
                for (i, (name, value)) in s.fields.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}: {}", name, value)?;
                }
                write!(f, " }}")
            }
            ctfe::Value::Enum(e) => {
                if let Some(v) = &e.value {
                    write!(f, "{}::{}<{}>", e.name, e.variant, v)
                } else {
                    write!(f, "{}::{}", e.name, e.variant)
                }
            }
            ctfe::Value::Function(fn_val) => write!(f, "fn {}", fn_val.name),
            ctfe::Value::Closure(_) => write!(f, "closure"),
            ctfe::Value::Pointer(p) => write!(f, "ptr{:?}", p),
            ctfe::Value::Error(e) => write!(f, "CTFE Error: {}", e.message),
            ctfe::Value::Uninitialized => write!(f, "<uninitialized>"),
        }
    }
}
