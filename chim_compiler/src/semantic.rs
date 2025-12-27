use crate::ast::{self, Literal, StructField};
use thiserror::Error;
use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Lifetime(pub String);

impl fmt::Display for Lifetime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "'{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LifetimeContext {
    pub active_lifetimes: Vec<Lifetime>,
    pub lifetime_parameters: Vec<Lifetime>,
    pub borrow_tracker: Vec<BorrowRecord>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BorrowRecord {
    pub borrowed_lifetime: Lifetime,
    pub borrower: String,
    pub is_mutable: bool,
    pub position: (usize, usize),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LifetimeRegion(pub Vec<Lifetime>);

impl LifetimeRegion {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn add_lifetime(&mut self, lifetime: Lifetime) {
        if !self.0.contains(&lifetime) {
            self.0.push(lifetime);
        }
    }

    pub fn contains(&self, lifetime: &Lifetime) -> bool {
        self.0.contains(lifetime)
    }
}

#[derive(Debug, Error, Clone)]
pub enum LifetimeError {
    #[error("Lifetime {0} is not in scope")]
    UndefinedLifetime(Lifetime),
    #[error("Cannot return reference with lifetime {0} that may outlive the function")]
    ReturnedLifetimeError(Lifetime),
    #[error("Mutable borrow {0} conflicts with immutable borrow of {1}")]
    MutableBorrowConflict(String, String),
    #[error("Immutable borrow conflicts with mutable borrow of {0}")]
    ImmutableBorrowConflict(String),
    #[error("Borrowed value does not live long enough")]
    BorrowedValueLifetimeTooShort,
    #[error("Lifetime mismatch: expected {expected}, found {found}")]
    LifetimeMismatch { expected: Lifetime, found: Lifetime },
    #[error("Cannot assign to borrowed value")]
    CannotAssignToBorrowed,
    #[error("Dangling reference: reference outlives referent")]
    DanglingReference,
}

impl Default for LifetimeContext {
    fn default() -> Self {
        Self::new()
    }
}

impl LifetimeContext {
    pub fn new() -> Self {
        Self {
            active_lifetimes: Vec::new(),
            lifetime_parameters: Vec::new(),
            borrow_tracker: Vec::new(),
        }
    }

    pub fn enter_scope(&mut self) {
        self.active_lifetimes.push(Lifetime(String::new()));
    }

    pub fn exit_scope(&mut self) {
        self.active_lifetimes.pop();
        self.borrow_tracker.retain(|record| {
            self.active_lifetimes.contains(&record.borrowed_lifetime)
        });
    }

    pub fn declare_lifetime(&mut self, lifetime: Lifetime) {
        self.lifetime_parameters.push(lifetime.clone());
        self.active_lifetimes.push(lifetime);
    }

    pub fn is_lifetime_in_scope(&self, lifetime: &Lifetime) -> bool {
        self.active_lifetimes.contains(lifetime) || self.lifetime_parameters.contains(lifetime)
    }

    pub fn record_borrow(&mut self, borrowed_lifetime: Lifetime, borrower: String, is_mutable: bool, position: (usize, usize)) {
        self.borrow_tracker.push(BorrowRecord {
            borrowed_lifetime,
            borrower,
            is_mutable,
            position,
        });
    }

    pub fn check_borrows(&self, lifetime: &Lifetime) -> Result<(), LifetimeError> {
        let lifetime_borrows: Vec<&BorrowRecord> = self.borrow_tracker.iter()
            .filter(|r| &r.borrowed_lifetime == lifetime)
            .collect();

        let has_mutable = lifetime_borrows.iter().any(|r| r.is_mutable);
        let immutable_count = lifetime_borrows.iter().filter(|r| !r.is_mutable).count();

        if has_mutable && immutable_count > 0 {
            return Err(LifetimeError::MutableBorrowConflict(
                lifetime_borrows.iter().find(|r| r.is_mutable).unwrap().borrower.clone(),
                lifetime_borrows.iter().find(|r| !r.is_mutable).unwrap().borrower.clone(),
            ));
        }

        Ok(())
    }
}

pub struct BorrowChecker {
    pub lifetime_context: LifetimeContext,
    pub errors: Vec<LifetimeError>,
}

impl BorrowChecker {
    pub fn new() -> Self {
        Self {
            lifetime_context: LifetimeContext::new(),
            errors: Vec::new(),
        }
    }

    pub fn check_function_borrows(&mut self, params: &[ast::Parameter], body_type: &str) -> Result<(), Vec<LifetimeError>> {
        for param in params {
            if let Some(ty) = &param.ty {
                if ty.starts_with('&') {
                    self.lifetime_context.enter_scope();
                    let parts: Vec<&str> = ty.split_whitespace().collect();
                    if let Some(lifetime_str) = parts.get(1) {
                        let lifetime = Lifetime(lifetime_str.trim_start_matches('\'').to_string());
                        self.lifetime_context.declare_lifetime(lifetime);
                    }
                }
            }
        }

        if !self.errors.is_empty() {
            Err(self.errors.clone())
        } else {
            Ok(())
        }
    }
}

impl Default for BorrowChecker {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EscapeInfo {
    pub escapes: bool,
    pub captured_by_ref: bool,
    pub address_taken: bool,
}

pub struct EscapeAnalyzer {
    pub escape_info: HashMap<String, EscapeInfo>,
    pub in_loop: bool,
    pub in_function: bool,
}

impl EscapeAnalyzer {
    pub fn new() -> Self {
        Self {
            escape_info: HashMap::new(),
            in_loop: false,
            in_function: false,
        }
    }

    pub fn analyze_variable(&mut self, name: &str, context: &str) -> EscapeInfo {
        let key = format!("{}_{}", context, name);
        self.escape_info.get(&key).cloned().unwrap_or_else(|| EscapeInfo {
            escapes: false,
            captured_by_ref: false,
            address_taken: false,
        })
    }

    pub fn mark_escaped(&mut self, name: &str, context: &str) {
        let key = format!("{}_{}", context, name);
        if let Some(info) = self.escape_info.get_mut(&key) {
            info.escapes = true;
        } else {
            self.escape_info.insert(key, EscapeInfo {
                escapes: true,
                captured_by_ref: false,
                address_taken: false,
            });
        }
    }

    pub fn mark_captured_by_ref(&mut self, name: &str, context: &str) {
        let key = format!("{}_{}", context, name);
        if let Some(info) = self.escape_info.get_mut(&key) {
            info.captured_by_ref = true;
        } else {
            self.escape_info.insert(key, EscapeInfo {
                escapes: false,
                captured_by_ref: true,
                address_taken: false,
            });
        }
    }

    pub fn mark_address_taken(&mut self, name: &str, context: &str) {
        let key = format!("{}_{}", context, name);
        if let Some(info) = self.escape_info.get_mut(&key) {
            info.address_taken = true;
        } else {
            self.escape_info.insert(key, EscapeInfo {
                escapes: false,
                captured_by_ref: false,
                address_taken: true,
            });
        }
    }

    pub fn should_allocate_on_heap(&self, name: &str, context: &str) -> bool {
        let key = format!("{}_{}", context, name);
        if let Some(info) = self.escape_info.get(&key) {
            info.escapes || info.captured_by_ref || info.address_taken
        } else {
            false
        }
    }
}

impl Default for EscapeAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct LoopInfo {
    pub is_invariant: bool,
    pub can_unroll: bool,
    pub unroll_factor: u32,
    pub induction_variable: Option<String>,
    pub bounds_known: bool,
}

pub struct LoopOptimizer {
    pub loop_info: HashMap<String, LoopInfo>,
    pub current_loop_depth: u32,
}

impl LoopOptimizer {
    pub fn new() -> Self {
        Self {
            loop_info: HashMap::new(),
            current_loop_depth: 0,
        }
    }

    pub fn enter_loop(&mut self, loop_label: &str) {
        self.current_loop_depth += 1;
    }

    pub fn exit_loop(&mut self) {
        if self.current_loop_depth > 0 {
            self.current_loop_depth -= 1;
        }
    }

    pub fn analyze_loop(&mut self, loop_label: &str, body: &ast::Expression) {
        let info = LoopInfo {
            is_invariant: false,
            can_unroll: self.current_loop_depth <= 2,
            unroll_factor: if self.current_loop_depth == 0 { 1 } else { 4 >> self.current_loop_depth },
            induction_variable: None,
            bounds_known: false,
        };
        self.loop_info.insert(loop_label.to_string(), info);
    }

    pub fn mark_induction_variable(&mut self, loop_label: &str, var_name: &str) {
        if let Some(info) = self.loop_info.get_mut(loop_label) {
            info.induction_variable = Some(var_name.to_string());
        }
    }

    pub fn can_optimize(&self, loop_label: &str) -> bool {
        self.loop_info.get(loop_label).map(|info| info.can_unroll).unwrap_or(false)
    }

    pub fn get_unroll_factor(&self, loop_label: &str) -> u32 {
        self.loop_info.get(loop_label).map(|info| info.unroll_factor).unwrap_or(1)
    }
}

impl Default for LoopOptimizer {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Error, Clone)]
pub enum SemanticError {
    #[error("Undefined identifier: {0}")]
    UndefinedIdentifier(String),
    #[error("Redeclaration of identifier: {0}")]
    Redeclaration(String),
    #[error("Expected expression of type {expected}, found {found}")]
    TypeMismatch {
        expected: String,
        found: String,
    },
    #[error("Function {name} expects {expected} arguments, found {found}")]
    WrongArgumentCount {
        name: String,
        expected: usize,
        found: usize,
    },
    #[error("Function {name} expects argument {arg_idx} to be of type {expected}, found {found}")]
    WrongArgumentType {
        name: String,
        arg_idx: usize,
        expected: String,
        found: String,
    },
    #[error("Struct {0} is not defined")]
    UndefinedStruct(String),
    #[error("Enum {0} is not defined")]
    UndefinedEnum(String),
    #[error("Struct {struct_name} does not have field {field_name}")]
    UndefinedField {
        struct_name: String,
        field_name: String,
    },
    #[error("Enum {enum_name} does not have variant {variant_name}")]
    UndefinedVariant {
        enum_name: String,
        variant_name: String,
    },
    #[error("Expected variant of enum {expected}, found {found}")]
    ExpectedEnumVariant {
        expected: String,
        found: String,
    },
    #[error("Cannot assign to immutable variable {0}")]
    CannotAssignToImmutable(String),
    #[error("{0} is not a function")]
    NotAFunction(String),
    #[error("{0} is not a type")]
    NotAType(String),
    #[error("Invalid type: {0}")]
    InvalidType(String),
    #[error("Unsupported feature: {0}")]
    UnsupportedFeature(String),
    #[error("Invalid use of dot operator on {0}")]
    InvalidDotOperator(String),
    #[error("Invalid use of index operator on {0}")]
    InvalidIndexOperator(String),
    #[error("Invalid use of dereference operator")]
    InvalidDeref,
    #[error("Invalid use of reference operator")]
    InvalidRef,
    #[error("Cannot return a value from a void function")]
    CannotReturnFromVoidFunction,
    #[error("Missing return value in function that returns {0}")]
    MissingReturnValue(String),
    #[error("Invalid expression in match pattern")]
    InvalidMatchPattern,
    #[error("Multiple patterns match the same value in match expression")]
    DuplicateMatchPattern,
    #[error("Match expression is not exhaustive")]
    NonExhaustiveMatch,
    #[error("Invalid use of {0} as a pattern")]
    InvalidPattern(String),
    #[error("Invalid syntax: {0}")]
    InvalidSyntax(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum SymbolKind {
    Variable {
        mutable: bool,
        ty: String,
    },
    Function {
        params: Vec<(String, String)>,
        return_type: String,
    },
    Struct {
        fields: Vec<StructField>,
    },
    Enum {
        variants: Vec<(String, Option<Vec<StructField>>)>,
    },
    TypeAlias {
        ty: String,
    },
}

#[derive(Debug)]
#[derive(Clone)]
pub struct Symbol {
    pub name: String,
    pub kind: SymbolKind,
    pub position: (usize, usize), // (line, column)
}

#[derive(Debug)]
pub struct SymbolTable {
    pub scopes: Vec<HashMap<String, Symbol>>,
    pub current_scope: usize,
}

impl SymbolTable {
    pub fn new() -> Self {
        let mut global_scope = HashMap::new();
        
        // 注册内置类型
        global_scope.insert("int".to_string(), Symbol {
            name: "int".to_string(),
            kind: SymbolKind::TypeAlias { ty: "int".to_string() },
            position: (0, 0),
        });
        
        global_scope.insert("float".to_string(), Symbol {
            name: "float".to_string(),
            kind: SymbolKind::TypeAlias { ty: "float".to_string() },
            position: (0, 0),
        });
        
        global_scope.insert("string".to_string(), Symbol {
            name: "string".to_string(),
            kind: SymbolKind::TypeAlias { ty: "string".to_string() },
            position: (0, 0),
        });
        
        global_scope.insert("bool".to_string(), Symbol {
            name: "bool".to_string(),
            kind: SymbolKind::TypeAlias { ty: "bool".to_string() },
            position: (0, 0),
        });
        
        global_scope.insert("void".to_string(), Symbol {
            name: "void".to_string(),
            kind: SymbolKind::TypeAlias { ty: "void".to_string() },
            position: (0, 0),
        });
        
        global_scope.insert("null".to_string(), Symbol {
            name: "null".to_string(),
            kind: SymbolKind::TypeAlias { ty: "null".to_string() },
            position: (0, 0),
        });
        
        // 注册内置函数
        global_scope.insert("print".to_string(), Symbol {
            name: "print".to_string(),
            kind: SymbolKind::Function {
                params: vec![("value".to_string(), "string".to_string())],
                return_type: "void".to_string(),
            },
            position: (0, 0),
        });
        
        global_scope.insert("println".to_string(), Symbol {
            name: "println".to_string(),
            kind: SymbolKind::Function {
                params: vec![("value".to_string(), "string".to_string())],
                return_type: "void".to_string(),
            },
            position: (0, 0),
        });
        
        global_scope.insert("len".to_string(), Symbol {
            name: "len".to_string(),
            kind: SymbolKind::Function {
                params: vec![("value".to_string(), "string".to_string())],
                return_type: "int".to_string(),
            },
            position: (0, 0),
        });
        
        global_scope.insert("to_string".to_string(), Symbol {
            name: "to_string".to_string(),
            kind: SymbolKind::Function {
                params: vec![("value".to_string(), "int".to_string())],
                return_type: "string".to_string(),
            },
            position: (0, 0),
        });
        
        // 注册List类型
        global_scope.insert("List".to_string(), Symbol {
            name: "List".to_string(),
            kind: SymbolKind::TypeAlias { ty: "List".to_string() },
            position: (0, 0),
        });
        
        // 注册Unit类型
        global_scope.insert("Unit".to_string(), Symbol {
            name: "Unit".to_string(),
            kind: SymbolKind::TypeAlias { ty: "Unit".to_string() },
            position: (0, 0),
        });
        
        Self {
            scopes: vec![global_scope],
            current_scope: 0,
        }
    }
    
    pub fn enter_scope(&mut self) {
        self.scopes.push(HashMap::new());
        self.current_scope += 1;
    }
    
    pub fn exit_scope(&mut self) {
        if self.current_scope > 0 {
            self.scopes.pop();
            self.current_scope -= 1;
        }
    }
    
    pub fn define_symbol(&mut self, symbol: Symbol) -> Result<(), SemanticError> {
        let scope = self.scopes.get_mut(self.current_scope).unwrap();
        
        if scope.contains_key(&symbol.name) {
            return Err(SemanticError::Redeclaration(symbol.name));
        }
        
        scope.insert(symbol.name.clone(), symbol);
        Ok(())
    }
    
    pub fn lookup_symbol(&self, name: &str) -> Option<&Symbol> {
        for i in (0..=self.current_scope).rev() {
            if let Some(symbol) = self.scopes[i].get(name) {
                return Some(symbol);
            }
        }
        None
    }
    
    pub fn lookup_symbol_mut(&mut self, name: &str) -> Option<&mut Symbol> {
        // 先找到包含符号的作用域索引
        let mut found_scope = None;
        for i in (0..=self.current_scope).rev() {
            if self.scopes[i].contains_key(name) {
                found_scope = Some(i);
                break;
            }
        }
        
        if let Some(scope_idx) = found_scope {
            // 只在找到的作用域中获取可变引用
            self.scopes[scope_idx].get_mut(name)
        } else {
            None
        }
    }
    
    pub fn is_type(&self, name: &str) -> bool {
        if let Some(symbol) = self.lookup_symbol(name) {
            matches!(symbol.kind, SymbolKind::TypeAlias { .. } | SymbolKind::Struct { .. } | SymbolKind::Enum { .. })
        } else {
            false
        }
    }
}

pub struct SemanticAnalyzer {
    pub symbol_table: SymbolTable,
    pub borrow_checker: BorrowChecker,
    pub escape_analyzer: EscapeAnalyzer,
    pub loop_optimizer: LoopOptimizer,
    pub errors: Vec<SemanticError>,
    pub current_line: usize,
    pub current_column: usize,
}

impl SemanticAnalyzer {
    pub fn new() -> Self {
        Self {
            symbol_table: SymbolTable::new(),
            borrow_checker: BorrowChecker::new(),
            escape_analyzer: EscapeAnalyzer::new(),
            loop_optimizer: LoopOptimizer::new(),
            errors: Vec::new(),
            current_line: 1,
            current_column: 1,
        }
    }
    
    pub fn analyze(&mut self, program: &ast::Program) -> Result<(), Vec<SemanticError>> {
        // 分析全局作用域
        for stmt in &program.statements {
            self.analyze_statement(stmt)?;
        }
        
        if !self.errors.is_empty() {
            return Err(self.errors.clone());
        }
        
        Ok(())
    }
    
    fn analyze_statement(&mut self, stmt: &ast::Statement) -> Result<(), Vec<SemanticError>> {
        match stmt {
            ast::Statement::Let { mutable, name, ty, value, .. } => {
                self.analyze_let_statement(mutable, name, ty, value)
            },
            ast::Statement::Function { name, params, return_type, body, .. } => {
                self.analyze_function_statement(name, params, return_type, body)
            },
            ast::Statement::Struct { name, fields, .. } => {
                self.analyze_struct_statement(name, fields)
            },
            ast::Statement::Enum { name, variants, .. } => {
                self.analyze_enum_statement(name, variants)
            },
            ast::Statement::Expression(expr) => {
                self.analyze_expression(expr)?;
                Ok(())
            },
            ast::Statement::Return(expr) => {
                self.analyze_return_statement(expr)
            },
            ast::Statement::While { condition, body, .. } => {
                self.analyze_while_statement(condition, body)
            },
            ast::Statement::For { pattern, in_expr, body, .. } => {
                self.analyze_for_statement(pattern, in_expr, body)
            },
            ast::Statement::Import(_) | ast::Statement::ImportAs(_, _) => {
                // 暂时跳过导入语句
                Ok(())
            },
            ast::Statement::Group { name, members, .. } => {
                self.analyze_group_statement(name, members)
            },
        }
    }
    
    fn analyze_let_statement(&mut self, mutable: &bool, name: &str, ty: &Option<String>, value: &ast::Expression) -> Result<(), Vec<SemanticError>> {
        // 分析变量值的类型
        let value_type = self.analyze_expression(value)?;
        
        // 检查是否为引用类型
        let is_ref_type = if let Some(type_str) = ty {
            type_str.starts_with("&") || type_str.starts_with("ref ")
        } else {
            false
        };
        
        // 检查是否取地址
        if let ast::Expression::UnaryOp { op: ast::UnaryOperator::Ref, .. } = value {
            if !is_ref_type {
                self.errors.push(SemanticError::InvalidRef);
                return Err(self.errors.clone());
            }
        }
        
        // 确定变量类型
        let var_type = match ty {
            Some(ty) => {
                if !self.symbol_table.is_type(ty) && !ty.starts_with("&") && !ty.starts_with("ref ") {
                    self.errors.push(SemanticError::InvalidType(ty.clone()));
                    return Err(self.errors.clone());
                }
                ty.clone()
            },
            None => value_type.clone(),
        };
        
        // 检查类型兼容性
        if ty.is_some() && ty.as_ref().unwrap() != &value_type {
            self.errors.push(SemanticError::TypeMismatch {
                expected: ty.as_ref().unwrap().clone(),
                found: value_type.clone(),
            });
            return Err(self.errors.clone());
        }
        
        // 逃逸分析：检查变量是否逃逸
        let context = "global";
        if self.escape_analyzer.should_allocate_on_heap(name, context) {
            self.escape_analyzer.mark_escaped(name, context);
        }
        
        // 定义变量
        let symbol = Symbol {
            name: name.to_string(),
            kind: SymbolKind::Variable {
                mutable: *mutable,
                ty: var_type.clone(),
            },
            position: (self.current_line, self.current_column),
        };
        
        if let Err(err) = self.symbol_table.define_symbol(symbol) {
            self.errors.push(err);
            return Err(self.errors.clone());
        }
        
        // 如果是引用类型，检查借用规则
        if var_type.starts_with("&") || var_type.starts_with("ref ") {
            self.check_borrow_rules(name, mutable);
        }
        
        Ok(())
    }
    
    fn check_borrow_rules(&mut self, _name: &str, _mutable: &bool) {
        // 检查可变借用和不可变借用的冲突
        // 这里简化处理，未来可以增强
    }
    
    fn analyze_function_statement(&mut self, name: &str, params: &[ast::Parameter], return_type: &Option<String>, body: &ast::Expression) -> Result<(), Vec<SemanticError>> {
        // 检查返回类型是否有效
        let ret_ty = return_type.clone().unwrap_or_else(|| "void".to_string());
        if !self.symbol_table.is_type(&ret_ty) {
            self.errors.push(SemanticError::InvalidType(ret_ty.clone()));
            return Err(self.errors.clone());
        }
        
        // 检查参数类型是否有效
        let mut param_types = Vec::new();
        for param in params {
            if let Some(ty) = &param.ty {
                if !self.symbol_table.is_type(ty) {
                    self.errors.push(SemanticError::InvalidType(ty.clone()));
                    return Err(self.errors.clone());
                }
                param_types.push((param.name.clone(), ty.clone()));
            } else {
                // 参数必须有类型
                self.errors.push(SemanticError::InvalidType("undefined".to_string()));
                return Err(self.errors.clone());
            }
        }
        
        // 定义函数
        let symbol = Symbol {
            name: name.to_string(),
            kind: SymbolKind::Function {
                params: param_types,
                return_type: ret_ty.clone(),
            },
            position: (self.current_line, self.current_column),
        };
        
        if let Err(err) = self.symbol_table.define_symbol(symbol) {
            self.errors.push(err);
            return Err(self.errors.clone());
        }
        
        // 进入函数作用域
        self.symbol_table.enter_scope();
        
        // 定义参数
        for param in params {
            let param_symbol = Symbol {
                name: param.name.clone(),
                kind: SymbolKind::Variable {
                    mutable: false,
                    ty: param.ty.as_ref().unwrap().clone(),
                },
                position: (self.current_line, self.current_column),
            };
            
            if let Err(err) = self.symbol_table.define_symbol(param_symbol) {
                self.errors.push(err);
                self.symbol_table.exit_scope();
                return Err(self.errors.clone());
            }
        }
        
        // 分析函数体
        let body_type = self.analyze_expression(body)?;
        
        // 检查返回类型
        if ret_ty != "void" && ret_ty != body_type {
            self.errors.push(SemanticError::TypeMismatch {
                expected: ret_ty.clone(),
                found: body_type.clone(),
            });
        }
        
        // 退出函数作用域
        self.symbol_table.exit_scope();
        
        Ok(())
    }
    
    fn analyze_struct_statement(&mut self, name: &str, fields: &[ast::StructField]) -> Result<(), Vec<SemanticError>> {
        // 检查字段类型是否有效
        for field in fields {
            if !self.symbol_table.is_type(&field.ty) {
                self.errors.push(SemanticError::InvalidType(field.ty.clone()));
                return Err(self.errors.clone());
            }
        }
        
        // 定义结构体
        let symbol = Symbol {
            name: name.to_string(),
            kind: SymbolKind::Struct {
                fields: fields.to_vec(),
            },
            position: (self.current_line, self.current_column),
        };
        
        if let Err(err) = self.symbol_table.define_symbol(symbol) {
            self.errors.push(err);
            return Err(self.errors.clone());
        }
        
        Ok(())
    }
    
    fn analyze_enum_statement(&mut self, name: &str, variants: &[(String, Option<Vec<ast::StructField>>)]) -> Result<(), Vec<SemanticError>> {
        // 检查变体字段类型是否有效
        for (_variant_name, fields) in variants {
            if let Some(fields) = fields {
                for field in fields {
                    if !self.symbol_table.is_type(&field.ty) {
                        self.errors.push(SemanticError::InvalidType(field.ty.clone()));
                        return Err(self.errors.clone());
                    }
                }
            }
        }
        
        // 定义枚举
        let symbol = Symbol {
            name: name.to_string(),
            kind: SymbolKind::Enum {
                variants: variants.to_vec(),
            },
            position: (self.current_line, self.current_column),
        };
        
        if let Err(err) = self.symbol_table.define_symbol(symbol) {
            self.errors.push(err);
            return Err(self.errors.clone());
        }
        
        Ok(())
    }
    
    fn analyze_return_statement(&mut self, _expr: &Option<ast::Expression>) -> Result<(), Vec<SemanticError>> {
        // 暂时跳过返回语句的分析，需要知道当前函数的返回类型
        Ok(())
    }
    
    fn analyze_while_statement(&mut self, condition: &ast::Expression, body: &ast::Expression) -> Result<(), Vec<SemanticError>> {
        // 分析条件表达式，必须是bool类型
        let condition_type = self.analyze_expression(condition)?;
        if condition_type != "bool" {
            self.errors.push(SemanticError::TypeMismatch {
                expected: "bool".to_string(),
                found: condition_type.clone(),
            });
        }
        
        // 进入循环作用域
        self.symbol_table.enter_scope();
        
        // 分析循环体
        self.analyze_expression(body)?;
        
        // 退出循环作用域
        self.symbol_table.exit_scope();
        
        Ok(())
    }
    
    fn analyze_for_statement(&mut self, pattern: &str, in_expr: &ast::Expression, body: &ast::Expression) -> Result<(), Vec<SemanticError>> {
        // 分析in表达式
        self.analyze_expression(in_expr)?;
        
        // 进入循环作用域
        self.symbol_table.enter_scope();
        
        // 定义循环变量
        let loop_var = Symbol {
            name: pattern.to_string(),
            kind: SymbolKind::Variable {
                mutable: true,
                ty: "auto".to_string(), // 暂时使用auto类型，后续需要推断
            },
            position: (self.current_line, self.current_column),
        };
        
        if let Err(err) = self.symbol_table.define_symbol(loop_var) {
            self.errors.push(err);
            self.symbol_table.exit_scope();
            return Err(self.errors.clone());
        }
        
        // 分析循环体
        self.analyze_expression(body)?;
        
        // 退出循环作用域
        self.symbol_table.exit_scope();
        
        Ok(())
    }
    
    fn analyze_group_statement(&mut self, _name: &str, members: &[ast::Statement]) -> Result<(), Vec<SemanticError>> {
        // 进入组作用域
        self.symbol_table.enter_scope();
        
        // 分析组成员
        for member in members {
            self.analyze_statement(member)?;
        }
        
        // 退出组作用域
        self.symbol_table.exit_scope();
        
        Ok(())
    }
    
    fn analyze_expression(&mut self, expr: &ast::Expression) -> Result<String, Vec<SemanticError>> {
        match expr {
            ast::Expression::Literal(lit) => {
                self.analyze_literal(lit)
            },
            ast::Expression::Identifier(name) => {
                self.analyze_identifier(name)
            },
            ast::Expression::UnaryOp { op, expr } => {
                self.analyze_unary_op(op, expr)
            },
            ast::Expression::BinaryOp { left, op, right } => {
                self.analyze_binary_op(left, op, right)
            },
            ast::Expression::Call { callee, args } => {
                self.analyze_call(callee, args)
            },
            ast::Expression::Index { array, index } => {
                self.analyze_index(array, index)
            },
            ast::Expression::FieldAccess { expr, field } => {
                self.analyze_field_access(expr, field)
            },
            ast::Expression::Assign { left, right } => {
                self.analyze_assign(left, right)
            },
            ast::Expression::Block(stmts) => {
                self.analyze_block(stmts)
            },
            ast::Expression::If { condition, then_branch, else_branch } => {
                self.analyze_if_expression(condition, then_branch, else_branch)
            },
            ast::Expression::Match { expr, cases } => {
                self.analyze_match_expression(expr, cases)
            },
            ast::Expression::Lambda { params, body } => {
                self.analyze_lambda(params, body)
            },
            ast::Expression::Range { start, end, inclusive } => {
                self.analyze_range(start, end, *inclusive)
            },
            ast::Expression::Array(items) => {
                self.analyze_array(items)
            },
            ast::Expression::Struct { name, fields } => {
                self.analyze_struct_literal(name, fields)
            },
            ast::Expression::Group { name: _name, members: _members } => {
                // 暂时跳过组表达式的分析
                Ok("group".to_string())
            },
        }
    }
    
    fn analyze_literal(&self, lit: &ast::Literal) -> Result<String, Vec<SemanticError>> {
        match lit {
            Literal::Integer(_) => Ok("int".to_string()),
            Literal::Float(_) => Ok("float".to_string()),
            Literal::String(_) => Ok("string".to_string()),
            Literal::Boolean(_) => Ok("bool".to_string()),
            Literal::Null => Ok("null".to_string()),
            Literal::UnitLiteral(_, unit) => Ok(format!("Unit[float, {}]", unit)),
        }
    }
    
    fn analyze_identifier(&mut self, name: &str) -> Result<String, Vec<SemanticError>> {
        if let Some(symbol) = self.symbol_table.lookup_symbol(name) {
            match &symbol.kind {
                SymbolKind::Variable { ty, .. } => Ok(ty.clone()),
                SymbolKind::Function { return_type, .. } => Ok(return_type.clone()),
                SymbolKind::Struct { .. } => Ok(name.to_string()),
                SymbolKind::Enum { .. } => Ok(name.to_string()),
                SymbolKind::TypeAlias { ty } => Ok(ty.clone()),
            }
        } else {
            self.errors.push(SemanticError::UndefinedIdentifier(name.to_string()));
            Err(self.errors.clone())
        }
    }
    
    fn analyze_unary_op(&mut self, op: &ast::UnaryOperator, expr: &ast::Expression) -> Result<String, Vec<SemanticError>> {
        let expr_type = self.analyze_expression(expr)?;
        
        match op {
            ast::UnaryOperator::Not => {
                // Not操作符只能用于bool类型
                if expr_type != "bool" {
                    self.errors.push(SemanticError::TypeMismatch {
                        expected: "bool".to_string(),
                        found: expr_type.clone(),
                    });
                    return Err(self.errors.clone());
                }
                Ok("bool".to_string())
            },
            ast::UnaryOperator::Neg => {
                // Neg操作符只能用于数值类型
                if expr_type != "int" && expr_type != "float" {
                    self.errors.push(SemanticError::TypeMismatch {
                        expected: "int or float".to_string(),
                        found: expr_type.clone(),
                    });
                    return Err(self.errors.clone());
                }
                Ok(expr_type)
            },
            ast::UnaryOperator::Ref => {
                // Ref操作符返回引用类型
                Ok(format!("&{}", expr_type))
            },
            ast::UnaryOperator::Deref => {
                // Deref操作符只能用于引用类型
                if expr_type.starts_with("&") {
                    Ok(expr_type[1..].to_string())
                } else {
                    self.errors.push(SemanticError::InvalidDeref);
                    Err(self.errors.clone())
                }
            },
        }
    }
    
    fn analyze_binary_op(&mut self, left: &ast::Expression, op: &ast::BinaryOperator, right: &ast::Expression) -> Result<String, Vec<SemanticError>> {
        let left_type = self.analyze_expression(left)?;
        let right_type = self.analyze_expression(right)?;
        
        match op {
            // 算术操作符
            ast::BinaryOperator::Add => {
                if left_type == "string" && right_type == "string" {
                    // 字符串拼接
                    Ok("string".to_string())
                } else if (left_type == "int" || left_type == "float") && (right_type == "int" || right_type == "float") {
                    // 数值加法
                    if left_type == "float" || right_type == "float" {
                        Ok("float".to_string())
                    } else {
                        Ok("int".to_string())
                    }
                } else {
                    self.errors.push(SemanticError::TypeMismatch {
                        expected: "int, float, or string".to_string(),
                        found: format!("{}, {}", left_type, right_type),
                    });
                    Err(self.errors.clone())
                }
            },
            ast::BinaryOperator::Sub | ast::BinaryOperator::Mul | ast::BinaryOperator::Div | ast::BinaryOperator::Mod => {
                // 算术操作符只能用于数值类型
                if (left_type == "int" || left_type == "float") && (right_type == "int" || right_type == "float") {
                    if left_type == "float" || right_type == "float" {
                        Ok("float".to_string())
                    } else {
                        Ok("int".to_string())
                    }
                } else {
                    self.errors.push(SemanticError::TypeMismatch {
                        expected: "int or float".to_string(),
                        found: format!("{}, {}", left_type, right_type),
                    });
                    Err(self.errors.clone())
                }
            },
            // 比较操作符
            ast::BinaryOperator::Eq | ast::BinaryOperator::Ne | ast::BinaryOperator::Lt | ast::BinaryOperator::Le | ast::BinaryOperator::Gt | ast::BinaryOperator::Ge => {
                // 比较操作符返回bool类型
                if left_type == right_type {
                    Ok("bool".to_string())
                } else {
                    self.errors.push(SemanticError::TypeMismatch {
                        expected: left_type.clone(),
                        found: right_type.clone(),
                    });
                    Err(self.errors.clone())
                }
            },
            // 逻辑操作符
            ast::BinaryOperator::And | ast::BinaryOperator::Or => {
                // 逻辑操作符只能用于bool类型
                if left_type == "bool" && right_type == "bool" {
                    Ok("bool".to_string())
                } else {
                    self.errors.push(SemanticError::TypeMismatch {
                        expected: "bool".to_string(),
                        found: format!("{}, {}", left_type, right_type),
                    });
                    Err(self.errors.clone())
                }
            },
            // 范围操作符
            ast::BinaryOperator::Range | ast::BinaryOperator::RangeInclusive => {
                // 范围操作符返回range类型
                if left_type == right_type && (left_type == "int" || left_type == "float") {
                    Ok("range".to_string())
                } else {
                    self.errors.push(SemanticError::TypeMismatch {
                        expected: "int or float".to_string(),
                        found: format!("{}, {}", left_type, right_type),
                    });
                    Err(self.errors.clone())
                }
            },
            // 赋值操作符 - 复合赋值已经在parser中转换为二元操作符+赋值
            // 这里只处理简单赋值，复合赋值会作为BinaryOp出现
            ast::BinaryOperator::Assign => {
                // 这条路径不应该被执行，因为赋值操作符应该通过Assign表达式处理
                self.errors.push(SemanticError::InvalidSyntax("赋值操作符不能直接用于二元操作".to_string()));
                Err(self.errors.clone())
            },
            ast::BinaryOperator::AddAssign | ast::BinaryOperator::SubAssign | ast::BinaryOperator::MulAssign | ast::BinaryOperator::DivAssign | ast::BinaryOperator::ModAssign => {
                // 复合赋值操作符已经在parser中转换为二元操作符
                // 需要分析二元操作的结果类型
                let result_type = if left_type == "int" || left_type == "float" {
                    left_type.clone()
                } else {
                    left_type.clone()
                };
                Ok(result_type)
            },
        }
    }
    
    fn analyze_call(&mut self, callee: &ast::Expression, args: &[ast::Expression]) -> Result<String, Vec<SemanticError>> {
        // 分析函数名
        let callee_type = self.analyze_expression(callee)?;
        
        // 检查是否为函数
        if let ast::Expression::Identifier(name) = callee {
            // 先获取函数信息，避免借用冲突
            let (param_types, return_type) = {
                let symbol = if let Some(symbol) = self.symbol_table.lookup_symbol(name) {
                    symbol.clone()
                } else {
                    self.errors.push(SemanticError::UndefinedIdentifier(name.to_string()));
                    return Err(self.errors.clone());
                };
                
                if let SymbolKind::Function { params, return_type } = &symbol.kind {
                    let param_types = params.iter().map(|(_, ty)| ty.clone()).collect::<Vec<_>>();
                    (param_types, return_type.clone())
                } else {
                    self.errors.push(SemanticError::NotAFunction(name.to_string()));
                    return Err(self.errors.clone());
                }
            };
            
            // 检查参数数量
            if param_types.len() != args.len() {
                self.errors.push(SemanticError::WrongArgumentCount {
                    name: name.to_string(),
                    expected: param_types.len(),
                    found: args.len(),
                });
                return Err(self.errors.clone());
            }
            
            // 检查参数类型
            for (i, (arg, expected_type)) in args.iter().zip(param_types.iter()).enumerate() {
                let arg_type = self.analyze_expression(arg)?;
                if arg_type != *expected_type {
                    self.errors.push(SemanticError::WrongArgumentType {
                        name: name.to_string(),
                        arg_idx: i,
                        expected: expected_type.clone(),
                        found: arg_type.clone(),
                    });
                    return Err(self.errors.clone());
                }
            }
            
            return Ok(return_type);
        }
        
        self.errors.push(SemanticError::NotAFunction(callee_type.clone()));
        Err(self.errors.clone())
    }
    
    fn analyze_index(&mut self, array: &ast::Expression, index: &ast::Expression) -> Result<String, Vec<SemanticError>> {
        let _array_type = self.analyze_expression(array)?;
        let index_type = self.analyze_expression(index)?;
        
        // 检查索引类型
        if index_type != "int" {
            self.errors.push(SemanticError::TypeMismatch {
                expected: "int".to_string(),
                found: index_type.clone(),
            });
            return Err(self.errors.clone());
        }
        
        // 暂时返回auto类型，后续需要根据数组类型推断
        Ok("auto".to_string())
    }
    
    fn analyze_field_access(&mut self, expr: &ast::Expression, field: &str) -> Result<String, Vec<SemanticError>> {
        let expr_type = self.analyze_expression(expr)?;
        
        // 检查是否为结构体
        if let Some(symbol) = self.symbol_table.lookup_symbol(&expr_type) {
            if let SymbolKind::Struct { fields } = &symbol.kind {
                // 检查字段是否存在
                for struct_field in fields {
                    if struct_field.name == *field {
                        return Ok(struct_field.ty.clone());
                    }
                }
                
                self.errors.push(SemanticError::UndefinedField {
                    struct_name: expr_type.clone(),
                    field_name: field.to_string(),
                });
                return Err(self.errors.clone());
            }
        }
        
        self.errors.push(SemanticError::InvalidDotOperator(expr_type.clone()));
        Err(self.errors.clone())
    }
    
    fn analyze_assign(&mut self, left: &ast::Expression, right: &ast::Expression) -> Result<String, Vec<SemanticError>> {
        let right_type = self.analyze_expression(right)?;
        
        match left {
            ast::Expression::Identifier(name) => {
                // 检查变量是否存在
                if let Some(symbol) = self.symbol_table.lookup_symbol(name) {
                    if let SymbolKind::Variable { mutable, ty } = &symbol.kind {
                        // 检查变量是否可变
                        if !*mutable {
                            self.errors.push(SemanticError::CannotAssignToImmutable(name.to_string()));
                            return Err(self.errors.clone());
                        }
                        
                        // 检查类型兼容性
                        if ty != &right_type && ty != "auto" {
                            self.errors.push(SemanticError::TypeMismatch {
                                expected: ty.clone(),
                                found: right_type.clone(),
                            });
                            return Err(self.errors.clone());
                        }
                        
                        return Ok(right_type);
                    }
                }
                
                self.errors.push(SemanticError::UndefinedIdentifier(name.to_string()));
                Err(self.errors.clone())
            },
            ast::Expression::FieldAccess { expr: struct_expr, field } => {
                let struct_type = self.analyze_expression(struct_expr)?;
                
                // 检查结构体是否存在
                if let Some(symbol) = self.symbol_table.lookup_symbol(&struct_type) {
                    if let SymbolKind::Struct { fields } = &symbol.kind {
                        // 检查字段是否存在
                        for struct_field in fields {
                            if struct_field.name == *field {
                                // 检查类型兼容性
                                if struct_field.ty != right_type {
                                    self.errors.push(SemanticError::TypeMismatch {
                                        expected: struct_field.ty.clone(),
                                        found: right_type.clone(),
                                    });
                                    return Err(self.errors.clone());
                                }
                                
                                return Ok(right_type);
                            }
                        }
                        
                        self.errors.push(SemanticError::UndefinedField {
                            struct_name: struct_type.clone(),
                            field_name: field.to_string(),
                        });
                        return Err(self.errors.clone());
                    }
                }
                
                self.errors.push(SemanticError::UndefinedStruct(struct_type.clone()));
                Err(self.errors.clone())
            },
            ast::Expression::Index { array, index } => {
                let _array_type = self.analyze_expression(array)?;
                let index_type = self.analyze_expression(index)?;
                
                // 检查索引类型
                if index_type != "int" {
                    self.errors.push(SemanticError::TypeMismatch {
                        expected: "int".to_string(),
                        found: index_type.clone(),
                    });
                    return Err(self.errors.clone());
                }
                
                // 暂时返回right_type，后续需要根据数组类型检查
                Ok(right_type)
            },
            _ => {
                self.errors.push(SemanticError::InvalidSyntax("赋值操作符左侧必须是可修改的表达式".to_string()));
                Err(self.errors.clone())
            },
        }
    }
    
    fn analyze_block(&mut self, stmts: &[ast::Statement]) -> Result<String, Vec<SemanticError>> {
        // 进入块作用域
        self.symbol_table.enter_scope();
        
        // 分析块内语句
        let mut last_type = "void".to_string();
        for stmt in stmts {
            match stmt {
                ast::Statement::Expression(expr) => {
                    last_type = self.analyze_expression(expr)?;
                },
                ast::Statement::Return(expr) => {
                    // 暂时跳过返回语句的分析
                    if let Some(expr) = expr {
                        last_type = self.analyze_expression(expr)?;
                    }
                },
                _ => {
                    self.analyze_statement(stmt)?;
                },
            }
        }
        
        // 退出块作用域
        self.symbol_table.exit_scope();
        
        Ok(last_type)
    }
    
    fn analyze_if_expression(&mut self, condition: &ast::Expression, then_branch: &ast::Expression, else_branch: &Option<Box<ast::Expression>>) -> Result<String, Vec<SemanticError>> {
        // 分析条件表达式，必须是bool类型
        let condition_type = self.analyze_expression(condition)?;
        if condition_type != "bool" {
            self.errors.push(SemanticError::TypeMismatch {
                expected: "bool".to_string(),
                found: condition_type.clone(),
            });
        }
        
        // 进入then分支作用域
        self.symbol_table.enter_scope();
        
        // 分析then分支
        let then_type = self.analyze_expression(then_branch)?;
        
        // 退出then分支作用域
        self.symbol_table.exit_scope();
        
        // 进入else分支作用域
        self.symbol_table.enter_scope();
        
        // 分析else分支
        let else_type = if let Some(else_branch) = else_branch {
            self.analyze_expression(else_branch)?
        } else {
            "void".to_string()
        };
        
        // 退出else分支作用域
        self.symbol_table.exit_scope();
        
        // 检查then和else分支的类型是否兼容
        if then_type == else_type {
            Ok(then_type)
        } else if then_type == "void" {
            Ok(else_type)
        } else if else_type == "void" {
            Ok(then_type)
        } else {
            self.errors.push(SemanticError::TypeMismatch {
                expected: then_type.clone(),
                found: else_type.clone(),
            });
            Err(self.errors.clone())
        }
    }
    
    fn analyze_match_expression(&mut self, expr: &ast::Expression, cases: &[ast::MatchCase]) -> Result<String, Vec<SemanticError>> {
        // 分析匹配表达式
        let expr_type = self.analyze_expression(expr)?;
        
        // 检查是否为枚举类型
        if !self.symbol_table.is_type(&expr_type) {
            self.errors.push(SemanticError::TypeMismatch {
                expected: "enum or struct".to_string(),
                found: expr_type.clone(),
            });
            return Err(self.errors.clone());
        }
        
        // 检查所有分支的类型是否一致
        let mut case_types = Vec::new();
        for case in cases {
            // 分析模式
            self.analyze_pattern(&case.pattern, &expr_type)?;
            
            // 分析守卫条件
            if let Some(guard) = &case.guard {
                let guard_type = self.analyze_expression(guard)?;
                if guard_type != "bool" {
                    self.errors.push(SemanticError::TypeMismatch {
                        expected: "bool".to_string(),
                        found: guard_type.clone(),
                    });
                    return Err(self.errors.clone());
                }
            }
            
            // 分析分支体
            let case_type = self.analyze_expression(&case.body)?;
            case_types.push(case_type);
        }
        
        // 检查所有分支的类型是否一致
        if case_types.is_empty() {
            self.errors.push(SemanticError::NonExhaustiveMatch);
            return Err(self.errors.clone());
        }
        
        let expected_type = &case_types[0];
        for case_type in &case_types {
            if case_type != expected_type {
                self.errors.push(SemanticError::TypeMismatch {
                    expected: expected_type.clone(),
                    found: case_type.clone(),
                });
                return Err(self.errors.clone());
            }
        }
        
        Ok(expected_type.clone())
    }
    
    fn analyze_pattern(&mut self, pattern: &ast::Expression, expected_type: &str) -> Result<(), Vec<SemanticError>> {
        // 分析模式
        match pattern {
            ast::Expression::Identifier(name) => {
                if name == "_" {
                    // 通配符，不检查
                    Ok(())
                } else {
                    // 变量绑定，检查类型
                    // 暂时跳过，后续需要实现
                    Ok(())
                }
            },
            ast::Expression::Literal(_) => {
                // 字面量模式，检查类型
                let pattern_type = self.analyze_expression(pattern)?;
                if pattern_type != expected_type {
                    self.errors.push(SemanticError::TypeMismatch {
                        expected: expected_type.to_string(),
                        found: pattern_type.clone(),
                    });
                    Err(self.errors.clone())
                } else {
                    Ok(())
                }
            },
            ast::Expression::Call { callee, args } => {
                // 枚举变体模式，如 Status::Success(msg)
                if let ast::Expression::Identifier(variant_path) = &**callee {
                    // 解析枚举名和变体名
                    if let Some((enum_name, variant_name)) = variant_path.split_once("::") {
                        // 先克隆enum symbol，避免借用冲突
                        let enum_symbol = if let Some(symbol) = self.symbol_table.lookup_symbol(enum_name) {
                            symbol.clone()
                        } else {
                            self.errors.push(SemanticError::UndefinedIdentifier(enum_name.to_string()));
                            return Err(self.errors.clone());
                        };
                        
                        if let SymbolKind::Enum { variants } = &enum_symbol.kind {
                            // 检查变体是否存在
                            let mut found_variant = false;
                            let mut variant_fields = None;
                            
                            for (name, fields) in variants {
                                if name == variant_name {
                                    found_variant = true;
                                    variant_fields = fields.clone();
                                    break;
                                }
                            }
                            
                            if !found_variant {
                                self.errors.push(SemanticError::UndefinedVariant {
                                    enum_name: enum_name.to_string(),
                                    variant_name: variant_name.to_string(),
                                });
                                return Err(self.errors.clone());
                            }
                            
                            // 检查变体字段数量
                            if let Some(variant_fields) = variant_fields {
                                if variant_fields.len() != args.len() {
                                    self.errors.push(SemanticError::WrongArgumentCount {
                                        name: variant_path.to_string(),
                                        expected: variant_fields.len(),
                                        found: args.len(),
                                    });
                                    return Err(self.errors.clone());
                                }
                                
                                // 检查变体字段类型
                                for (arg, field) in args.iter().zip(variant_fields.iter()) {
                                    // 分析参数（模式）
                                    self.analyze_pattern(arg, &field.ty)?;
                                }
                            }
                            
                            return Ok(());
                        }
                    }
                }
                
                self.errors.push(SemanticError::InvalidMatchPattern);
                Err(self.errors.clone())
            },
            _ => {
                self.errors.push(SemanticError::InvalidPattern(format!("{:?}", pattern)));
                Err(self.errors.clone())
            },
        }
    }
    
    fn analyze_lambda(&mut self, params: &[ast::Parameter], body: &ast::Expression) -> Result<String, Vec<SemanticError>> {
        // 进入lambda作用域
        self.symbol_table.enter_scope();
        
        // 定义参数
        for param in params {
            if let Some(ty) = &param.ty {
                if !self.symbol_table.is_type(ty) {
                    self.errors.push(SemanticError::InvalidType(ty.clone()));
                    self.symbol_table.exit_scope();
                    return Err(self.errors.clone());
                }
                
                let param_symbol = Symbol {
                    name: param.name.clone(),
                    kind: SymbolKind::Variable {
                        mutable: false,
                        ty: ty.clone(),
                    },
                    position: (self.current_line, self.current_column),
                };
                
                if let Err(err) = self.symbol_table.define_symbol(param_symbol) {
                    self.errors.push(err);
                    self.symbol_table.exit_scope();
                    return Err(self.errors.clone());
                }
            } else {
                // 参数必须有类型
                self.errors.push(SemanticError::InvalidType("undefined".to_string()));
                self.symbol_table.exit_scope();
                return Err(self.errors.clone());
            }
        }
        
        // 分析lambda体
        let body_type = self.analyze_expression(body)?;
        
        // 退出lambda作用域
        self.symbol_table.exit_scope();
        
        Ok(format!("lambda -> {}", body_type))
    }
    
    fn analyze_range(&mut self, start: &ast::Expression, end: &ast::Expression, _inclusive: bool) -> Result<String, Vec<SemanticError>> {
        let start_type = self.analyze_expression(start)?;
        let end_type = self.analyze_expression(end)?;
        
        // 检查start和end的类型是否一致
        if start_type != end_type {
            self.errors.push(SemanticError::TypeMismatch {
                expected: start_type.clone(),
                found: end_type.clone(),
            });
            return Err(self.errors.clone());
        }
        
        // 检查是否为数值类型
        if start_type != "int" && start_type != "float" {
            self.errors.push(SemanticError::TypeMismatch {
                expected: "int or float".to_string(),
                found: start_type.clone(),
            });
            return Err(self.errors.clone());
        }
        
        Ok("range".to_string())
    }
    
    fn analyze_array(&mut self, items: &[ast::Expression]) -> Result<String, Vec<SemanticError>> {
        if items.is_empty() {
            // 空数组，暂时返回auto类型
            Ok("auto".to_string())
        } else {
            // 检查所有元素的类型是否一致
            let first_type = self.analyze_expression(&items[0])?;
            for item in &items[1..] {
                let item_type = self.analyze_expression(item)?;
                if item_type != first_type {
                    self.errors.push(SemanticError::TypeMismatch {
                        expected: first_type.clone(),
                        found: item_type.clone(),
                    });
                    return Err(self.errors.clone());
                }
            }
            
            // 返回List[type]类型
            Ok(format!("List[{}]", first_type))
        }
    }
    
    fn analyze_struct_literal(&mut self, name: &str, fields: &[(String, ast::Expression)]) -> Result<String, Vec<SemanticError>> {
        // 先获取结构体字段信息，避免借用冲突
        let struct_field_map = {
            let symbol = if let Some(symbol) = self.symbol_table.lookup_symbol(name) {
                symbol.clone()
            } else {
                self.errors.push(SemanticError::UndefinedStruct(name.to_string()));
                return Err(self.errors.clone());
            };
            
            if let SymbolKind::Struct { fields: struct_fields } = &symbol.kind {
                let mut field_map = std::collections::HashMap::new();
                for field in struct_fields {
                    field_map.insert(field.name.clone(), field.ty.clone());
                }
                field_map
            } else {
                self.errors.push(SemanticError::NotAType(name.to_string()));
                return Err(self.errors.clone());
            }
        };
        
        // 检查字段数量
        if struct_field_map.len() != fields.len() {
            self.errors.push(SemanticError::WrongArgumentCount {
                name: name.to_string(),
                expected: struct_field_map.len(),
                found: fields.len(),
            });
            return Err(self.errors.clone());
        }
        
        // 检查字段名和类型
        for (field_name, field_value) in fields {
            if let Some(expected_type) = struct_field_map.get(field_name) {
                // 分析字段值
                let field_value_type = self.analyze_expression(field_value)?;
                
                // 检查字段类型
                if field_value_type != *expected_type {
                    self.errors.push(SemanticError::TypeMismatch {
                        expected: expected_type.clone(),
                        found: field_value_type.clone(),
                    });
                    return Err(self.errors.clone());
                }
            } else {
                self.errors.push(SemanticError::UndefinedField {
                    struct_name: name.to_string(),
                    field_name: field_name.clone(),
                });
                return Err(self.errors.clone());
            }
        }
        
        return Ok(name.to_string());
    }
}
