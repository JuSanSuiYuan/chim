// ==================== 递归终止检查器 ====================
// 参考 Agda 实现，支持终止性检查、递归调用分析

pub mod termination {
    use crate::stdlib::prelude::{Option, Result, Vec, HashMap, HashSet, String, StringBuilder};
    use crate::stdlib::string::String as StdString;

    // ==================== 递归调用 ====================

    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct RecursiveCall {
        pub callee: string,           // 被调用的函数名
        pub args: Vec<CallArg>,       // 调用参数
        pub position: Position,       // 源码位置
        pub is_tail_call: bool,       // 是否是尾调用
    }

    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct CallArg {
        pub expr: Expression,
        pub size_change: Option<SizeChange>,  // 大小变化
    }

    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub enum SizeChange {
        Decreased,       // 减小（如 n-1）
        Increased,       // 增加
        Unchanged,       // 不变
        Unknown,         // 未知
    }

    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct Position {
        pub file: string,
        pub line: int,
        pub column: int,
    }

    // ==================== 表达式类型 ====================

    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub enum Expression {
        Variable(string),                      // x
        Literal(Literal),
        Constructor(string, Vec<Expression>),  // C(e1, e2, ...)
        FieldAccess(Box<Expression>, string),  // e.field
        Index(Box<Expression>, Box<Expression>),  // e[i]
        Tuple(Vec<Expression>),
        Array(Vec<Expression>),
        Call(string, Vec<Expression>),
        BinOp(BinOp, Box<Expression>, Box<Expression>),
        UnOp(UnOp, Box<Expression>),
        If(Box<Expression>, Box<Expression>, Option<Box<Expression>>),
        Match(Box<Expression>, Vec<MatchArm>),
        Block(Vec<Statement>, Option<Box<Expression>>),
        // ... 其他表达式
    }

    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub enum Literal {
        Unit,
        Bool(bool),
        Int(i128),
        Float(f64),
        Char(char),
        String(StdString),
    }

    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub enum BinOp {
        Add, Sub, Mul, Div, Mod,
        Eq, Ne, Lt, Le, Gt, Ge,
        And, Or,
        BitAnd, BitOr, BitXor,
        Shl, Shr,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub enum UnOp {
        Neg, Not, BitNot,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct MatchArm {
        pub pattern: Pattern,
        pub body: Expression,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub enum Pattern {
        Variable(string),
        Wildcard,
        Constructor(string, Vec<Pattern>),
        Tuple(Vec<Pattern>),
        Literal(Literal),
        Or(Vec<Pattern>),
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct Statement {
        pub kind: StatementKind,
        pub position: Position,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum StatementKind {
        Let(Pattern, Option<Type>, Expression),
        Expression(Expression),
        While(Expression, Expression),
        For(Option<Statement>, Option<Expression>, Option<Expression>, Expression),
        Break(Option<string>),
        Continue(Option<string>),
        Return(Option<Expression>),
    }

    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct Type {
        pub kind: TypeKind,
        pub size: Option<int>,  // 用于大小类型分析
    }

    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub enum TypeKind {
        Unit,
        Bool,
        Int(Option<usize>),
        Float,
        Char,
        String,
        Array(Box<Type>, Option<int>),
        Tuple(Vec<Type>),
        Named(string),
        Enum(string),
        Record(string),
        Function(Box<Type>, Box<Type>),
        Var(string),
    }

    // ==================== 函数定义 ====================

    #[derive(Debug, Clone)]
    pub struct FunctionDef {
        pub name: string,
        pub params: Vec<string>,           // 参数名
        pub param_types: Vec<Type>,        // 参数类型
        pub return_type: Type,
        pub body: Expression,
        pub termination: TerminationStatus,
        pub is_recursive: bool,
        pub calls_self: Vec<RecursiveCall>,
        pub position: Position,
    }

    #[derive(Debug, Clone, PartialEq)]
    pub enum TerminationStatus {
        Proving,         // 正在证明
        Proved,          // 已证明终止
        NonTerminating,  // 不终止
        Unknown,         // 未知
        Structural,      // 结构递归（自动终止）
        SizeChange(Vec<SizeChange>),  // 规模递减证明
    }

    impl Default for TerminationStatus {
        fn default() -> Self {
            TerminationStatus::Unknown
        }
    }

    // ==================== 终止检查器 ====================

    pub struct TerminationChecker {
        functions: HashMap<string, FunctionDef>,
        current_function: Option<string>,
        call_graph: CallGraph,
        recursion_depth: int,
        max_recursion_depth: int,
    }

    #[derive(Debug, Clone)]
    pub struct CallGraph {
        edges: Vec<(string, string)>,  // caller -> callee
        recursive_calls: HashMap<string, Vec<RecursiveCall>>,
    }

    impl CallGraph {
        pub fn new() -> Self {
            CallGraph {
                edges: Vec::new(),
                recursive_calls: HashMap::new(),
            }
        }

        pub fn add_edge(&mut self, caller: &string, callee: &string) {
            if caller != callee {
                self.edges.push((caller.clone(), callee.clone()));
            }
        }

        pub fn add_recursive_call(&mut self, func: &string, call: RecursiveCall) {
            self.recursive_calls
                .entry(func.clone())
                .or_insert_with(Vec::new)
                .push(call);
        }
    }

    impl TerminationChecker {
        pub fn new() -> Self {
            TerminationChecker {
                functions: HashMap::new(),
                current_function: None,
                call_graph: CallGraph::new(),
                recursion_depth: 0,
                max_recursion_depth: 1000,
            }
        }

        /// 注册函数
        pub fn register_function(&mut self, func: FunctionDef) {
            self.functions.insert(func.name.clone(), func);
        }

        /// 检查所有函数的终止性
        pub fn check_all(&mut self) -> Vec<TerminationError> {
            let mut errors = Vec::new();
            
            for (name, func) in &self.functions {
                self.current_function = Some(name.clone());
                if let Err(e) = self.check_termination(name, func) {
                    errors.push(e);
                }
            }
            
            errors
        }

        /// 检查单个函数的终止性
        fn check_termination(&mut self, name: &string, func: &FunctionDef) -> Result<TerminationStatus, TerminationError> {
            // 1. 检测递归调用
            let recursive_calls = self.find_recursive_calls(&func.body, name);
            
            // 2. 更新调用图
            for call in &recursive_calls {
                self.call_graph.add_recursive_call(name, call.clone());
            }
            
            // 3. 检查间接递归
            let has_indirect_recursion = self.check_indirect_recursion(name);
            if has_indirect_recursion {
                return Err(TerminationError::IndirectRecursion {
                    function: name.clone(),
                });
            }
            
            // 4. 分析规模变化
            if recursive_calls.is_empty() {
                // 无递归，终止
                return Ok(TerminationStatus::Proved);
            }
            
            // 5. 规模递减分析
            let size_changes = self.analyze_size_changes(&recursive_calls, &func.params);
            
            if self.is_decreasing(&size_changes) {
                Ok(TerminationStatus::SizeChange(size_changes))
            } else if self.is_structural_recursion(&recursive_calls, &func.params, &func.param_types) {
                Ok(TerminationStatus::Structural)
            } else {
                Ok(TerminationStatus::NonTerminating)
            }
        }

        /// 查找递归调用
        fn find_recursive_calls(&self, expr: &Expression, func_name: &string) -> Vec<RecursiveCall> {
            let mut calls = Vec::new();
            self.find_calls_in_expr(expr, func_name, &mut calls);
            calls
        }

        fn find_calls_in_expr(&self, expr: &Expression, func_name: &string, calls: &mut Vec<RecursiveCall>) {
            match expr {
                Expression::Call(name, args) => {
                    if name == func_name {
                        let call_args: Vec<CallArg> = args.iter()
                            .map(|a| CallArg {
                                expr: a.clone(),
                                size_change: self.analyze_size_change(a, func_name),
                            })
                            .collect();
                        
                        calls.push(RecursiveCall {
                            callee: name.clone(),
                            args: call_args,
                            position: Position {
                                file: "".to_string(),
                                line: 0,
                                column: 0,
                            },
                            is_tail_call: false,
                        });
                    }
                    
                    // 递归检查参数中的调用
                    for arg in args {
                        self.find_calls_in_expr(arg, func_name, calls);
                    }
                }
                
                Expression::BinOp(_, left, right) => {
                    self.find_calls_in_expr(left, func_name, calls);
                    self.find_calls_in_expr(right, func_name, calls);
                }
                
                Expression::UnOp(_, expr) => {
                    self.find_calls_in_expr(expr, func_name, calls);
                }
                
                Expression::If(cond, then_br, else_br) => {
                    self.find_calls_in_expr(cond, func_name, calls);
                    self.find_calls_in_expr(then_br, func_name, calls);
                    if let Some(else_) = else_br {
                        self.find_calls_in_expr(else_, func_name, calls);
                    }
                }
                
                Expression::Match(expr, arms) => {
                    self.find_calls_in_expr(expr, func_name, calls);
                    for arm in arms {
                        self.find_calls_in_expr(&arm.body, func_name, calls);
                    }
                }
                
                Expression::Tuple(exprs) | Expression::Array(exprs) => {
                    for e in exprs {
                        self.find_calls_in_expr(e, func_name, calls);
                    }
                }
                
                Expression::FieldAccess(expr, _) | Expression::Index(expr, idx) => {
                    self.find_calls_in_expr(expr, func_name, calls);
                    self.find_calls_in_expr(idx, func_name, calls);
                }
                
                _ => {}
            }
        }

        /// 分析参数的大小变化
        fn analyze_size_change(&self, arg: &Expression, func_name: &string) -> Option<SizeChange> {
            match arg {
                Expression::Variable(name) => {
                    // 参数变量，大小不变
                    Some(SizeChange::Unchanged)
                }
                
                Expression::BinOp(BinOp::Sub, left, right) => {
                    // 检查是否是 n-1 或 n-k (k > 0)
                    if let Expression::Variable(_) = **left {
                        if let Expression::Literal(Literal::Int(k)) = **right {
                            if k > 0 {
                                return Some(SizeChange::Decreased);
                            }
                        }
                    }
                    Some(SizeChange::Unknown)
                }
                
                Expression::BinOp(BinOp::Add, left, right) => {
                    // 检查是否是 n + (-k) 或类似
                    Some(SizeChange::Unknown)
                }
                
                Expression::BinOp(BinOp::Div, left, _) => {
                    // n / 2 等
                    Some(SizeChange::Decreased)
                }
                
                Expression::Constructor(_, args) => {
                    // 构造器调用，检查参数
                    for arg in args {
                        if let Some(change) = self.analyze_size_change(arg, func_name) {
                            if change == SizeChange::Decreased {
                                return Some(SizeChange::Decreased);
                            }
                        }
                    }
                    Some(SizeChange::Unchanged)
                }
                
                _ => Some(SizeChange::Unknown),
            }
        }

        fn analyze_size_changes(&self, calls: &[RecursiveCall], params: &[string]) -> Vec<SizeChange> {
            let mut all_changes = Vec::new();
            
            for call in calls {
                let mut call_changes = Vec::new();
                for (i, arg) in call.args.iter().enumerate() {
                    call_changes.push(arg.size_change.clone().unwrap_or(SizeChange::Unknown));
                }
                
                // 取所有调用的共同变化
                if let Some(first) = call_changes.first() {
                    all_changes.push(first.clone());
                }
            }
            
            all_changes
        }

        /// 检查是否规模递减
        fn is_decreasing(&self, changes: &[SizeChange]) -> bool {
            changes.iter().all(|c| *c == SizeChange::Decreased)
        }

        /// 检查是否是结构递归
        fn is_structural_recursion(
            &self,
            calls: &[RecursiveCall],
            params: &[string],
            param_types: &[Type],
        ) -> bool {
            for call in calls {
                for (i, arg) in call.args.iter().enumerate() {
                    if let Some(param_name) = params.get(i) {
                        if self.is_structural_subpattern(&arg.expr, param_name) {
                            return true;
                        }
                    }
                }
            }
            false
        }

        /// 检查是否是结构的子模式（如 cons 的 tail）
        fn is_structural_subpattern(&self, expr: &Expression, param_name: &string) -> bool {
            match expr {
                Expression::Variable(name) => name == param_name,
                
                Expression::Constructor(name, args) => {
                    // 检查是否是列表或类似结构的构造器
                    if name == "Cons" || name == "::" {
                        if let Some(tail) = args.get(1) {
                            self.is_structural_subpattern(tail, param_name)
                        } else {
                            false
                        }
                    } else if name == "Nil" || name == "[]" {
                        true
                    } else {
                        false
                    }
                }
                
                Expression::FieldAccess(expr, field) => {
                    // 检查结构体字段
                    self.is_structural_subpattern(expr, param_name)
                }
                
                _ => false,
            }
        }

        /// 检查间接递归
        fn check_indirect_recursion(&self, func_name: &string) -> bool {
            // A -> B -> A 形式的递归
            let mut visited = HashSet::new();
            let mut stack = vec![func_name.clone()];
            
            while let Some(current) = stack.pop() {
                if visited.contains(&current) {
                    if current == func_name {
                        return true;  // 找到回到起点的路径
                    }
                    continue;
                }
                visited.insert(current.clone());
                
                // 查找当前函数调用的函数
                for (caller, callee) in &self.call_graph.edges {
                    if caller == &current {
                        stack.push(callee.clone());
                    }
                }
            }
            
            false
        }

        /// 证明终止（简化版）
        pub fn prove_termination(&mut self, func_name: &string) -> Result<TerminationStatus, TerminationError> {
            if let Some(func) = self.functions.get(func_name).cloned() {
                self.check_termination(func_name, &func)
            } else {
                Err(TerminationError::UnknownFunction {
                    name: func_name.clone(),
                })
            }
        }
    }

    // ==================== 终止证明结果 ====================

    #[derive(Debug, Clone)]
    pub struct TerminationReport {
        pub functions: Vec<FunctionReport>,
        pub total_errors: int,
    }

    impl TerminationReport {
        pub fn new() -> Self {
            TerminationReport {
                functions: Vec::new(),
                total_errors: 0,
            }
        }

        pub fn add_function(&mut self, report: FunctionReport) {
            if !report.is_terminating {
                self.total_errors += 1;
            }
            self.functions.push(report);
        }

        pub fn report(&self) -> String {
            let mut report = StringBuilder::new();
            
            report.push_str("=== Termination Analysis Report ===\n\n");
            
            for func in &self.functions {
                report.push_str(&format!("Function: {}\n", func.name));
                report.push_str(&format!("  Status: {}\n", func.status.to_string()));
                
                if !func.errors.is_empty() {
                    report.push_str("  Errors:\n");
                    for error in &func.errors {
                        report.push_str(&format!("    - {}\n", error));
                    }
                }
                
                if let Some(size_changes) = &func.size_changes {
                    report.push_str(&format!("  Size changes: {:?}\n", size_changes));
                }
                
                report.push_str("\n");
            }
            
            report.push_str(&format!("Total non-terminating functions: {}\n", self.total_errors));
            
            report.to_string()
        }
    }

    #[derive(Debug, Clone)]
    pub struct FunctionReport {
        pub name: string,
        pub is_terminating: bool,
        pub status: TerminationStatus,
        pub size_changes: Option<Vec<SizeChange>>,
        pub errors: Vec<String>,
    }

    impl FunctionReport {
        pub fn new(name: string) -> Self {
            FunctionReport {
                name,
                is_terminating: true,
                status: TerminationStatus::Unknown,
                size_changes: None,
                errors: Vec::new(),
            }
        }
    }

    // ==================== 错误类型 ====================

    #[derive(Debug, Clone)]
    pub enum TerminationError {
        NonTerminating {
            function: string,
            call: RecursiveCall,
        },
        IndirectRecursion {
            function: string,
        },
        MutualRecursion {
            functions: Vec<string>,
        },
        UnknownFunction {
            name: string,
        },
        InfiniteRecursion {
            function: string,
            reason: string,
        },
    }

    impl std::fmt::Display for TerminationError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                TerminationError::NonTerminating { function, call } => {
                    write!(f, "Function '{}' may not terminate. Recursive call at position {:?}", 
                        function, call.position)
                }
                TerminationError::IndirectRecursion { function } => {
                    write!(f, "Function '{}' has indirect recursion", function)
                }
                TerminationError::MutualRecursion { functions } => {
                    write!(f, "Mutual recursion detected: {}", functions.join(" -> "))
                }
                TerminationError::UnknownFunction { name } => {
                    write!(f, "Unknown function: {}", name)
                }
                TerminationError::InfiniteRecursion { function, reason } => {
                    write!(f, "Function '{}' has infinite recursion: {}", function, reason)
                }
            }
        }
    }

    impl TerminationStatus {
        pub fn to_string(&self) -> String {
            match self {
                TerminationStatus::Proving => "⏳ Proving".to_string(),
                TerminationStatus::Proved => "✅ Terminating".to_string(),
                TerminationStatus::NonTerminating => "❌ Non-terminating".to_string(),
                TerminationStatus::Unknown => "❓ Unknown".to_string(),
                TerminationStatus::Structural => "🔀 Structural".to_string(),
                TerminationStatus::SizeChange(changes) => {
                    format!("📉 Decreasing ({:?})", changes)
                }
            }
        }
    }
}
