use rayon::prelude::*;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use crate::{Compiler, Function, Type, NodeId};

#[derive(Debug, Clone)]
pub struct ParallelCompileOptions {
    pub num_jobs: usize,
    pub parallel_lexing: bool,
    pub parallel_parsing: bool,
    pub parallel_typecheck: bool,
    pub parallel_optimization: bool,
    pub parallel_codegen: bool,
}

impl Default for ParallelCompileOptions {
    fn default() -> Self {
        Self {
            num_jobs: rayon::current_num_threads(),
            parallel_lexing: true,
            parallel_parsing: true,
            parallel_typecheck: true,
            parallel_optimization: true,
            parallel_codegen: true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CompileResult<T> {
    pub data: T,
    pub warnings: Vec<CompileWarning>,
    pub duration_ms: f64,
}

#[derive(Debug, Clone)]
pub struct CompileWarning {
    pub message: String,
    pub location: (String, u32, u32),
}

struct SharedCompilerState {
    types: Vec<Type>,
    functions: Vec<Function>,
    errors: Mutex<Vec<CompileError>>,
    warnings: Mutex<Vec<CompileWarning>>,
}

#[derive(Debug, Clone)]
pub struct CompileError {
    pub message: String,
    pub node_id: NodeId,
    pub severity: ErrorSeverity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorSeverity {
    Error,
    Fatal,
    Warning,
    Note,
}

impl SharedCompilerState {
    fn new() -> Self {
        Self {
            types: Vec::new(),
            functions: Vec::new(),
            errors: Mutex::new(Vec::new()),
            warnings: Mutex::new(Vec::new()),
        }
    }

    fn add_error(&self, error: CompileError) {
        let mut errors = self.errors.lock().unwrap();
        errors.push(error);
    }

    fn add_warning(&self, warning: CompileWarning) {
        let mut warnings = self.warnings.lock().unwrap();
        warnings.push(warning);
    }

    fn get_errors(&self) -> Vec<CompileError> {
        self.errors.lock().unwrap().clone()
    }

    fn get_warnings(&self) -> Vec<CompileWarning> {
        self.warnings.lock().unwrap().clone()
    }
}

pub fn parallel_compile(
    sources: &[String],
    options: &ParallelCompileOptions,
) -> Result<CompileResult<Compiler>, Vec<CompileError>> {
    let start_time = std::time::Instant::now();
    let state = Arc::new(SharedCompilerState::new());
    let jobs = if options.num_jobs > 0 { options.num_jobs } else { rayon::current_num_threads() };
    
    rayon::ThreadPoolBuilder::new()
        .num_threads(jobs)
        .build_global()
        .unwrap();

    let parsed_modules = if options.parallel_parsing && sources.len() > 1 {
        sources.par_iter()
            .map(|source| parse_source(source))
            .collect::<Vec<_>>()
    } else {
        sources.iter()
            .map(|source| parse_source(source))
            .collect()
    };

    let typechecked_modules = if options.parallel_typecheck && parsed_modules.len() > 1 {
        parsed_modules.par_iter()
            .map(|module| typecheck_module(module, &state))
            .collect::<Vec<_>>()
    } else {
        parsed_modules.iter()
            .map(|module| typecheck_module(module, &state))
            .collect()
    };

    let optimized_modules = if options.parallel_optimization && typechecked_modules.len() > 1 {
        typechecked_modules.par_iter()
            .map(|module| optimize_module(module, &state))
            .collect::<Vec<_>>()
    } else {
        typechecked_modules.iter()
            .map(|module| optimize_module(module, &state))
            .collect()
    };

    let mut compiler = Compiler::new();
    merge_modules(&mut compiler, &optimized_modules);

    let errors = state.get_errors();
    if !errors.is_empty() {
        return Err(errors);
    }

    let duration_ms = start_time.elapsed().as_secs_f64() * 1000.0;
    let warnings = state.get_warnings();

    Ok(CompileResult {
        data: compiler,
        warnings,
        duration_ms,
    })
}

fn parse_source(source: &str) -> ParsedModule {
    let lexer = Lexer::new(source);
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(&tokens);
    let ast = parser.parse().unwrap();
    ParsedModule {
        source: source.to_string(),
        ast,
        tokens,
    }
}

fn typecheck_module(module: &ParsedModule, state: &Arc<SharedCompilerState>) -> TypecheckedModule {
    let mut typechecker = Typechecker::new(state);
    let types = typechecker.check(&module.ast);
    TypecheckedModule {
        source: module.source.clone(),
        ast: module.ast.clone(),
        types: types.clone(),
        hir: HIR::from_ast(&module.ast, &types),
    }
}

fn optimize_module(module: &TypecheckedModule, state: &Arc<SharedCompilerState>) -> OptimizedModule {
    let mut optimizer = Optimizer::new(state);
    let optimized_ir = optimizer.optimize(&module.hir);
    OptimizedModule {
        source: module.source.clone(),
        ast: module.ast.clone(),
        types: module.types.clone(),
        hir: module.hir.clone(),
        ir: optimized_ir,
    }
}

fn merge_modules(compiler: &mut Compiler, modules: &[OptimizedModule]) {
    for module in modules {
        for fun in &module.ir.functions {
            compiler.functions.push(fun.clone());
        }
        for ty in &module.types {
            compiler.types.push(ty.clone());
        }
    }
}

pub struct Lexer<'a> {
    source: &'a str,
    position: usize,
}

impl<'a> Lexer<'a> {
    fn new(source: &'a str) -> Self {
        Self {
            source,
            position: 0,
        }
    }

    fn tokenize(&self) -> Result<Vec<Token>, LexError> {
        let mut tokens = Vec::new();
        let mut chars = self.source.chars().peekable();
        while let Some(c) = chars.next() {
            if c.is_whitespace() {
                continue;
            }
            if c.is_alphabetic() || c == '_' {
                let ident = self.read_identifier(c, &mut chars);
                if self.is_keyword(&ident) {
                    tokens.push(Token::Keyword(ident));
                } else {
                    tokens.push(Token::Identifier(ident));
                }
            } else if c.is_digit(c) {
                let number = self.read_number(c, &mut chars);
                tokens.push(Token::Number(number));
            } else {
                let op = self.read_operator(c, &mut chars);
                tokens.push(Token::Operator(op));
            }
        }
        Ok(tokens)
    }

    fn read_identifier<C: Iterator<Item = char>>(&self, first: char, chars: &mut C) -> String {
        let mut ident = String::new();
        ident.push(first);
        while let Some(&c) = chars.peek() {
            if c.is_alphanumeric() || c == '_' {
                ident.push(c);
                chars.next();
            } else {
                break;
            }
        }
        ident
    }

    fn read_number<C: Iterator<Item = char>>(&self, first: char, chars: &mut C) -> String {
        let mut number = String::new();
        number.push(first);
        while let Some(&c) = chars.peek() {
            if c.is_digit(c) || c == '.' || c == 'e' || c == 'E' {
                number.push(c);
                chars.next();
            } else {
                break;
            }
        }
        number
    }

    fn read_operator<C: Iterator<Item = char>>(&self, first: char, chars: &mut C) -> String {
        let mut op = String::new();
        op.push(first);
        while let Some(&c) = chars.peek() {
            if "+-*/%=<>!&|^~".contains(c) {
                op.push(c);
                chars.next();
            } else {
                break;
            }
        }
        op
    }

    fn is_keyword(&self, s: &str) -> bool {
        matches!(s,
            "fn" | "let" | "if" | "else" | "while" | "for" | "return" | "struct" | "enum" |
            "impl" | "trait" | "where" | "match" | "break" | "continue" | "defer" |
            "true" | "false" | "null" | "self" | "super" | "pub" | "priv" | "const" |
            "static" | "async" | "await" | "move" | "ref" | "mut" | "linear" | "comptime"
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    Identifier(String),
    Keyword(String),
    Number(String),
    String(String),
    Operator(String),
    Delimiter(char),
    EOF,
}

#[derive(Debug)]
pub struct LexError {
    pub message: String,
    pub position: usize,
}

struct Parser<'a> {
    tokens: &'a [Token],
    position: usize,
}

impl<'a> Parser<'a> {
    fn new(tokens: &'a [Token]) -> Self {
        Self {
            tokens,
            position: 0,
        }
    }

    fn parse(&mut self) -> Result<AST, ParseError> {
        let mut module = Module::new();
        while !self.at_end() {
            module.items.push(self.parse_item()?);
        }
        Ok(module)
    }

    fn parse_item(&mut self) -> Result<ASTItem, ParseError> {
        match self.current() {
            Token::Keyword(kw) if kw == "fn" => self.parse_function(),
            Token::Keyword(kw) if kw == "struct" => self.parse_struct(),
            Token::Keyword(kw) if kw == "enum" => self.parse_enum(),
            Token::Keyword(kw) if kw == "let" => self.parse_let(),
            _ => Err(ParseError::new("unexpected token".to_string(), self.position)),
        }
    }

    fn parse_function(&mut self) -> Result<ASTItem, ParseError> {
        self.consume_keyword("fn")?;
        let name = self.consume_identifier()?;
        self.consume(Token::Delimiter('('))?;
        let mut params = Vec::new();
        if !self.check(Token::Delimiter(')')) {
            loop {
                let param_name = self.consume_identifier()?;
                self.consume(Token::Delimiter(':'))?;
                let param_type = self.parse_type()?;
                params.push((param_name, param_type));
                if !self.consume(Token::Operator(",".to_string())) {
                    break;
                }
            }
        }
        self.consume(Token::Delimiter(')'))?;
        let return_type = if self.consume(Token::Operator("->".to_string())) {
            self.parse_type()?
        } else {
            Type::Unit
        };
        self.consume(Token::Delimiter('{'))?;
        let mut body = Vec::new();
        while !self.check(Token::Delimiter('}')) {
            body.push(self.parse_statement()?);
        }
        self.consume(Token::Delimiter('}'))?;
        Ok(ASTItem::Function(Function {
            name,
            params,
            return_type,
            body,
        }))
    }

    fn parse_struct(&mut self) -> Result<ASTItem, ParseError> {
        self.consume_keyword("struct")?;
        let name = self.consume_identifier()?;
        self.consume(Token::Delimiter('{'))?;
        let mut fields = Vec::new();
        while !self.check(Token::Delimiter('}')) {
            let field_name = self.consume_identifier()?;
            self.consume(Token::Delimiter(':'))?;
            let field_type = self.parse_type()?;
            fields.push((field_name, field_type));
            self.consume(Token::Operator(",".to_string())).ok();
        }
        self.consume(Token::Delimiter('}'))?;
        Ok(ASTItem::Struct(StructDef { name, fields }))
    }

    fn parse_enum(&mut self) -> Result<ASTItem, ParseError> {
        self.consume_keyword("enum")?;
        let name = self.consume_identifier()?;
        self.consume(Token::Delimiter('{'))?;
        let mut variants = Vec::new();
        while !self.check(Token::Delimiter('}')) {
            let variant_name = self.consume_identifier()?;
            self.consume(Token::Delimiter('('))?;
            let mut variant_types = Vec::new();
            if !self.check(Token::Delimiter(')')) {
                loop {
                    variant_types.push(self.parse_type()?);
                    if !self.consume(Token::Operator(",".to_string())) {
                        break;
                    }
                }
            }
            self.consume(Token::Delimiter(')'))?;
            variants.push(EnumVariant {
                name: variant_name,
                types: variant_types,
            });
            self.consume(Token::Operator(",".to_string())).ok();
        }
        self.consume(Token::Delimiter('}'))?;
        Ok(ASTItem::Enum(EnumDef { name, variants }))
    }

    fn parse_let(&mut self) -> Result<ASTItem, ParseError> {
        self.consume_keyword("let")?;
        let name = self.consume_identifier()?;
        let ty = if self.consume(Token::Delimiter(':')).is_ok() {
            self.parse_type()?
        } else {
            Type::Inferred
        };
        self.consume(Token::Operator("=".to_string()))?;
        let initializer = self.parse_expression()?;
        Ok(ASTItem::Let(LetStmt { name, ty, initializer }))
    }

    fn parse_statement(&mut self) -> Result<Stmt, ParseError> {
        match self.current() {
            Token::Keyword(kw) if kw == "let" => self.parse_let_statement(),
            Token::Keyword(kw) if kw == "if" => self.parse_if_statement(),
            Token::Keyword(kw) if kw == "while" => self.parse_while_statement(),
            Token::Keyword(kw) if kw == "for" => self.parse_for_statement(),
            Token::Keyword(kw) if kw == "return" => self.parse_return_statement(),
            Token::Keyword(kw) if kw == "break" => self.parse_break_statement(),
            Token::Keyword(kw) if kw == "continue" => self.parse_continue_statement(),
            Token::Keyword(kw) if kw == "defer" => self.parse_defer_statement(),
            _ => self.parse_expression_statement(),
        }
    }

    fn parse_if_statement(&mut self) -> Result<Stmt, ParseError> {
        self.consume_keyword("if")?;
        let condition = self.parse_expression()?;
        self.consume(Token::Delimiter('{'))?;
        let then_branch = self.parse_block()?;
        let else_branch = if self.consume_keyword("else").is_ok() {
            Some(self.parse_else_branch()?)
        } else {
            None
        };
        Ok(Stmt::If(IfStmt { condition, then_branch, else_branch }))
    }

    fn parse_else_branch(&mut self) -> Result<ElseBranch, ParseError> {
        if self.consume_keyword("if").is_ok() {
            let condition = self.parse_expression()?;
            self.consume(Token::Delimiter('{'))?;
            let then_branch = self.parse_block()?;
            let else_branch = if self.consume_keyword("else").is_ok() {
                Some(self.parse_else_branch()?)
            } else {
                None
            };
            Ok(ElseBranch::ElseIf(Box::new(IfStmt { condition, then_branch, else_branch })))
        } else {
            self.consume(Token::Delimiter('{'))?;
            let block = self.parse_block()?;
            Ok(ElseBranch::Else(block))
        }
    }

    fn parse_while_statement(&mut self) -> Result<Stmt, ParseError> {
        self.consume_keyword("while")?;
        let condition = self.parse_expression()?;
        self.consume(Token::Delimiter('{'))?;
        let body = self.parse_block()?;
        Ok(Stmt::While(WhileStmt { condition, body }))
    }

    fn parse_for_statement(&mut self) -> Result<Stmt, ParseError> {
        self.consume_keyword("for")?;
        let variable = self.consume_identifier()?;
        self.consume_keyword("in")?;
        let range = self.parse_expression()?;
        self.consume(Token::Delimiter('{'))?;
        let body = self.parse_block()?;
        Ok(Stmt::For(ForStmt { variable, range, body }))
    }

    fn parse_return_statement(&mut self) -> Result<Stmt, ParseError> {
        self.consume_keyword("return")?;
        let value = if self.check(Token::Delimiter('{')) || self.check(Token::Keyword(_)) {
            None
        } else {
            Some(self.parse_expression()?)
        };
        Ok(Stmt::Return(value))
    }

    fn parse_break_statement(&mut self) -> Result<Stmt, ParseError> {
        self.consume_keyword("break")?;
        Ok(Stmt::Break)
    }

    fn parse_continue_statement(&mut self) -> Result<Stmt, ParseError> {
        self.consume_keyword("continue")?;
        Ok(Stmt::Continue)
    }

    fn parse_defer_statement(&mut self) -> Result<Stmt, ParseError> {
        self.consume_keyword("defer")?;
        let expression = self.parse_expression()?;
        Ok(Stmt::Defer(expression))
    }

    fn parse_expression_statement(&mut self) -> Result<Stmt, ParseError> {
        let expr = self.parse_expression()?;
        Ok(Stmt::Expr(expr))
    }

    fn parse_block(&mut self) -> Result<Vec<Stmt>, ParseError> {
        let mut statements = Vec::new();
        while !self.check(Token::Delimiter('}')) {
            statements.push(self.parse_statement()?);
        }
        Ok(statements)
    }

    fn parse_expression(&mut self) -> Result<Expr, ParseError> {
        self.parse_assignment()
    }

    fn parse_assignment(&mut self) -> Result<Expr, ParseError> {
        let lhs = self.parse_binary()?;
        if self.consume(Token::Operator("=".to_string())).is_ok() {
            let rhs = self.parse_assignment()?;
            return Ok(Expr::Assign(Box::new(lhs), Box::new(rhs)));
        }
        if self.consume(Token::Operator("+=".to_string())).is_ok() {
            let rhs = self.parse_assignment()?;
            return Ok(Expr::AddAssign(Box::new(lhs), Box::new(rhs)));
        }
        Ok(lhs)
    }

    fn parse_binary(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.parse_unary()?;
        while let Ok(op) = self.parse_binary_op() {
            let rhs = self.parse_unary()?;
            expr = Expr::Binary(Box::new(expr), op, Box::new(rhs));
        }
        Ok(expr)
    }

    fn parse_binary_op(&mut self) -> Result<BinaryOp, ParseError> {
        match self.current() {
            Token::Operator(op) => {
                let binop = match op.as_str() {
                    "+" => BinaryOp::Add,
                    "-" => BinaryOp::Sub,
                    "*" => BinaryOp::Mul,
                    "/" => BinaryOp::Div,
                    "%" => BinaryOp::Mod,
                    "==" => BinaryOp::Eq,
                    "!=" => BinaryOp::Ne,
                    "<" => BinaryOp::Lt,
                    "<=" => BinaryOp::Le,
                    ">" => BinaryOp::Gt,
                    ">=" => BinaryOp::Ge,
                    "&&" => BinaryOp::And,
                    "||" => BinaryOp::Or,
                    "&" => BinaryOp::BitAnd,
                    "|" => BinaryOp::BitOr,
                    "^" => BinaryOp::BitXor,
                    "<<" => BinaryOp::Shl,
                    ">>" => BinaryOp::Shr,
                    _ => return Err(ParseError::new("unknown operator".to_string(), self.position)),
                };
                self.position += 1;
                Ok(binop)
            }
            _ => Err(ParseError::new("expected operator".to_string(), self.position)),
        }
    }

    fn parse_unary(&mut self) -> Result<Expr, ParseError> {
        if let Token::Operator(op) = self.current() {
            let unop = match op.as_str() {
                "-" => Some(UnaryOp::Neg),
                "!" => Some(UnaryOp::Not),
                "&" => Some(UnaryOp::Ref),
                "*" => Some(UnaryOp::Deref),
                _ => None,
            };
            if let Some(unop) = unop {
                self.position += 1;
                let operand = self.parse_unary()?;
                return Ok(Expr::Unary(unop, Box::new(operand)));
            }
        }
        self.parse_postfix()
    }

    fn parse_postfix(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.parse_primary()?;
        loop {
            if self.consume(Token::Delimiter('.')).is_ok() {
                let field = self.consume_identifier()?;
                expr = Expr::Field(Box::new(expr), field);
            } else if self.consume(Token::Delimiter('[')).is_ok() {
                let index = self.parse_expression()?;
                self.consume(Token::Delimiter(']'))?;
                expr = Expr::Index(Box::new(expr), Box::new(index));
            } else if self.consume(Token::Delimiter('(')).is_ok() {
                let mut args = Vec::new();
                if !self.check(Token::Delimiter(')')) {
                    loop {
                        args.push(self.parse_expression()?);
                        if !self.consume(Token::Operator(",".to_string())) {
                            break;
                        }
                    }
                }
                self.consume(Token::Delimiter(')'))?;
                expr = Expr::Call(Box::new(expr), args);
            } else {
                break;
            }
        }
        Ok(expr)
    }

    fn parse_primary(&mut self) -> Result<Expr, ParseError> {
        match self.current() {
            Token::Identifier(name) => {
                self.position += 1;
                Ok(Expr::Var(name))
            }
            Token::Number(n) => {
                self.position += 1;
                Ok(Expr::Lit(Literal::Number(n)))
            }
            Token::String(s) => {
                self.position += 1;
                Ok(Expr::Lit(Literal::String(s)))
            }
            Token::Keyword(kw) if kw == "true" => {
                self.position += 1;
                Ok(Expr::Lit(Literal::Bool(true)))
            }
            Token::Keyword(kw) if kw == "false" => {
                self.position += 1;
                Ok(Expr::Lit(Literal::Bool(false)))
            }
            Token::Keyword(kw) if kw == "null" => {
                self.position += 1;
                Ok(Expr::Lit(Literal::Null))
            }
            Token::Keyword(kw) if kw == "if" => self.parse_if_expr(),
            Token::Keyword(kw) if kw == "match" => self.parse_match_expr(),
            Token::Keyword(kw) if kw == "new" => self.parse_new_expr(),
            Token::Delimiter('(') => self.parse_paren_or_tuple(),
            _ => Err(ParseError::new("unexpected token".to_string(), self.position)),
        }
    }

    fn parse_if_expr(&mut self) -> Result<Expr, ParseError> {
        self.consume_keyword("if")?;
        let condition = self.parse_expression()?;
        self.consume_keyword("then")?;
        let then_expr = self.parse_expression()?;
        self.consume_keyword("else")?;
        let else_expr = self.parse_expression()?;
        Ok(Expr::If(Box::new(condition), Box::new(then_expr), Box::new(else_expr)))
    }

    fn parse_match_expr(&mut self) -> Result<Expr, ParseError> {
        self.consume_keyword("match")?;
        let target = self.parse_expression()?;
        self.consume(Token::Delimiter('{'))?;
        let mut arms = Vec::new();
        while !self.check(Token::Delimiter('}')) {
            let pattern = self.parse_pattern()?;
            self.consume(Token::Operator("=>".to_string()))?;
            let result = self.parse_expression()?;
            arms.push((pattern, result));
            self.consume(Token::Operator(",".to_string())).ok();
        }
        self.consume(Token::Delimiter('}'))?;
        Ok(Expr::Match(Box::new(target), arms))
    }

    fn parse_pattern(&mut self) -> Result<Pattern, ParseError> {
        match self.current() {
            Token::Identifier(name) => {
                self.position += 1;
                if self.consume(Token::Delimiter('(')).is_ok() {
                    let mut variants = Vec::new();
                    if !self.check(Token::Delimiter(')')) {
                        loop {
                            variants.push(self.parse_expression()?);
                            if !self.consume(Token::Operator(",".to_string())) {
                                break;
                            }
                        }
                    }
                    self.consume(Token::Delimiter(')'))?;
                    Ok(Pattern::Variant(name, variants))
                } else {
                    Ok(Pattern::Var(name))
                }
            }
            Token::Number(n) => Ok(Pattern::Lit(Literal::Number(n))),
            Token::Keyword(kw) if kw == "_" => {
                self.position += 1;
                Ok(Pattern::Wildcard)
            }
            _ => Err(ParseError::new("invalid pattern".to_string(), self.position)),
        }
    }

    fn parse_new_expr(&mut self) -> Result<Expr, ParseError> {
        self.consume_keyword("new")?;
        let type_name = self.consume_identifier()?;
        self.consume(Token::Delimiter('('))?;
        let mut args = Vec::new();
        if !self.check(Token::Delimiter(')')) {
            loop {
                args.push(self.parse_expression()?);
                if !self.consume(Token::Operator(",".to_string())) {
                    break;
                }
            }
        }
        self.consume(Token::Delimiter(')'))?;
        Ok(Expr::New(type_name, args))
    }

    fn parse_paren_or_tuple(&mut self) -> Result<Expr, ParseError> {
        self.consume(Token::Delimiter('('))?;
        if self.consume(Token::Delimiter(')')).is_ok() {
            return Ok(Expr::Lit(Literal::Unit));
        }
        let first = self.parse_expression()?;
        if self.consume(Token::Delimiter(')')).is_ok() {
            return Ok(first);
        }
        self.consume(Token::Operator(",".to_string()))?;
        let mut elements = vec![first];
        loop {
            elements.push(self.parse_expression()?);
            if !self.consume(Token::Operator(",".to_string())) {
                break;
            }
        }
        self.consume(Token::Delimiter(')'))?;
        Ok(Expr::Tuple(elements))
    }

    fn parse_type(&mut self) -> Result<Type, ParseError> {
        match self.current() {
            Token::Identifier(name) => {
                self.position += 1;
                let base_type = match name.as_str() {
                    "int" => Type::Int64,
                    "float" => Type::Float64,
                    "bool" => Type::Bool,
                    "string" => Type::String,
                    "unit" => Type::Unit,
                    "char" => Type::Char,
                    "void" => Type::Void,
                    _ => Type::Named(name),
                };
                if self.consume(Token::Operator("[".to_string())).is_ok() {
                    self.consume(Token::Operator("]".to_string()))?;
                    return Ok(Type::Slice(Box::new(base_type)));
                }
                if self.consume(Token::Operator("*".to_string())).is_ok() {
                    return Ok(Type::Pointer(Box::new(base_type)));
                }
                Ok(base_type)
            }
            Token::Operator(Token::Operator("fn".to_string())) => self.parse_function_type(),
            Token::Delimiter('(') => self.parse_tuple_type(),
            _ => Err(ParseError::new("expected type".to_string(), self.position)),
        }
    }

    fn parse_function_type(&mut self) -> Result<Type, ParseError> {
        self.consume(Token::Keyword("fn".to_string()))?;
        self.consume(Token::Delimiter('('))?;
        let mut param_types = Vec::new();
        if !self.check(Token::Delimiter(')')) {
            loop {
                param_types.push(self.parse_type()?);
                if !self.consume(Token::Operator(",".to_string())) {
                    break;
                }
            }
        }
        self.consume(Token::Delimiter(')'))?;
        self.consume(Token::Operator("->".to_string()))?;
        let return_type = self.parse_type()?;
        Ok(Type::Function(param_types, Box::new(return_type)))
    }

    fn parse_tuple_type(&mut self) -> Result<Type, ParseError> {
        self.consume(Token::Delimiter('('))?;
        let mut types = Vec::new();
        if !self.check(Token::Delimiter(')')) {
            loop {
                types.push(self.parse_type()?);
                if !self.consume(Token::Operator(",".to_string())) {
                    break;
                }
            }
        }
        self.consume(Token::Delimiter(')'))?;
        Ok(Type::Tuple(types))
    }

    fn consume(&mut self, expected: Token) -> Result<(), ParseError> {
        if self.current() == expected {
            self.position += 1;
            Ok(())
        } else {
            Err(ParseError::new(format!("expected {:?}", expected), self.position))
        }
    }

    fn consume_keyword(&mut self, keyword: &str) -> Result<(), ParseError> {
        if let Token::Keyword(kw) = self.current() {
            if kw == keyword {
                self.position += 1;
                return Ok(());
            }
        }
        Err(ParseError::new(format!("expected keyword '{}'", keyword), self.position))
    }

    fn consume_identifier(&mut self) -> Result<String, ParseError> {
        if let Token::Identifier(name) = self.current() {
            self.position += 1;
            Ok(name.clone())
        } else {
            Err(ParseError::new("expected identifier".to_string(), self.position))
        }
    }

    fn current(&self) -> Token {
        if self.position < self.tokens.len() {
            self.tokens[self.position].clone()
        } else {
            Token::EOF
        }
    }

    fn at_end(&self) -> bool {
        self.position >= self.tokens.len()
    }

    fn check(&self, token: Token) -> bool {
        if self.position < self.tokens.len() {
            self.tokens[self.position] == token
        } else {
            false
        }
    }
}

#[derive(Debug)]
pub struct ParseError {
    pub message: String,
    pub position: usize,
}

impl ParseError {
    fn new(message: String, position: usize) -> Self {
        Self { message, position }
    }
}

struct Typechecker<'a> {
    state: &'a Arc<SharedCompilerState>,
    variables: HashMap<String, Type>,
    type_pool: HashMap<String, Type>,
}

impl<'a> Typechecker<'a> {
    fn new(state: &'a Arc<SharedCompilerState>) -> Self {
        Self {
            state,
            variables: HashMap::new(),
            type_pool: HashMap::new(),
        }
    }

    fn check(&mut self, ast: &AST) -> Vec<Type> {
        self.type_pool.insert("int".to_string(), Type::Int64);
        self.type_pool.insert("float".to_string(), Type::Float64);
        self.type_pool.insert("bool".to_string(), Type::Bool);
        self.type_pool.insert("string".to_string(), Type::String);
        self.type_pool.insert("unit".to_string(), Type::Unit);
        for item in &ast.items {
            self.check_item(item);
        }
        self.type_pool.values().cloned().collect()
    }

    fn check_item(&mut self, item: &ASTItem) {
        match item {
            ASTItem::Function(func) => self.check_function(func),
            ASTItem::Struct(def) => self.check_struct(def),
            ASTItem::Enum(def) => self.check_enum(def),
            ASTItem::Let(stmt) => self.check_let(stmt),
        }
    }

    fn check_function(&mut self, func: &Function) {
        for (name, ty) in &func.params {
            self.variables.insert(name.clone(), ty.clone());
        }
        for stmt in &func.body {
            self.check_statement(stmt);
        }
    }

    fn check_struct(&mut self, def: &StructDef) {
        let fields: Vec<(String, Type)> = def.fields.iter()
            .map(|(n, t)| (n.clone(), self.resolve_type(t)))
            .collect();
        self.type_pool.insert(def.name.clone(), Type::Struct(fields));
    }

    fn check_enum(&mut self, def: &EnumDef) {
        let variants: Vec<(String, Vec<Type>)> = def.variants.iter()
            .map(|v| (v.name.clone(), v.types.iter().map(|t| self.resolve_type(t)).collect()))
            .collect();
        self.type_pool.insert(def.name.clone(), Type::Enum(variants));
    }

    fn check_let(&mut self, stmt: &LetStmt) {
        let ty = if stmt.ty == Type::Inferred {
            self.infer_type(&stmt.initializer)
        } else {
            stmt.ty.clone()
        };
        self.variables.insert(stmt.name.clone(), ty.clone());
    }

    fn check_statement(&mut self, stmt: &Stmt) -> Type {
        match stmt {
            Stmt::Let(l) => self.check_let(l),
            Stmt::If(i) => self.check_if(i),
            Stmt::While(w) => self.check_while(w),
            Stmt::For(f) => self.check_for(f),
            Stmt::Return(r) => self.check_return(r),
            Stmt::Break => Type::Void,
            Stmt::Continue => Type::Void,
            Stmt::Defer(d) => {
                self.check_expression(d);
                Type::Void
            }
            Stmt::Expr(e) => self.check_expression(e),
        }
    }

    fn check_if(&mut self, stmt: &IfStmt) -> Type {
        let cond_ty = self.check_expression(&stmt.condition);
        if cond_ty != Type::Bool {
            self.state.add_error(CompileError {
                message: "if condition must be bool".to_string(),
                node_id: NodeId(0),
                severity: ErrorSeverity::Error,
            });
        }
        self.check_block(&stmt.then_branch);
        if let Some(else_branch) = &stmt.else_branch {
            self.check_else_branch(else_branch);
        }
        Type::Unit
    }

    fn check_else_branch(&mut self, branch: &ElseBranch) {
        match branch {
            ElseBranch::Else(block) => self.check_block(block),
            ElseBranch::ElseIf(if_stmt) => self.check_if(if_stmt),
        }
    }

    fn check_while(&mut self, stmt: &WhileStmt) {
        let cond_ty = self.check_expression(&stmt.condition);
        if cond_ty != Type::Bool {
            self.state.add_error(CompileError {
                message: "while condition must be bool".to_string(),
                node_id: NodeId(0),
                severity: ErrorSeverity::Error,
            });
        }
        self.check_block(&stmt.body);
    }

    fn check_for(&mut self, stmt: &ForStmt) {
        self.variables.insert(stmt.variable.clone(), Type::Int64);
        let range_ty = self.check_expression(&stmt.range);
        if range_ty != Type::Int64 && range_ty != Type::Slice(Box::new(Type::Int64)) {
            self.state.add_error(CompileError {
                message: "for loop range must be int".to_string(),
                node_id: NodeId(0),
                severity: ErrorSeverity::Error,
            });
        }
        self.check_block(&stmt.body);
    }

    fn check_return(&mut self, expr: &Option<Expr>) -> Type {
        if let Some(e) = expr {
            self.check_expression(e)
        } else {
            Type::Void
        }
    }

    fn check_block(&mut self, block: &[Stmt]) -> Type {
        let mut last_ty = Type::Unit;
        for stmt in block {
            last_ty = self.check_statement(stmt);
        }
        last_ty
    }

    fn check_expression(&mut self, expr: &Expr) -> Type {
        match expr {
            Expr::Var(name) => {
                self.variables.get(name)
                    .cloned()
                    .unwrap_or(Type::Unknown)
            }
            Expr::Lit(lit) => self.check_literal(lit),
            Expr::Binary(lhs, op, rhs) => self.check_binary(lhs, op, rhs),
            Expr::Unary(op, operand) => self.check_unary(op, operand),
            Expr::Call(func, args) => self.check_call(func, args),
            Expr::If(cond, then_expr, else_expr) => self.check_if_expr(cond, then_expr, else_expr),
            Expr::Match(target, arms) => self.check_match(target, arms),
            Expr::Field(expr, field) => self.check_field(expr, field),
            Expr::Index(expr, index) => self.check_index(expr, index),
            Expr::Assign(lhs, rhs) => self.check_assign(lhs, rhs),
            Expr::Tuple(elements) => Type::Tuple(elements.iter().map(|e| self.check_expression(e)).collect()),
            Expr::New(name, args) => self.check_new(name, args),
            Expr::Block(block) => self.check_block(block),
            Expr::Loop(body) => {
                self.check_block(body);
                Type::Never
            }
        }
    }

    fn check_literal(&self, lit: &Literal) -> Type {
        match lit {
            Literal::Number(_) => Type::Int64,
            Literal::Float(_) => Type::Float64,
            Literal::String(_) => Type::String,
            Literal::Bool(_) => Type::Bool,
            Literal::Char(_) => Type::Char,
            Literal::Null => Type::Pointer(Box::new(Type::Void)),
            Literal::Unit => Type::Unit,
        }
    }

    fn check_binary(&mut self, lhs: &Expr, op: &BinaryOp, rhs: &Expr) -> Type {
        let lhs_ty = self.check_expression(lhs);
        let rhs_ty = self.check_expression(rhs);
        match op {
            BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div | BinaryOp::Mod => {
                if lhs_ty == Type::Int64 && rhs_ty == Type::Int64 {
                    Type::Int64
                } else if lhs_ty == Type::Float64 && rhs_ty == Type::Float64 {
                    Type::Float64
                } else {
                    Type::Unknown
                }
            }
            BinaryOp::Eq | BinaryOp::Ne | BinaryOp::Lt | BinaryOp::Le | BinaryOp::Gt | BinaryOp::Ge => {
                if lhs_ty == rhs_ty {
                    Type::Bool
                } else {
                    Type::Unknown
                }
            }
            BinaryOp::And | BinaryOp::Or => {
                if lhs_ty == Type::Bool && rhs_ty == Type::Bool {
                    Type::Bool
                } else {
                    Type::Unknown
                }
            }
            BinaryOp::BitAnd | BinaryOp::BitOr | BinaryOp::BitXor => {
                if lhs_ty == rhs_ty && (lhs_ty == Type::Int64 || lhs_ty == Type::Bool) {
                    lhs_ty
                } else {
                    Type::Unknown
                }
            }
            BinaryOp::Shl | BinaryOp::Shr => {
                if lhs_ty == Type::Int64 && rhs_ty == Type::Int64 {
                    Type::Int64
                } else {
                    Type::Unknown
                }
            }
        }
    }

    fn check_unary(&mut self, op: &UnaryOp, operand: &Expr) -> Type {
        let operand_ty = self.check_expression(operand);
        match op {
            UnaryOp::Neg => {
                if operand_ty == Type::Int64 || operand_ty == Type::Float64 {
                    operand_ty
                } else {
                    Type::Unknown
                }
            }
            UnaryOp::Not => {
                if operand_ty == Type::Bool {
                    Type::Bool
                } else {
                    Type::Unknown
                }
            }
            UnaryOp::Ref => Type::Pointer(Box::new(operand_ty)),
            UnaryOp::Deref => {
                if let Type::Pointer(inner) = operand_ty {
                    *inner
                } else {
                    Type::Unknown
                }
            }
        }
    }

    fn check_call(&mut self, func: &Expr, args: &[Expr]) -> Type {
        self.check_expression(func);
        for arg in args {
            self.check_expression(arg);
        }
        Type::Unknown
    }

    fn check_if_expr(&mut self, cond: &Expr, then_expr: &Expr, else_expr: &Expr) -> Type {
        let cond_ty = self.check_expression(cond);
        if cond_ty != Type::Bool {
            self.state.add_error(CompileError {
                message: "if condition must be bool".to_string(),
                node_id: NodeId(0),
                severity: ErrorSeverity::Error,
            });
        }
        let then_ty = self.check_expression(then_expr);
        let else_ty = self.check_expression(else_expr);
        if then_ty == else_ty {
            then_ty
        } else {
            Type::Unknown
        }
    }

    fn check_match(&mut self, _target: &Expr, _arms: &[(Pattern, Expr)]) -> Type {
        Type::Unknown
    }

    fn check_field(&mut self, expr: &Expr, _field: &str) -> Type {
        self.check_expression(expr)
    }

    fn check_index(&mut self, expr: &Expr, index: &Expr) -> Type {
        self.check_expression(expr);
        self.check_expression(index);
        Type::Unknown
    }

    fn check_assign(&mut self, lhs: &Expr, rhs: &Expr) -> Type {
        let lhs_ty = self.check_expression(lhs);
        let rhs_ty = self.check_expression(rhs);
        if lhs_ty == rhs_ty || rhs_ty == Type::Unknown {
            lhs_ty
        } else {
            Type::Unknown
        }
    }

    fn check_new(&mut self, name: &str, args: &[Expr]) -> Type {
        for arg in args {
            self.check_expression(arg);
        }
        self.type_pool.get(name)
            .cloned()
            .unwrap_or(Type::Unknown)
    }

    fn infer_type(&self, expr: &Expr) -> Type {
        self.check_expression(expr)
    }

    fn resolve_type(&self, ty: &Type) -> Type {
        ty.clone()
    }
}

struct Optimizer<'a> {
    state: &'a Arc<SharedCompilerState>,
}

impl<'a> Optimizer<'a> {
    fn new(state: &'a Arc<SharedCompilerState>) -> Self {
        Self { state }
    }

    fn optimize(&self, hir: &HIR) -> IR {
        let mut ir = IR::new();
        for func in &hir.functions {
            let mut ir_func = IRFunction::new(&func.name);
            for stmt in &func.body {
                self.lower_statement(stmt, &mut ir_func);
            }
            ir.functions.push(ir_func);
        }
        ir
    }

    fn lower_statement(&self, stmt: &Stmt, func: &mut IRFunction) {
        match stmt {
            Stmt::Let(l) => self.lower_let(l, func),
            Stmt::Return(r) => self.lower_return(r, func),
            Stmt::Expr(e) => self.lower_expr(e, func),
            _ => {}
        }
    }

    fn lower_let(&self, _l: &LetStmt, _func: &mut IRFunction) {}
    fn lower_return(&self, _r: &Option<Expr>, _func: &mut IRFunction) {}
    fn lower_expr(&self, _e: &Expr, _func: &mut IRFunction) {}
}

struct ParsedModule {
    source: String,
    ast: AST,
    tokens: Vec<Token>,
}

struct TypecheckedModule {
    source: String,
    ast: AST,
    types: Vec<Type>,
    hir: HIR,
}

struct OptimizedModule {
    source: String,
    ast: AST,
    types: Vec<Type>,
    hir: HIR,
    ir: IR,
}

#[derive(Debug, Clone)]
struct AST {
    items: Vec<ASTItem>,
}

impl AST {
    fn new() -> Self {
        Self { items: Vec::new() }
    }
}

#[derive(Debug, Clone)]
enum ASTItem {
    Function(Function),
    Struct(StructDef),
    Enum(EnumDef),
    Let(LetStmt),
}

#[derive(Debug, Clone)]
struct Function {
    name: String,
    params: Vec<(String, Type)>,
    return_type: Type,
    body: Vec<Stmt>,
}

#[derive(Debug, Clone)]
struct StructDef {
    name: String,
    fields: Vec<(String, Type)>,
}

#[derive(Debug, Clone)]
struct EnumDef {
    name: String,
    variants: Vec<EnumVariant>,
}

#[derive(Debug, Clone)]
struct EnumVariant {
    name: String,
    types: Vec<Type>,
}

#[derive(Debug, Clone)]
struct LetStmt {
    name: String,
    ty: Type,
    initializer: Expr,
}

#[derive(Debug, Clone)]
enum Stmt {
    Let(LetStmt),
    If(IfStmt),
    While(WhileStmt),
    For(ForStmt),
    Return(Option<Expr>),
    Break,
    Continue,
    Defer(Expr),
    Expr(Expr),
}

#[derive(Debug, Clone)]
struct IfStmt {
    condition: Expr,
    then_branch: Vec<Stmt>,
    else_branch: Option<ElseBranch>,
}

#[derive(Debug, Clone)]
enum ElseBranch {
    Else(Vec<Stmt>),
    ElseIf(Box<IfStmt>),
}

#[derive(Debug, Clone)]
struct WhileStmt {
    condition: Expr,
    body: Vec<Stmt>,
}

#[derive(Debug, Clone)]
struct ForStmt {
    variable: String,
    range: Expr,
    body: Vec<Stmt>,
}

#[derive(Debug, Clone)]
enum Expr {
    Var(String),
    Lit(Literal),
    Binary(Box<Expr>, BinaryOp, Box<Expr>),
    Unary(UnaryOp, Box<Expr>),
    Call(Box<Expr>, Vec<Expr>),
    If(Box<Expr>, Box<Expr>, Box<Expr>),
    Match(Box<Expr>, Vec<(Pattern, Expr)>),
    Field(Box<Expr>, String),
    Index(Box<Expr>, Box<Expr>),
    Assign(Box<Expr>, Box<Expr>),
    Tuple(Vec<Expr>),
    New(String, Vec<Expr>),
    Block(Vec<Stmt>),
    Loop(Vec<Stmt>),
}

#[derive(Debug, Clone)]
enum Literal {
    Number(String),
    Float(String),
    String(String),
    Bool(bool),
    Char(char),
    Null,
    Unit,
}

#[derive(Debug, Clone, Copy)]
enum BinaryOp {
    Add, Sub, Mul, Div, Mod,
    Eq, Ne, Lt, Le, Gt, Ge,
    And, Or,
    BitAnd, BitOr, BitXor,
    Shl, Shr,
}

#[derive(Debug, Clone, Copy)]
enum UnaryOp {
    Neg, Not, Ref, Deref,
}

#[derive(Debug, Clone)]
enum Pattern {
    Var(String),
    Variant(String, Vec<Expr>),
    Lit(Literal),
    Wildcard,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Type {
    Void,
    Unit,
    Bool,
    Int64,
    Float64,
    Char,
    String,
    Named(String),
    Pointer(Box<Type>),
    Slice(Box<Type>),
    Function(Vec<Type>, Box<Type>),
    Tuple(Vec<Type>),
    Struct(Vec<(String, Type)>),
    Enum(Vec<(String, Vec<Type>)>),
    Unknown,
    Inferred,
    Never,
}

struct HIR {
    functions: Vec<HIRFunction>,
}

impl HIR {
    fn from_ast(ast: &AST, types: &[Type]) -> Self {
        Self { functions: Vec::new() }
    }

    fn new() -> Self {
        Self { functions: Vec::new() }
    }
}

struct HIRFunction {
    name: String,
    params: Vec<(String, Type)>,
    return_type: Type,
    body: Vec<HIRStmt>,
}

struct IR {
    functions: Vec<IRFunction>,
}

impl IR {
    fn new() -> Self {
        Self { functions: Vec::new() }
    }
}

struct IRFunction {
    name: String,
    instructions: Vec<IRInstruction>,
}

impl IRFunction {
    fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            instructions: Vec::new(),
        }
    }
}

enum IRInstruction {
    Mov(IROperand, IROperand),
    Add(IROperand, IROperand),
    Sub(IROperand, IROperand),
    Mul(IROperand, IROperand),
    Div(IROperand, IROperand),
    Call(String),
    Ret(Option<IROperand>),
    Label(String),
    Jmp(String),
    Je(String),
    Jne(String),
}

enum IROperand {
    Register(usize),
    Immediate(i64),
    Label(String),
}

impl Module {
    fn new() -> Self {
        Self { items: Vec::new() }
    }
}

struct Module {
    items: Vec<ASTItem>,
}
