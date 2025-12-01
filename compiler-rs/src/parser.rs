use pest::Parser as PestParser;
use pest_derive::Parser;
use std::collections::HashMap;
use std::convert::From;
use crate::lexer::Token;

// 定义语法错误类型
#[derive(Debug)]
pub enum ParseError {
    PestError(pest::error::Error<Rule>),
    UnexpectedToken(Token),
    SyntaxError(String),
}

impl From<pest::error::Error<Rule>> for ParseError {
    fn from(error: pest::error::Error<Rule>) -> Self {
        ParseError::PestError(error)
    }
}

// 定义AST节点类型
#[derive(Debug)]
pub struct Program {
    pub statements: Vec<Statement>,
    pub module_flags: Vec<String>,
}

#[derive(Debug)]
pub enum Statement {
    Import(ImportStatement),
    Export(ExportStatement),
    VariableDeclaration(VariableDeclaration),
    ConstantDeclaration(ConstantDeclaration),
    StaticDeclaration(StaticDeclaration),
    FunctionDefinition(FunctionDefinition),
    StructDefinition(StructDefinition),
    EnumDefinition(EnumDefinition),
    ClassDefinition(ClassDefinition),
    TraitDefinition(TraitDefinition),
    ImplDefinition(ImplDefinition),
    TypeAlias(TypeAlias),
    ExpressionStatement(Expression),
    IfStatement(IfStatement),
    LoopStatement(LoopStatement),
    ForStatement(ForStatement),
    WhileStatement(WhileStatement),
    MatchStatement(MatchStatement),
    TryCatchStatement(TryCatchStatement),
    ReturnStatement(Option<Expression>),
    BreakStatement,
    ContinueStatement,
    InlineAsmStatement(String),
    UnsafeBlock(Vec<Statement>),
}

#[derive(Debug)]
pub struct ImportStatement {
    pub path: Vec<String>,
    pub alias: Option<String>,
}

#[derive(Debug)]
pub struct ExportStatement {
    pub item: ExportItem,
}

#[derive(Debug)]
pub enum ExportItem {
    Declaration(Box<Statement>),
    ImportPath(Vec<String>),
}

#[derive(Debug)]
pub struct VariableDeclaration {
    pub name: String,
    pub is_mutable: bool,
    pub type_annotation: Option<Type>,
    pub initializer: Option<Expression>,
}

#[derive(Debug)]
pub struct ConstantDeclaration {
    pub name: String,
    pub type_annotation: Option<Type>,
    pub value: Expression,
}

#[derive(Debug)]
pub struct StaticDeclaration {
    pub name: String,
    pub is_mutable: bool,
    pub type_annotation: Option<Type>,
    pub initializer: Expression,
}

#[derive(Debug)]
pub struct FunctionDefinition {
    pub name: String,
    pub parameters: Vec<Parameter>,
    pub return_type: Option<Type>,
    pub body: Vec<Statement>,
}

#[derive(Debug)]
pub struct Parameter {
    pub name: String,
    pub is_mutable: bool,
    pub type_annotation: Type,
}

#[derive(Debug)]
pub struct Block {
    pub statements: Vec<Statement>,
}

#[derive(Debug)]
pub enum Expression {
    Literal(Literal),
    Identifier(String),
    BinaryOp(Box<Expression>, BinaryOperator, Box<Expression>),
    UnaryOp(UnaryOperator, Box<Expression>),
    FunctionCall(Box<Expression>, Vec<Expression>),
    MethodCall(Box<Expression>, String, Vec<Expression>),
    Array(Vec<Expression>),
    Tuple(Vec<Expression>),
    ArrayAccess(Box<Expression>, Box<Expression>),
    StructLiteral(String, HashMap<String, Expression>),
    EnumVariant(String, String),
    Parentheses(Box<Expression>),
    Direction(Box<Expression>, Box<Expression>),
    Await(Box<Expression>),
}

#[derive(Debug)]
pub enum Literal {
    Null,
    Boolean(bool),
    Integer(i64),
    Float(f64),
    String(String),
    Character(char),
}

#[derive(Debug)]
pub enum BinaryOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    Equal,
    NotEqual,
    GreaterThan,
    LessThan,
    GreaterThanOrEqual,
    LessThanOrEqual,
    And,
    Or,
    BitAnd,
    BitOr,
    BitXor,
    LeftShift,
    RightShift,
    Assignment,
    AddAssign,
    SubtractAssign,
    MultiplyAssign,
    DivideAssign,
    ModuloAssign,
    AndAssign,
    OrAssign,
    XorAssign,
    LeftShiftAssign,
    RightShiftAssign,
}

#[derive(Debug)]
pub enum UnaryOperator {
    Not,
    Negate,
    Positive,
}

#[derive(Debug)]
pub enum Type {
    Identifier(String),
    Tuple(Vec<Type>),
    Array(Box<Type>),
    Function(Vec<Type>, Box<Type>),
    Generic(String, Vec<Type>),
}

#[derive(Debug)]
pub struct StructDefinition {
    pub is_public: bool,
    pub name: String,
    pub fields: Vec<FieldDefinition>,
}

#[derive(Debug)]
pub struct FieldDefinition {
    pub is_public: bool,
    pub name: String,
    pub type_annotation: Type,
}

#[derive(Debug)]
pub struct EnumDefinition {
    pub is_public: bool,
    pub name: String,
    pub variants: Vec<EnumVariantDefinition>,
}

#[derive(Debug)]
pub struct EnumVariantDefinition {
    pub is_public: bool,
    pub name: String,
    pub fields: Option<Vec<Type>>,
}

#[derive(Debug)]
pub struct ClassDefinition {
    pub is_public: bool,
    pub name: String,
    pub members: Vec<ClassMember>,
}

#[derive(Debug)]
pub enum ClassMember {
    Field(FieldDefinition),
    Method(MethodDefinition),
    Constructor(ConstructorDefinition),
}

#[derive(Debug)]
pub struct MethodDefinition {
    pub is_public: bool,
    pub is_async: bool,
    pub name: String,
    pub parameters: Vec<Parameter>,
    pub return_type: Option<Type>,
    pub body: Vec<Statement>,
}

#[derive(Debug)]
pub struct ConstructorDefinition {
    pub is_public: bool,
    pub parameters: Vec<Parameter>,
    pub body: Vec<Statement>,
}

#[derive(Debug)]
pub struct TraitDefinition {
    pub is_public: bool,
    pub name: String,
    pub members: Vec<TraitMember>,
}

#[derive(Debug)]
pub enum TraitMember {
    MethodSignature(MethodSignature),
    TypeAlias(TypeAlias),
}

#[derive(Debug)]
pub struct MethodSignature {
    pub is_public: bool,
    pub is_async: bool,
    pub name: String,
    pub parameters: Vec<Parameter>,
    pub return_type: Option<Type>,
}

#[derive(Debug)]
pub struct ImplDefinition {
    pub is_public: bool,
    pub trait_type: Option<Type>,
    pub for_type: Type,
    pub members: Vec<ImplMember>,
}

#[derive(Debug)]
pub enum ImplMember {
    Method(MethodDefinition),
    Constructor(ConstructorDefinition),
    Field(FieldDefinition),
}

#[derive(Debug)]
pub struct TypeAlias {
    pub is_public: bool,
    pub name: String,
    pub type_expr: Type,
}

#[derive(Debug)]
pub struct IfStatement {
    pub condition: Expression,
    pub then_block: Vec<Statement>,
    pub elif_clauses: Vec<(Expression, Vec<Statement>)>,
    pub else_block: Option<Vec<Statement>>,
}

#[derive(Debug)]
pub struct LoopStatement {
    pub body: Vec<Statement>,
}

#[derive(Debug)]
pub struct ForStatement {
    pub is_mutable: bool,
    pub variable_name: String,
    pub iterable: Expression,
    pub body: Vec<Statement>,
}

#[derive(Debug)]
pub struct WhileStatement {
    pub condition: Expression,
    pub body: Vec<Statement>,
}

#[derive(Debug)]
pub struct MatchStatement {
    pub expression: Expression,
    pub cases: Vec<(Expression, Vec<Statement>)>,
    pub default_case: Option<Vec<Statement>>,
}

#[derive(Debug)]
pub struct TryCatchStatement {
    pub try_block: Vec<Statement>,
    pub catch_block: Option<Vec<Statement>>,
}

// 定义Pest语法分析器
#[derive(Parser)]
#[grammar = "grammar.pest"]
pub struct ChimParser;

// 语法分析器实现
pub struct Parser {
    // 可以添加用于缓存或状态跟踪的字段
    diagnostics: Vec<ParseError>,
}

impl Parser {
    pub fn new() -> Self {
        Parser {
            diagnostics: Vec::new(),
        }
    }
    
    // 解析源代码文本
    pub fn parse(&mut self, source_code: &str) -> Result<Program, ParseError> {
        let pairs = ChimParser::parse(Rule::program, source_code)?;
        let mut program = Program {
            statements: Vec::new(),
            module_flags: Vec::new(),
        };
        
        // 解析顶层元素
        for pair in pairs {
            match pair.as_rule() {
                Rule::module_directive => {
                    // 处理模块指令
                    for directive in pair.into_inner() {
                        program.module_flags.push(directive.as_str().to_string());
                    }
                },
                Rule::import_stmt => {
                    // 处理导入语句
                    program.statements.push(Statement::Import(self.parse_import_stmt(pair)?));
                },
                Rule::export_stmt => {
                    // 处理导出语句
                    program.statements.push(Statement::Export(self.parse_export_stmt(pair)?));
                },
                Rule::statement => {
                    // 处理语句
                    program.statements.push(self.parse_statement(pair)?);
                },
                Rule::function_def => {
                    // 处理函数定义
                    program.statements.push(Statement::FunctionDefinition(self.parse_function_def(pair)?));
                },
                _ => {}
            }
        }
        
        Ok(program)
    }
    
    // 解析导入语句
    fn parse_import_stmt(&mut self, pair: pest::iterators::Pair<Rule>) -> Result<ImportStatement, ParseError> {
        let mut inner = pair.into_inner();
        let mut path = Vec::new();
        
        // 解析导入路径
        if let Some(import_path) = inner.find(|p| p.as_rule() == Rule::import_path) {
            for path_part in import_path.into_inner() {
                if path_part.as_rule() == Rule::IDENTIFIER {
                    path.push(path_part.as_str().to_string());
                }
            }
        }
        
        // 解析别名
        let mut alias = None;
        if inner.any(|p| p.as_rule() == Rule::AS) {
            if let Some(ident) = inner.find(|p| p.as_rule() == Rule::IDENTIFIER) {
                alias = Some(ident.as_str().to_string());
            }
        }
        
        Ok(ImportStatement {
            path,
            alias,
        })
    }
    
    // 解析导出语句
    fn parse_export_stmt(&mut self, pair: pest::iterators::Pair<Rule>) -> Result<ExportStatement, ParseError> {
        let mut inner = pair.into_inner();
        
        // 检查是导出声明/函数还是导入
        if inner.clone().any(|p| p.as_rule() == Rule::IMPORT) {
            // 导出导入
            if let Some(import_path) = inner.find(|p| p.as_rule() == Rule::import_path) {
                let mut path = Vec::new();
                for path_part in import_path.into_inner() {
                    if path_part.as_rule() == Rule::IDENTIFIER {
                        path.push(path_part.as_str().to_string());
                    }
                }
                return Ok(ExportStatement {
                    item: ExportItem::ImportPath(path),
                });
            }
        }
        
        // 导出声明或函数 - 简化处理
        for item in inner {
            if item.as_rule() == Rule::function_def {
                return Ok(ExportStatement {
                    item: ExportItem::Declaration(Box::new(Statement::FunctionDefinition(self.parse_function_def(item)?))),
                });
            }
        }
        
        Err(ParseError::SyntaxError("Invalid export statement".to_string()))
    }
    
    // 解析声明
    fn parse_declaration(&mut self, pair: pest::iterators::Pair<Rule>) -> Result<Statement, ParseError> {
        let inner = pair.into_inner().next().ok_or_else(|| ParseError::SyntaxError("Empty statement".to_string()))?;
        
        match inner.as_rule() {
            Rule::variable_decl => Ok(Statement::VariableDeclaration(self.parse_variable_decl(inner)?)),
            Rule::constant_decl => Ok(Statement::ConstantDeclaration(self.parse_constant_decl(inner)?)),
            Rule::static_decl => Ok(Statement::StaticDeclaration(self.parse_static_decl(inner)?)),
            Rule::struct_def => Ok(Statement::StructDefinition(self.parse_struct_def(inner)?)),
            Rule::enum_def => Ok(Statement::EnumDefinition(self.parse_enum_def(inner)?)),
            Rule::class_def => Ok(Statement::ClassDefinition(self.parse_class_def(inner)?)),
            Rule::trait_def => Ok(Statement::TraitDefinition(self.parse_trait_def(inner)?)),
            Rule::impl_def => Ok(Statement::ImplDefinition(self.parse_impl_def(inner)?)),
            Rule::type_alias => Ok(Statement::TypeAlias(self.parse_type_alias(inner)?)),
            _ => Err(ParseError::SyntaxError(format!("Unknown statement type: {:?}", inner.as_rule()))),
        }
    }
    
    // 解析变量声明
    fn parse_variable_decl(&mut self, pair: pest::iterators::Pair<Rule>) -> Result<VariableDeclaration, ParseError> {
        let mut inner = pair.into_inner();
        let mut name = String::new();
        let mut is_mutable = false;
        let mut type_annotation = None;
        let mut initializer = None;
        
        // 解析let/mut关键字
        while let Some(p) = inner.find(|p| p.as_rule() == Rule::LET || p.as_rule() == Rule::MUT) {
            if p.as_rule() == Rule::MUT {
                is_mutable = true;
            }
        }
        
        // 解析标识符
        if let Some(ident) = inner.find(|p| p.as_rule() == Rule::IDENTIFIER) {
            name = ident.as_str().to_string();
        }
        
        // 解析类型注解
        if inner.any(|p| p.as_rule() == Rule::COLON) {
            if let Some(type_pair) = inner.find(|p| p.as_rule() == Rule::type_expr) {
                type_annotation = Some(self.parse_type_expr(type_pair)?);
            }
        }
        
        // 解析初始化表达式
        if inner.any(|p| p.as_rule() == Rule::ASSIGN) {
            if let Some(expr_pair) = inner.find(|p| p.as_rule() == Rule::expression) {
                initializer = Some(self.parse_expression(expr_pair)?);
            }
        }
        
        Ok(VariableDeclaration {
            name,
            is_mutable,
            type_annotation,
            initializer,
        })
    }
    
    // 解析常量声明
    fn parse_constant_decl(&mut self, pair: pest::iterators::Pair<Rule>) -> Result<ConstantDeclaration, ParseError> {
        let mut inner = pair.into_inner();
        let mut name = String::new();
        let mut type_annotation = None;
        let mut value = None;
        
        // 解析标识符
        if let Some(ident) = inner.find(|p| p.as_rule() == Rule::IDENTIFIER) {
            name = ident.as_str().to_string();
        }
        
        // 解析类型注解
        if inner.any(|p| p.as_rule() == Rule::COLON) {
            if let Some(type_pair) = inner.find(|p| p.as_rule() == Rule::type_expr) {
                type_annotation = Some(self.parse_type_expr(type_pair)?);
            }
        }
        
        // 解析表达式
        if let Some(expr_pair) = inner.find(|p| p.as_rule() == Rule::expression) {
            value = Some(self.parse_expression(expr_pair)?);
        }
        
        Ok(ConstantDeclaration {
            name,
            type_annotation,
            value: value.unwrap(),
        })
    }
    
    // 解析静态变量声明
    fn parse_static_decl(&mut self, pair: pest::iterators::Pair<Rule>) -> Result<StaticDeclaration, ParseError> {
        let mut inner = pair.into_inner();
        let mut name = String::new();
        let mut is_mutable = false;
        let mut type_annotation = None;
        let mut initializer = None;
        
        // 检查mut关键字
        if inner.any(|p| p.as_rule() == Rule::MUT) {
            is_mutable = true;
        }
        
        // 解析标识符
        if let Some(ident) = inner.find(|p| p.as_rule() == Rule::IDENTIFIER) {
            name = ident.as_str().to_string();
        }
        
        // 解析类型注解
        if inner.any(|p| p.as_rule() == Rule::COLON) {
            if let Some(type_pair) = inner.find(|p| p.as_rule() == Rule::type_expr) {
                type_annotation = Some(self.parse_type_expr(type_pair)?);
            }
        }
        
        // 解析初始化表达式
        if let Some(expr_pair) = inner.find(|p| p.as_rule() == Rule::expression) {
            initializer = Some(self.parse_expression(expr_pair)?);
        }
        
        Ok(StaticDeclaration {
            name,
            is_mutable,
            type_annotation,
            initializer: initializer.unwrap(),
        })
    }
    
    // 解析函数定义
    fn parse_function_def(&mut self, pair: pest::iterators::Pair<Rule>) -> Result<FunctionDefinition, ParseError> {
        let mut inner = pair.into_inner();
        let mut name = String::new();
        let mut parameters = Vec::new();
        let mut return_type = None;
        let mut body = Vec::new();
        
        // 解析函数名
        if let Some(ident) = inner.find(|p| p.as_rule() == Rule::IDENTIFIER) {
            name = ident.as_str().to_string();
        }
        
        // 解析参数列表
        if let Some(param_pair) = inner.find(|p| p.as_rule() == Rule::parameter) {
            for param in param_pair.into_inner() {
                parameters.push(self.parse_parameter(param)?);
            }
        }
        
        // 解析返回类型
        if inner.any(|p| p.as_rule() == Rule::ARROW) {
            if let Some(type_pair) = inner.find(|p| p.as_rule() == Rule::type_expr) {
                return_type = Some(self.parse_type_expr(type_pair)?);
            }
        }
        
        // 解析函数体
        if let Some(block) = inner.find(|p| p.as_rule() == Rule::block) {
            body = self.parse_block(block)?;
        }
        
        Ok(FunctionDefinition {
            name,
            parameters,
            return_type,
            body,
        })
    }
    
    // 解析参数
    fn parse_parameter(&mut self, pair: pest::iterators::Pair<Rule>) -> Result<Parameter, ParseError> {
        let mut inner = pair.into_inner();
        let mut name = String::new();
        let mut is_mutable = false;
        let mut type_annotation = None;
        
        // 检查mut关键字
        if inner.any(|p| p.as_rule() == Rule::MUT) {
            is_mutable = true;
        }
        
        // 解析标识符
        if let Some(ident) = inner.find(|p| p.as_rule() == Rule::IDENTIFIER) {
            name = ident.as_str().to_string();
        }
        
        // 解析类型
        if let Some(type_pair) = inner.find(|p| p.as_rule() == Rule::type_expr) {
            type_annotation = Some(self.parse_type_expr(type_pair)?);
        }
        
        Ok(Parameter {
            name,
            is_mutable,
            type_annotation: type_annotation.unwrap(),
        })
    }
    
    // 解析代码块
    fn parse_block(&mut self, pair: pest::iterators::Pair<Rule>) -> Result<Vec<Statement>, ParseError> {
        let mut statements = Vec::new();
        
        for statement in pair.into_inner().filter(|p| p.as_rule() == Rule::statement) {
            statements.push(self.parse_statement(statement)?);
        }
        
        Ok(statements)
    }
    
    // 解析语句
    fn parse_statement(&mut self, pair: pest::iterators::Pair<Rule>) -> Result<Statement, ParseError> {
        let inner = pair.into_inner().next().ok_or_else(|| ParseError::SyntaxError("Empty statement".to_string()))?;
        
        match inner.as_rule() {
            Rule::expression_statement => {
                if let Some(expr_pair) = inner.into_inner().find(|p| p.as_rule() == Rule::expression) {
                    Ok(Statement::ExpressionStatement(self.parse_expression(expr_pair)?))
                } else {
                    Err(ParseError::SyntaxError("Empty expression statement".to_string()))
                }
            },
            Rule::variable_decl => Ok(Statement::VariableDeclaration(self.parse_variable_decl(inner)?)),
            Rule::control_flow => self.parse_control_flow(inner),
            Rule::function_def => Ok(Statement::FunctionDefinition(self.parse_function_def(inner)?)),
            Rule::return_stmt => {
                let mut inner = inner.into_inner();
                let expression = if let Some(expr_pair) = inner.find(|p| p.as_rule() == Rule::expression) {
                    Some(self.parse_expression(expr_pair)?)
                } else {
                    None
                };
                Ok(Statement::ReturnStatement(expression))
            },
            Rule::break_stmt => Ok(Statement::BreakStatement),
            Rule::continue_stmt => Ok(Statement::ContinueStatement),
            Rule::inline_asm_stmt => {
                // 解析内联汇编语句
                if let Some(str_pair) = inner.into_inner().find(|p| p.as_rule() == Rule::STRING) {
                    // 移除引号
                    let asm = str_pair.as_str()[1..str_pair.as_str().len()-1].to_string();
                    Ok(Statement::InlineAsmStatement(asm))
                } else {
                    Err(ParseError::SyntaxError("Invalid inline assembly statement".to_string()))
                }
            },
            Rule::unsafe_block => {
                // 解析unsafe块
                if let Some(block) = inner.into_inner().find(|p| p.as_rule() == Rule::block) {
                    Ok(Statement::UnsafeBlock(self.parse_block(block)?))
                } else {
                    Err(ParseError::SyntaxError("Invalid unsafe block".to_string()))
                }
            },
            _ => Err(ParseError::SyntaxError(format!("Unknown statement type: {:?}", inner.as_rule()))),
        }
    }
    
    // 解析控制流语句
    fn parse_control_flow(&mut self, pair: pest::iterators::Pair<Rule>) -> Result<Statement, ParseError> {
        let inner = pair.into_inner().next().ok_or_else(|| ParseError::SyntaxError("Empty control flow".to_string()))?;
        
        match inner.as_rule() {
            Rule::if_stmt => Ok(Statement::IfStatement(self.parse_if_stmt(inner)?)),
            Rule::loop_stmt => Ok(Statement::LoopStatement(self.parse_loop_stmt(inner)?)),
            Rule::for_stmt => Ok(Statement::ForStatement(self.parse_for_stmt(inner)?)),
            Rule::while_stmt => Ok(Statement::WhileStatement(self.parse_while_stmt(inner)?)),
            Rule::match_stmt => Ok(Statement::MatchStatement(self.parse_match_stmt(inner)?)),
            Rule::try_catch_stmt => Ok(Statement::TryCatchStatement(self.parse_try_catch_stmt(inner)?)),
            _ => Err(ParseError::SyntaxError(format!("Unknown control flow type: {:?}", inner.as_rule()))),
        }
    }
    
    // 解析类型表达式
    fn parse_type_expr(&mut self, pair: pest::iterators::Pair<Rule>) -> Result<Type, ParseError> {
        let mut inner = pair.into_inner();
        
        // 检查类型表达式类型
        if let Some(tuple_type) = inner.find(|p| p.as_rule() == Rule::tuple_type) {
            return self.parse_tuple_type(tuple_type);
        } else if let Some(array_type) = inner.find(|p| p.as_rule() == Rule::array_type) {
            return self.parse_array_type(array_type);
        } else if let Some(function_type) = inner.find(|p| p.as_rule() == Rule::function_type) {
            return self.parse_function_type(function_type);
        } else if let Some(generic_type) = inner.find(|p| p.as_rule() == Rule::generic_type) {
            return self.parse_generic_type(generic_type);
        } else if let Some(ident) = inner.find(|p| p.as_rule() == Rule::IDENTIFIER) {
            // 简单标识符类型
            return Ok(Type::Identifier(ident.as_str().to_string()));
        }
        
        Err(ParseError::SyntaxError("Invalid type expression".to_string()))
    }
    
    // 解析元组类型
    fn parse_tuple_type(&mut self, pair: pest::iterators::Pair<Rule>) -> Result<Type, ParseError> {
        let mut types = Vec::new();
        
        for type_pair in pair.into_inner().filter(|p| p.as_rule() == Rule::type_expr) {
            types.push(self.parse_type_expr(type_pair)?);
        }
        
        Ok(Type::Tuple(types))
    }
    
    // 解析数组类型
    fn parse_array_type(&mut self, pair: pest::iterators::Pair<Rule>) -> Result<Type, ParseError> {
        if let Some(type_pair) = pair.into_inner().find(|p| p.as_rule() == Rule::type_expr) {
            Ok(Type::Array(Box::new(self.parse_type_expr(type_pair)?)))
        } else {
            Err(ParseError::SyntaxError("Invalid array type".to_string()))
        }
    }
    
    // 解析函数类型
    fn parse_function_type(&mut self, pair: pest::iterators::Pair<Rule>) -> Result<Type, ParseError> {
        let mut inner = pair.into_inner();
        let mut param_types = Vec::new();
        let mut return_type = None;
        
        // 解析参数类型
        if let Some(param_pair) = inner.find(|p| p.as_rule() == Rule::type_expr) {
            param_types.push(self.parse_type_expr(param_pair)?);
        }
        
        // 解析返回类型
        if let Some(ret_pair) = inner.find(|p| p.as_rule() == Rule::type_expr) {
            return_type = Some(self.parse_type_expr(ret_pair)?);
        }
        
        Ok(Type::Function(
            param_types,
            Box::new(return_type.unwrap())
        ))
    }
    
    // 解析泛型类型
    fn parse_generic_type(&mut self, pair: pest::iterators::Pair<Rule>) -> Result<Type, ParseError> {
        let mut inner = pair.into_inner();
        let mut name = String::new();
        let mut type_args = Vec::new();
        
        // 解析类型名称
        if let Some(ident) = inner.find(|p| p.as_rule() == Rule::IDENTIFIER) {
            name = ident.as_str().to_string();
        }
        
        // 解析类型参数
        for type_pair in inner.filter(|p| p.as_rule() == Rule::type_expr) {
            type_args.push(self.parse_type_expr(type_pair)?);
        }
        
        Ok(Type::Generic(name, type_args))
    }
    
    // 解析表达式（简化版实现，实际需要处理运算符优先级）
    fn parse_expression(&mut self, pair: pest::iterators::Pair<Rule>) -> Result<Expression, ParseError> {
        // 这个函数会根据规则层级逐步解析表达式
        // 这里只提供一个简化的实现
        let mut inner = pair.into_inner();
        
        // 检查表达式类型
        if let Some(assign_pair) = inner.find(|p| p.as_rule() == Rule::assignment) {
            return self.parse_assignment(assign_pair);
        } else if let Some(or_pair) = inner.find(|p| p.as_rule() == Rule::logical_or) {
            return self.parse_logical_or(or_pair);
        } else if let Some(and_pair) = inner.find(|p| p.as_rule() == Rule::logical_and) {
            return self.parse_logical_and(and_pair);
        } else if let Some(comp_pair) = inner.find(|p| p.as_rule() == Rule::comparison) {
            return self.parse_comparison(comp_pair);
        } else if let Some(term_pair) = inner.find(|p| p.as_rule() == Rule::term) {
            return self.parse_term(term_pair);
        } else if let Some(factor_pair) = inner.find(|p| p.as_rule() == Rule::factor) {
            return self.parse_factor(factor_pair);
        } else if let Some(unary_pair) = inner.find(|p| p.as_rule() == Rule::unary) {
            return self.parse_unary(unary_pair);
        } else if let Some(primary_pair) = inner.find(|p| p.as_rule() == Rule::primary) {
            return self.parse_primary(primary_pair);
        }
        
        Err(ParseError::SyntaxError("Invalid expression".to_string()))
    }
    
    // 以下是各种表达式解析函数的简化实现
    fn parse_assignment(&mut self, pair: pest::iterators::Pair<Rule>) -> Result<Expression, ParseError> {
        // 简化实现，实际需要处理各种赋值运算符
        let mut inner = pair.into_inner();
        let left = self.parse_logical_or(inner.next().ok_or_else(|| ParseError::SyntaxError("Missing left operand in assignment".to_string()))?)?;
        
        // 找到赋值运算符
        let mut op = BinaryOperator::Assignment;
        if let Some(op_pair) = inner.find(|p| {
            p.as_rule() == Rule::ASSIGN ||
            p.as_rule() == Rule::ADD_ASSIGN ||
            p.as_rule() == Rule::SUBTRACT_ASSIGN
        }) {
            op = match op_pair.as_rule() {
                Rule::ASSIGN => BinaryOperator::Assignment,
                Rule::ADD_ASSIGN => BinaryOperator::AddAssign,
                Rule::SUBTRACT_ASSIGN => BinaryOperator::SubtractAssign,
                _ => BinaryOperator::Assignment,
            };
        }
        
        // 解析右侧表达式
        let right = self.parse_logical_or(inner.next().ok_or_else(|| ParseError::SyntaxError("Missing right operand in assignment".to_string()))?)?;
        
        Ok(Expression::BinaryOp(Box::new(left), op, Box::new(right)))
    }
    
    fn parse_logical_or(&mut self, pair: pest::iterators::Pair<Rule>) -> Result<Expression, ParseError> {
        let mut inner = pair.into_inner();
        let mut expr = self.parse_logical_and(inner.next().ok_or_else(|| ParseError::SyntaxError("Missing operand in OR expression".to_string()))?)?;
        
        // 处理多个OR操作符
        while let Some(or_pair) = inner.find(|p| p.as_rule() == Rule::OR) {
            let right = self.parse_logical_and(inner.next().ok_or_else(|| ParseError::SyntaxError("Missing right operand in OR expression".to_string()))?)?;
            expr = Expression::BinaryOp(Box::new(expr), BinaryOperator::Or, Box::new(right));
        }
        
        Ok(expr)
    }
    
    fn parse_logical_and(&mut self, pair: pest::iterators::Pair<Rule>) -> Result<Expression, ParseError> {
        let mut inner = pair.into_inner();
        let mut expr = self.parse_comparison(inner.next().ok_or_else(|| ParseError::SyntaxError("Missing operand in AND expression".to_string()))?)?;
        
        // 处理多个AND操作符
        while let Some(and_pair) = inner.find(|p| p.as_rule() == Rule::AND) {
            let right = self.parse_comparison(inner.next().ok_or_else(|| ParseError::SyntaxError("Missing right operand in AND expression".to_string()))?)?;
            expr = Expression::BinaryOp(Box::new(expr), BinaryOperator::And, Box::new(right));
        }
        
        Ok(expr)
    }
    
    fn parse_comparison(&mut self, pair: pest::iterators::Pair<Rule>) -> Result<Expression, ParseError> {
        let mut inner = pair.into_inner();
        let mut expr = self.parse_term(inner.next().ok_or_else(|| ParseError::SyntaxError("Missing operand in comparison".to_string()))?)?;
        
        // 处理比较运算符
        while let Some(op_pair) = inner.find(|p| {
            p.as_rule() == Rule::EQUAL ||
            p.as_rule() == Rule::NOT_EQUAL ||
            p.as_rule() == Rule::GREATER ||
            p.as_rule() == Rule::LESS ||
            p.as_rule() == Rule::GREATER_OR_EQUAL ||
            p.as_rule() == Rule::LESS_OR_EQUAL
        }) {
            let op = match op_pair.as_rule() {
                Rule::EQUAL => BinaryOperator::Equal,
                Rule::NOT_EQUAL => BinaryOperator::NotEqual,
                Rule::GREATER => BinaryOperator::GreaterThan,
                Rule::LESS => BinaryOperator::LessThan,
                Rule::GREATER_OR_EQUAL => BinaryOperator::GreaterThanOrEqual,
                Rule::LESS_OR_EQUAL => BinaryOperator::LessThanOrEqual,
                _ => unreachable!(),
            };
            let right = self.parse_term(inner.next().ok_or_else(|| ParseError::SyntaxError("Missing right operand in comparison".to_string()))?)?;
            expr = Expression::BinaryOp(Box::new(expr), op, Box::new(right));
        }
        
        Ok(expr)
    }
    
    // 这里只实现部分必要的解析函数，完整实现需要更多代码
    
    fn parse_term(&mut self, pair: pest::iterators::Pair<Rule>) -> Result<Expression, ParseError> {
        // 解析加减法
        let mut inner = pair.into_inner();
        let mut expr = self.parse_factor(inner.next().ok_or_else(|| ParseError::SyntaxError("Missing operand in term".to_string()))?)?;
        
        while let Some(op_pair) = inner.find(|p| p.as_rule() == Rule::PLUS || p.as_rule() == Rule::MINUS) {
            let op = match op_pair.as_rule() {
                Rule::PLUS => BinaryOperator::Add,
                Rule::MINUS => BinaryOperator::Subtract,
                _ => unreachable!(),
            };
            let right = self.parse_factor(inner.next().ok_or_else(|| ParseError::SyntaxError("Missing right operand in term".to_string()))?)?;
            expr = Expression::BinaryOp(Box::new(expr), op, Box::new(right));
        }
        
        Ok(expr)
    }
    
    fn parse_factor(&mut self, pair: pest::iterators::Pair<Rule>) -> Result<Expression, ParseError> {
        // 解析乘除法等
        let mut inner = pair.into_inner();
        let mut expr = self.parse_unary(inner.next().ok_or_else(|| ParseError::SyntaxError("Missing operand in factor".to_string()))?)?;
        
        while let Some(op_pair) = inner.find(|p| {
            p.as_rule() == Rule::MULTIPLY ||
            p.as_rule() == Rule::DIVIDE ||
            p.as_rule() == Rule::MODULO
        }) {
            let op = match op_pair.as_rule() {
                Rule::MULTIPLY => BinaryOperator::Multiply,
                Rule::DIVIDE => BinaryOperator::Divide,
                Rule::MODULO => BinaryOperator::Modulo,
                _ => unreachable!(),
            };
            let right = self.parse_unary(inner.next().ok_or_else(|| ParseError::SyntaxError("Missing right operand in factor".to_string()))?)?;
            expr = Expression::BinaryOp(Box::new(expr), op, Box::new(right));
        }
        
        Ok(expr)
    }
    
    fn parse_unary(&mut self, pair: pest::iterators::Pair<Rule>) -> Result<Expression, ParseError> {
        let mut inner = pair.into_inner();
        
        // 检查是否有一元操作符
        if let Some(op_pair) = inner.find(|p| p.as_rule() == Rule::NOT || p.as_rule() == Rule::MINUS || p.as_rule() == Rule::PLUS) {
            let op = match op_pair.as_rule() {
                Rule::NOT => UnaryOperator::Not,
                Rule::MINUS => UnaryOperator::Negate,
                Rule::PLUS => UnaryOperator::Positive,
                _ => unreachable!(),
            };
            let expr = self.parse_unary(inner.next().ok_or_else(|| ParseError::SyntaxError("Missing operand for unary operator".to_string()))?)?;
            Ok(Expression::UnaryOp(op, Box::new(expr)))
        } else {
            // 没有一元操作符，直接解析primary
            if let Some(primary_pair) = inner.find(|p| p.as_rule() == Rule::primary) {
                self.parse_primary(primary_pair)
            } else {
                Err(ParseError::SyntaxError("Invalid unary expression".to_string()))
            }
        }
    }
    
    fn parse_primary(&mut self, pair: pest::iterators::Pair<Rule>) -> Result<Expression, ParseError> {
        // 解析基本表达式
        for expr in pair.into_inner() {
            match expr.as_rule() {
                Rule::literal => {
                    return self.parse_literal(expr);
                },
                Rule::identifier_expr => {
                    // 解析标识符表达式
                    if let Some(ident) = expr.into_inner().find(|p| p.as_rule() == Rule::IDENTIFIER) {
                        return Ok(Expression::Identifier(ident.as_str().to_string()));
                    }
                },
                Rule::function_call => {
                    // 解析函数调用
                    return self.parse_function_call(expr);
                },
                Rule::expression => {
                    // 括号中的表达式
                    return Ok(Expression::Parentheses(Box::new(self.parse_expression(expr)?)));
                },
                _ => {
                    // 可以根据需要实现其他primary类型的解析
                    // 如数组表达式、结构体表达式等
                }
            }
        }
        
        Err(ParseError::SyntaxError("Invalid primary expression".to_string()))
    }
    
    fn parse_literal(&mut self, pair: pest::iterators::Pair<Rule>) -> Result<Expression, ParseError> {
        // 解析字面量
        for lit in pair.into_inner() {
            match lit.as_rule() {
                Rule::NULL => return Ok(Expression::Literal(Literal::Null)),
                Rule::TRUE => return Ok(Expression::Literal(Literal::Boolean(true))),
                Rule::FALSE => return Ok(Expression::Literal(Literal::Boolean(false))),
                Rule::INTEGER => {
                    if let Ok(value) = lit.as_str().parse::<i64>() {
                        return Ok(Expression::Literal(Literal::Integer(value)));
                    }
                },
                Rule::FLOAT => {
                    if let Ok(value) = lit.as_str().parse::<f64>() {
                        return Ok(Expression::Literal(Literal::Float(value)));
                    }
                },
                Rule::STRING => {
                    // 移除引号
                    let s = lit.as_str();
                    if s.starts_with('"') && s.ends_with('"') {
                        return Ok(Expression::Literal(Literal::String(s[1..s.len()-1].to_string())));
                    }
                },
                Rule::CHAR => {
                    // 移除引号
                    let s = lit.as_str();
                    if s.starts_with('\'') && s.ends_with('\'') && s.len() == 3 {
                        return Ok(Expression::Literal(Literal::Character(s.chars().nth(1).unwrap())));
                    }
                },
                _ => {}
            }
        }
        
        Err(ParseError::SyntaxError("Invalid literal".to_string()))
    }
    
    fn parse_function_call(&mut self, pair: pest::iterators::Pair<Rule>) -> Result<Expression, ParseError> {
        // 解析函数调用
        let mut inner = pair.into_inner();
        let mut callee = None;
        let mut args = Vec::new();
        
        // 解析函数名或表达式
        if let Some(ident_pair) = inner.find(|p| p.as_rule() == Rule::identifier_expr || p.as_rule() == Rule::IDENTIFIER) {
            let ident_text = if ident_pair.as_rule() == Rule::IDENTIFIER {
                ident_pair.as_str().to_string()
            } else {
                ident_pair.into_inner().find(|p| p.as_rule() == Rule::IDENTIFIER)
                    .map(|p| p.as_str().to_string())
                    .unwrap_or_default()
            };
            callee = Some(Expression::Identifier(ident_text));
        }
        
        // 解析参数
        for arg_pair in inner.filter(|p| p.as_rule() == Rule::expression) {
            args.push(self.parse_expression(arg_pair)?);
        }
        
        if let Some(callee_expr) = callee {
            Ok(Expression::FunctionCall(Box::new(callee_expr), args))
        } else {
            Err(ParseError::SyntaxError("Invalid function call: missing callee".to_string()))
        }
    }
    
    // 以下是其他需要实现的解析函数的占位符
    fn parse_if_stmt(&mut self, pair: pest::iterators::Pair<Rule>) -> Result<IfStatement, ParseError> {
        // 解析if语句
        Err(ParseError::SyntaxError("If statement parsing not implemented".to_string()))
    }
    
    fn parse_loop_stmt(&mut self, pair: pest::iterators::Pair<Rule>) -> Result<LoopStatement, ParseError> {
        // 解析loop语句
        Err(ParseError::SyntaxError("Loop statement parsing not implemented".to_string()))
    }
    
    fn parse_for_stmt(&mut self, pair: pest::iterators::Pair<Rule>) -> Result<ForStatement, ParseError> {
        // 解析for语句
        Err(ParseError::SyntaxError("For statement parsing not implemented".to_string()))
    }
    
    fn parse_while_stmt(&mut self, pair: pest::iterators::Pair<Rule>) -> Result<WhileStatement, ParseError> {
        // 解析while语句
        Err(ParseError::SyntaxError("While statement parsing not implemented".to_string()))
    }
    
    fn parse_match_stmt(&mut self, pair: pest::iterators::Pair<Rule>) -> Result<MatchStatement, ParseError> {
        // 解析match语句
        Err(ParseError::SyntaxError("Match statement parsing not implemented".to_string()))
    }
    
    fn parse_try_catch_stmt(&mut self, pair: pest::iterators::Pair<Rule>) -> Result<TryCatchStatement, ParseError> {
        // 解析try-catch语句
        Err(ParseError::SyntaxError("Try-catch statement parsing not implemented".to_string()))
    }
    
    // 结构体、枚举等定义的解析函数
    fn parse_struct_def(&mut self, pair: pest::iterators::Pair<Rule>) -> Result<StructDefinition, ParseError> {
        // 解析结构体定义
        Err(ParseError::SyntaxError("Struct definition parsing not implemented".to_string()))
    }
    
    fn parse_enum_def(&mut self, pair: pest::iterators::Pair<Rule>) -> Result<EnumDefinition, ParseError> {
        // 解析枚举定义
        Err(ParseError::SyntaxError("Enum definition parsing not implemented".to_string()))
    }
    
    fn parse_class_def(&mut self, pair: pest::iterators::Pair<Rule>) -> Result<ClassDefinition, ParseError> {
        // 解析类定义
        Err(ParseError::SyntaxError("Class definition parsing not implemented".to_string()))
    }
    
    fn parse_trait_def(&mut self, pair: pest::iterators::Pair<Rule>) -> Result<TraitDefinition, ParseError> {
        // 解析特性定义
        Err(ParseError::SyntaxError("Trait definition parsing not implemented".to_string()))
    }
    
    fn parse_impl_def(&mut self, pair: pest::iterators::Pair<Rule>) -> Result<ImplDefinition, ParseError> {
        // 解析实现定义
        Err(ParseError::SyntaxError("Impl definition parsing not implemented".to_string()))
    }
    
    fn parse_type_alias(&mut self, pair: pest::iterators::Pair<Rule>) -> Result<TypeAlias, ParseError> {
        // 解析类型别名
        Err(ParseError::SyntaxError("Type alias parsing not implemented".to_string()))
    }
}

// 导出供FFI使用的API
#[no_mangle]
pub extern "C" fn create_parser() -> *mut Parser {
    let parser = Box::new(Parser::new());
    Box::into_raw(parser)
}

#[no_mangle]
pub extern "C" fn parse_program(parser: *mut Parser, source_code: *const c_char) -> *mut Program {
    let parser = unsafe { &mut *parser };
    let source_code_str = unsafe { CStr::from_ptr(source_code).to_str().unwrap_or("") };
    
    match parser.parse(source_code_str) {
        Ok(program) => {
            let program_box = Box::new(program);
            Box::into_raw(program_box)
        },
        Err(_) => std::ptr::null_mut(),
    }
}

#[no_mangle]
pub extern "C" fn free_parser(parser: *mut Parser) {
    if !parser.is_null() {
        unsafe { Box::from_raw(parser) };
    }
}

#[no_mangle]
pub extern "C" fn free_program(program: *mut Program) {
    if !program.is_null() {
        unsafe { Box::from_raw(program) };
    }
}

// 需要导入C相关的库
use std::ffi::{CStr, c_char};
