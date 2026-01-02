use std::fmt::{self, Display};

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    // 字面量
    Literal(Literal),
    // 标识符
    Identifier(String),
    // 一元操作符
    UnaryOp {
        op: UnaryOperator,
        expr: Box<Expression>,
    },
    // 二元操作符
    BinaryOp {
        left: Box<Expression>,
        op: BinaryOperator,
        right: Box<Expression>,
    },
    // 函数调用
    Call {
        callee: Box<Expression>,
        args: Vec<Expression>,
    },
    // 数组索引
    Index {
        array: Box<Expression>,
        index: Box<Expression>,
    },
    // 字段访问
    FieldAccess {
        expr: Box<Expression>,
        field: String,
    },
    // 赋值表达式
    Assign {
        left: Box<Expression>,
        right: Box<Expression>,
    },
    // 块表达式
    Block(Vec<Statement>),
    // If表达式
    If {
        condition: Box<Expression>,
        then_branch: Box<Expression>,
        else_branch: Option<Box<Expression>>,
    },
    // Match表达式
    Match {
        expr: Box<Expression>,
        cases: Vec<MatchCase>,
    },
    // Lambda表达式
    Lambda {
        params: Vec<Parameter>,
        body: Box<Expression>,
    },
    // 范围表达式
    Range {
        start: Box<Expression>,
        end: Box<Expression>,
        inclusive: bool,
    },
    // 数组表达式
    Array(Vec<Expression>),
    // 结构体表达式
    Struct {
        name: String,
        fields: Vec<(String, Expression)>,
    },
    // 组表达式
    Group {
        name: String,
        members: Vec<Statement>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Integer(i64),
    Float(f64),
    String(String),
    Boolean(bool),
    Null,
    // 物理单位字面量
    UnitLiteral(f64, String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOperator {
    Not,
    Neg,
    Ref,
    Deref,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOperator {
    // 算术操作符
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    // 比较操作符
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    // 逻辑操作符
    And,
    Or,
    // 赋值操作符
    Assign,
    AddAssign,
    SubAssign,
    MulAssign,
    DivAssign,
    ModAssign,
    // 范围操作符
    Range,
    RangeInclusive,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    // 表达式语句
    Expression(Expression),
    // 变量声明
    Let {
        mutable: bool,
        name: String,
        ty: Option<String>,
        value: Expression,
    },
    // 函数声明
    Function {
        name: String,
        params: Vec<Parameter>,
        return_type: Option<String>,
        body: Expression,
        kernel: bool,
    },
    // 结构体声明
    Struct {
        name: String,
        fields: Vec<StructField>,
    },
    // 枚举声明
    Enum {
        name: String,
        variants: Vec<(String, Option<Vec<StructField>>)>,
    },
    // 组声明
    Group {
        name: String,
        members: Vec<Statement>,
    },
    // ECS实体声明
    Entity {
        name: String,
        components: Vec<String>,
    },
    // ECS组件声明
    Component {
        name: String,
        fields: Vec<StructField>,
    },
    // ECS系统声明
    System {
        name: String,
        query: Vec<String>,  // 查询的组件类型
        body: Expression,
    },
    // 返回语句
    Return(Option<Expression>),
    // 循环语句
    While {
        condition: Expression,
        body: Expression,
    },
    // For循环
    For {
        pattern: String,
        in_expr: Expression,
        body: Expression,
    },
    // 导入语句
    Import(String),
    // 模块别名
    ImportAs(String, String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Parameter {
    pub name: String,
    pub ty: Option<String>,
    pub default: Option<Expression>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StructField {
    pub name: String,
    pub ty: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MatchCase {
    pub pattern: Expression,
    pub guard: Option<Expression>,
    pub body: Expression,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub statements: Vec<Statement>,
}

impl Display for Expression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expression::Literal(lit) => write!(f, "{}", lit),
            Expression::Identifier(id) => write!(f, "{}", id),
            Expression::UnaryOp { op, expr } => write!(f, "{}{}", op, expr),
            Expression::BinaryOp { left, op, right } => write!(f, "{} {} {}", left, op, right),
            Expression::Call { callee, args } => {
                write!(f, "{}(", callee)?;
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}", arg)?;
                }
                write!(f, ")")
            },
            Expression::Index { array, index } => write!(f, "{}[{index}]", array),
            Expression::FieldAccess { expr, field } => write!(f, "{expr}.{field}"),
            Expression::Assign { left, right } => write!(f, "{left} = {right}"),
            Expression::Block(stmts) => {
                write!(f, "{{")?;
                for stmt in stmts {
                    write!(f, "\n  {stmt}")?;
                }
                write!(f, "\n}}")
            },
            Expression::If { condition, then_branch, else_branch } => {
                write!(f, "if {condition}: {then_branch}")?;
                if let Some(else_branch) = else_branch {
                    write!(f, " else: {else_branch}")?;
                }
                Ok(())
            },
            Expression::Match { expr, cases } => {
                write!(f, "match {expr}:\n")?;
                for case in cases {
                    write!(f, "  {case}\n")?;
                }
                Ok(())
            },
            Expression::Lambda { params, body } => {
                write!(f, "(")?;
                for (i, param) in params.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{param}")?;
                }
                write!(f, ") -> {body}")
            },
            Expression::Range { start, end, inclusive } => {
                if *inclusive {
                    write!(f, "{start}..={end}")
                } else {
                    write!(f, "{start}..{end}")
                }
            },
            Expression::Array(items) => {
                write!(f, "[")?;
                for (i, item) in items.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{item}")?;
                }
                write!(f, "]")
            },
            Expression::Struct { name, fields } => {
                write!(f, "{name} {{")?;
                for (i, (field, value)) in fields.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{field}: {value}")?;
                }
                write!(f, "}}")
            },
            Expression::Group { name, members } => {
                write!(f, "group {name} {{")?;
                for member in members {
                    write!(f, "\n  {member}")?;
                }
                write!(f, "\n}}")
            },
        }
    }
}

impl Display for Literal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Literal::Integer(n) => write!(f, "{n}"),
            Literal::Float(n) => write!(f, "{n}"),
            Literal::String(s) => write!(f, "\"{s}\""),
            Literal::Boolean(b) => write!(f, "{b}"),
            Literal::Null => write!(f, "null"),
            Literal::UnitLiteral(value, unit) => write!(f, "{value}{unit}"),
        }
    }
}

impl Display for UnaryOperator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UnaryOperator::Not => write!(f, "!"),
            UnaryOperator::Neg => write!(f, "-"),
            UnaryOperator::Ref => write!(f, "&"),
            UnaryOperator::Deref => write!(f, "*"),
        }
    }
}

impl Display for BinaryOperator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BinaryOperator::Add => write!(f, "+"),
            BinaryOperator::Sub => write!(f, "-"),
            BinaryOperator::Mul => write!(f, "*"),
            BinaryOperator::Div => write!(f, "/"),
            BinaryOperator::Mod => write!(f, "%"),
            BinaryOperator::Eq => write!(f, "=="),
            BinaryOperator::Ne => write!(f, "!="),
            BinaryOperator::Lt => write!(f, "<"),
            BinaryOperator::Le => write!(f, "<="),
            BinaryOperator::Gt => write!(f, ">"),
            BinaryOperator::Ge => write!(f, ">="),
            BinaryOperator::And => write!(f, "&&"),
            BinaryOperator::Or => write!(f, "||"),
            BinaryOperator::Assign => write!(f, "="),
            BinaryOperator::AddAssign => write!(f, "+="),
            BinaryOperator::SubAssign => write!(f, "-="),
            BinaryOperator::MulAssign => write!(f, "*="),
            BinaryOperator::DivAssign => write!(f, "/="),
            BinaryOperator::ModAssign => write!(f, "%="),
            BinaryOperator::Range => write!(f, ".."),
            BinaryOperator::RangeInclusive => write!(f, "..="),
        }
    }
}

impl Display for Statement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Statement::Expression(expr) => write!(f, "{expr};")?,
            Statement::Let { mutable, name, ty, value } => {
                write!(f, "{}", if *mutable { "var" } else { "let" })?;
                write!(f, " {name}")?;
                if let Some(ty) = ty {
                    write!(f, ": {ty}")?;
                }
                write!(f, " = {value};" )?
            },
            Statement::Function { name, params, return_type, body, kernel } => {
                if *kernel {
                    write!(f, "@kernel ")?;
                }
                write!(f, "fn {name}(" )?;
                for (i, param) in params.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{param}")?;
                }
                write!(f, ")")?;
                if let Some(return_type) = return_type {
                    write!(f, " -> {return_type}")?;
                }
                write!(f, " = {body};" )?
            },
            Statement::Struct { name, fields } => {
                write!(f, "struct {name} {{")?;
                for field in fields {
                    write!(f, "\n  {field};")?;
                }
                write!(f, "\n}}" )?
            },
            Statement::Enum { name, variants } => {
                write!(f, "enum {name} {{")?;
                for (i, (variant, fields)) in variants.iter().enumerate() {
                    if i > 0 { write!(f, ",")?; }
                    write!(f, "\n  {variant}")?;
                    if let Some(fields) = fields {
                        write!(f, "(")?;
                        for (j, field) in fields.iter().enumerate() {
                            if j > 0 { write!(f, ", ")?; }
                            write!(f, "{field}")?;
                        }
                        write!(f, ")")?;
                    }
                }
                write!(f, "\n}}" )?
            },
            Statement::Group { name, members } => {
                write!(f, "group {name} {{")?;
                for member in members {
                    write!(f, "\n  {member}")?;
                }
                write!(f, "\n}}" )?
            },
            Statement::Return(expr) => {
                write!(f, "return")?;
                if let Some(expr) = expr {
                    write!(f, " {expr}")?;
                }
                write!(f, ";" )?
            },
            Statement::While { condition, body } => {
                write!(f, "while {condition}: {body}")?
            },
            Statement::For { pattern, in_expr, body } => {
                write!(f, "for {pattern} in {in_expr}: {body}")?
            },
            Statement::Import(path) => write!(f, "import \"{path}\";")?,
            Statement::ImportAs(path, alias) => write!(f, "import \"{path}\" as {alias};")?,
            // ECS声明
            Statement::Entity { name, components } => {
                write!(f, "entity {name} {{")?;
                for comp in components {
                    write!(f, " {comp}")?;
                }
                write!(f, " }}" )?
            },
            Statement::Component { name, fields } => {
                write!(f, "component {name} {{")?;
                for field in fields {
                    write!(f, "\n  {field};")?;
                }
                write!(f, "\n}}" )?
            },
            Statement::System { name, query, body } => {
                write!(f, "system {name} query(")?;
                for (i, comp) in query.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{comp}")?;
                }
                write!(f, ") {body}" )?
            },
        }
        Ok(())
    }
}

impl Display for Parameter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)?;
        if let Some(ty) = &self.ty {
            write!(f, ": {}", ty)?;
        }
        if let Some(default) = &self.default {
            write!(f, " = {}", default)?;
        }
        Ok(())
    }
}

impl Display for StructField {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{name}: {ty}", name = self.name, ty = self.ty)
    }
}

impl Display for MatchCase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.pattern)?;
        if let Some(guard) = &self.guard {
            write!(f, " if {}", guard)?;
        }
        write!(f, " => {}", self.body)
    }
}

/// 值类型修饰符
/// 用于标记类型为值类型（默认）或引用类型
#[derive(Debug, Clone, PartialEq)]
pub enum TypeModifier {
    Value,      // 值类型（默认）
    Ref,        // 引用类型（共享引用）
    MutRef,     // 可变引用
}

/// 类型表达式
#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    /// 基本类型
    Int,           // int - 整数
    Float,         // float - 浮点数
    Bool,          // bool - 布尔值
    String,        // string - 字符串
    Void,          // void - 空类型
    
    /// 修饰的基本类型（带生命周期）
    RefInt(String),           // &int - 整数引用
    RefFloat(String),         // &float - 浮点数引用
    RefBool(String),          // &bool - 布尔值引用
    RefString(String),        // &string - 字符串引用
    MutRefInt(String),        // &mut int - 可变整数引用
    MutRefFloat(String),      // &mut float - 可变浮点数引用
    MutRefBool(String),       // &mut bool - 可变布尔值引用
    MutRefString(String),     // &mut string - 可变字符串引用
    
    /// 复合类型
    List(Box<Type>),          // List[T] - 列表
    Optional(Box<Type>),      // Optional[T] - 可选类型
    Result(Box<Type>, Box<Type>),  // Result[T, E] - 结果类型
    
    /// 用户自定义类型
    Struct(String, Vec<(String, Type)>),  // 结构体类型
    Enum(String, Vec<EnumVariant>),       // 枚举类型
    
    /// 函数类型
    Function(Vec<Type>, Box<Type>),  // fn(Params) -> ReturnType
    
    /// 泛型类型
    Generic(String, Vec<Type>),      // GenericType[T, U, ...]
    
    /// 元组类型
    Tuple(Vec<Type>),                // (T1, T2, ...)
    
    /// 命名类型
    Named(String),                   // 类型别名或自定义类型名
    
    /// 类型参数（用于泛型）
    TypeParam(String),               // T, U, V...
    
    /// 范围类型
    Range(Box<Type>),                // Range[T]
    RangeInclusive(Box<Type>),       // RangeInclusive[T]
}

/// 枚举变体
#[derive(Debug, Clone, PartialEq)]
pub struct EnumVariant {
    pub name: String,
    pub fields: Vec<Type>,
    pub tag: u32,
}

/// 类型注解
#[derive(Debug, Clone, PartialEq)]
pub struct TypeAnnotation {
    pub modifier: Option<TypeModifier>,  // 类型修饰符
    pub type_expr: Type,                 // 类型表达式
    pub lifetime: Option<String>,        // 生命周期参数（如 'a）
}

impl Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Int => write!(f, "int"),
            Type::Float => write!(f, "float"),
            Type::Bool => write!(f, "bool"),
            Type::String => write!(f, "string"),
            Type::Void => write!(f, "void"),
            Type::RefInt(l) => write!(f, "&int{}", l),
            Type::RefFloat(l) => write!(f, "&float{}", l),
            Type::RefBool(l) => write!(f, "&bool{}", l),
            Type::RefString(l) => write!(f, "&string{}", l),
            Type::MutRefInt(l) => write!(f, "&mut int{}", l),
            Type::MutRefFloat(l) => write!(f, "&mut float{}", l),
            Type::MutRefBool(l) => write!(f, "&mut bool{}", l),
            Type::MutRefString(l) => write!(f, "&mut string{}", l),
            Type::List(inner) => write!(f, "List[{}]", inner),
            Type::Optional(inner) => write!(f, "Optional[{}]", inner),
            Type::Result(inner, err) => write!(f, "Result[{}, {}]", inner, err),
            Type::Struct(name, fields) => {
                write!(f, "{} {{", name)?;
                for (i, (field_name, field_type)) in fields.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}: {}", field_name, field_type)?;
                }
                write!(f, "}}")
            },
            Type::Enum(name, _) => write!(f, "{}", name),
            Type::Function(params, ret) => {
                write!(f, "fn(")?;
                for (i, param) in params.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}", param)?;
                }
                write!(f, ") -> {}", ret)
            },
            Type::Generic(name, args) => {
                write!(f, "{}<", name)?;
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}", arg)?;
                }
                write!(f, ">")
            },
            Type::Tuple(types) => {
                write!(f, "(")?;
                for (i, ty) in types.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}", ty)?;
                }
                if types.len() == 1 {
                    write!(f, ",")?;
                }
                write!(f, ")")
            },
            Type::Named(name) => write!(f, "{}", name),
            Type::TypeParam(name) => write!(f, "{}", name),
            Type::Range(inner) => write!(f, "Range[{}]", inner),
            Type::RangeInclusive(inner) => write!(f, "RangeInclusive[{}]", inner),
        }
    }
}

impl Display for TypeModifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TypeModifier::Value => Ok(()),
            TypeModifier::Ref => write!(f, "ref "),
            TypeModifier::MutRef => write!(f, "mut ref "),
        }
    }
}

impl Display for TypeAnnotation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(modifier) = &self.modifier {
            write!(f, "{}", modifier)?;
        }
        write!(f, "{}", self.type_expr)?;
        if let Some(lifetime) = &self.lifetime {
            write!(f, "{}", lifetime)?;
        }
        Ok(())
    }
}

impl Display for EnumVariant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)?;
        if !self.fields.is_empty() {
            write!(f, "(")?;
            for (i, field) in self.fields.iter().enumerate() {
                if i > 0 { write!(f, ", ")?; }
                write!(f, "{}", field)?;
            }
            write!(f, ")")?;
        }
        Ok(())
    }
}

impl Display for Program {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for stmt in &self.statements {
            write!(f, "{stmt}\n")?;
        }
        Ok(())
    }
}