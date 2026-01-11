use chim_ast::*;
use chim_span::Span;
use std::collections::HashMap;
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SessionType {
    Send(Type),
    Receive(Type),
    Choice(Vec<SessionType>),
    Recursion(Ident, Box<SessionType>),
    End,
}

#[derive(Debug, Clone)]
pub struct SessionTypeChecker {
    pub session_contexts: Vec<HashMap<Ident, SessionType>>,
    pub errors: Vec<SessionTypeError>,
}

#[derive(Debug, Clone)]
pub struct SessionTypeError {
    pub kind: SessionTypeErrorKind,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum SessionTypeErrorKind {
    SessionTypeMismatch(SessionType, SessionType),
    InvalidSessionType(Type),
    SessionProtocolViolation(String),
    DualSessionTypeError(SessionType, SessionType),
    RecursionDepthExceeded,
}

impl SessionTypeChecker {
    pub fn new() -> Self {
        SessionTypeChecker {
            session_contexts: vec![HashMap::new()],
            errors: Vec::new(),
        }
    }

    pub fn check(&mut self, expr: &Expr) -> Result<(), Vec<SessionTypeError>> {
        self.check_expr(expr)?;
        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(self.errors.clone())
        }
    }

    fn check_expr(&mut self, expr: &Expr) -> Result<(), Vec<SessionTypeError>> {
        match &expr.kind {
            ExprKind::Var(var_expr) => {
                self.check_variable_session(&var_expr.name)?;
            }
            ExprKind::Binary(bin_expr) => {
                self.check_expr(&bin_expr.left)?;
                self.check_expr(&bin_expr.right)?;
            }
            ExprKind::Unary(unary_expr) => {
                self.check_expr(&unary_expr.operand)?;
            }
            ExprKind::Call(call_expr) => {
                self.check_expr(&call_expr.callee)?;
                for arg in &call_expr.args {
                    self.check_expr(arg)?;
                }
            }
            ExprKind::Block(block_expr) => {
                self.push_session_context();
                for stmt in &block_expr.stmts {
                    self.check_stmt(stmt)?;
                }
                if let Some(expr) = &block_expr.expr {
                    self.check_expr(expr)?;
                }
                self.pop_session_context();
            }
            _ => {}
        }
        Ok(())
    }

    fn check_stmt(&mut self, stmt: &Stmt) -> Result<(), Vec<SessionTypeError>> {
        match stmt {
            Stmt::Let(let_stmt) => {
                if let Some(expr) = &let_stmt.value {
                    self.check_expr(expr)?;
                }
                self.register_variable_session(&let_stmt.name, SessionType::End);
            }
            Stmt::Expr(expr_stmt) => {
                self.check_expr(&expr_stmt.expr)?;
            }
            Stmt::Assign(assign_stmt) => {
                self.check_expr(&assign_stmt.value)?;
            }
            _ => {}
        }
        Ok(())
    }

    fn check_variable_session(&mut self, var: &Ident) -> Result<(), Vec<SessionTypeError>> {
        if let Some(context) = self.session_contexts.last() {
            if let Some(session_ty) = context.get(var) {
                self.check_session_type(session_ty)?;
            }
        }
        Ok(())
    }

    fn check_session_type(&mut self, session_ty: &SessionType) -> Result<(), Vec<SessionTypeError>> {
        match session_ty {
            SessionType::Send(ty) => {
                self.check_type(ty)?;
            }
            SessionType::Receive(ty) => {
                self.check_type(ty)?;
            }
            SessionType::Choice(choices) => {
                for choice in choices {
                    self.check_session_type(choice)?;
                }
            }
            SessionType::Recursion(var, body) => {
                self.push_session_context();
                self.register_variable_session(var, SessionType::Recursion(var.clone(), body.clone()));
                self.check_session_type(body)?;
                self.pop_session_context();
            }
            SessionType::End => {
                Ok(())
            }
        }
        Ok(())
    }

    fn check_type(&mut self, ty: &Type) -> Result<(), Vec<SessionTypeError>> {
        match ty {
            Type::Array(array_ty) => {
                self.check_type(&array_ty.element)?;
                if let Some(len) = &array_ty.len {
                    self.check_type(len)?;
                }
            }
            Type::Vector(vector_ty) => {
                self.check_type(&vector_ty.element)?;
                if let Some(len) = &vector_ty.len {
                    self.check_type(len)?;
                }
            }
            _ => {}
        }
        Ok(())
    }

    pub fn dual(&self, session_ty: &SessionType) -> Result<SessionType, SessionTypeError> {
        match session_ty {
            SessionType::Send(ty) => Ok(SessionType::Receive(ty.clone())),
            SessionType::Receive(ty) => Ok(SessionType::Send(ty.clone())),
            SessionType::End => Ok(SessionType::End),
            SessionType::Choice(choices) => {
                let dual_choices: Result<Vec<_>, _> = choices.iter().map(|c| self.dual(c)).collect();
                dual_choices.map(SessionType::Choice)
            }
            SessionType::Recursion(var, body) => {
                let dual_body = self.dual(body)?;
                Ok(SessionType::Recursion(var.clone(), Box::new(dual_body)))
            }
        }
    }

    pub fn check_duality(&mut self, session_ty1: &SessionType, session_ty2: &SessionType) -> Result<(), Vec<SessionTypeError>> {
        let dual_ty2 = self.dual(session_ty2)?;
        if session_ty1 != &dual_ty2 {
            self.errors.push(SessionTypeError {
                kind: SessionTypeErrorKind::DualSessionTypeError(session_ty1.clone(), dual_ty2),
                span: Span::default(),
            });
        }
        Ok(())
    }

    fn register_variable_session(&mut self, var: &Ident, session_ty: SessionType) {
        if let Some(context) = self.session_contexts.last_mut() {
            context.insert(var.clone(), session_ty);
        }
    }

    fn push_session_context(&mut self) {
        self.session_contexts.push(HashMap::new());
    }

    fn pop_session_context(&mut self) {
        self.session_contexts.pop();
    }

    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    pub fn clear_errors(&mut self) {
        self.errors.clear();
    }

    pub fn check_item(&mut self, item: &Item) -> Result<(), Vec<SessionTypeError>> {
        match item {
            Item::Function(func) => {
                for stmt in &func.body {
                    self.check_stmt(stmt)?;
                }
            }
            Item::Struct(_) => {}
            Item::Enum(_) => {}
            _ => {}
        }
        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(self.errors.clone())
        }
    }
}

impl Default for SessionTypeChecker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_type_checker_creation() {
        let checker = SessionTypeChecker::new();
        assert!(!checker.has_errors());
    }

    #[test]
    fn test_session_duality() {
        let checker = SessionTypeChecker::new();
        let send_ty = SessionType::Send(Type::Int);
        let receive_ty = SessionType::Receive(Type::Int);
        let dual = checker.dual(&send_ty).unwrap();
        assert_eq!(dual, receive_ty);
    }

    #[test]
    fn test_error_clearing() {
        let mut checker = SessionTypeChecker::new();
        checker.errors.push(SessionTypeError {
            kind: SessionTypeErrorKind::SessionProtocolViolation("test".to_string()),
            span: Span::default(),
        });
        assert!(checker.has_errors());
        checker.clear_errors();
        assert!(!checker.has_errors());
    }
}
