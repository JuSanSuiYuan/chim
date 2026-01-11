use chim_ast::*;
use chim_span::Span;
use std::collections::HashMap;
use std::collections::HashSet;

#[derive(Debug, Clone)]
pub struct LinearType {
    pub base: Type,
    pub usage_count: usize,
    pub is_linear: bool,
}

#[derive(Debug, Clone)]
pub struct LinearTypeChecker {
    pub variables: HashMap<Ident, LinearType>,
    pub usage_counts: HashMap<Ident, usize>,
    pub errors: Vec<LinearTypeError>,
}

#[derive(Debug, Clone)]
pub struct LinearTypeError {
    pub kind: LinearTypeErrorKind,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum LinearTypeErrorKind {
    VariableUsedMultipleTimes(Ident),
    VariableNotUsed(Ident),
    VariableUsedAfterConsumption(Ident),
    InvalidLinearType(Type),
    LinearConstraintViolation(String),
}

impl LinearTypeChecker {
    pub fn new() -> Self {
        LinearTypeChecker {
            variables: HashMap::new(),
            usage_counts: HashMap::new(),
            errors: Vec::new(),
        }
    }

    pub fn check(&mut self, expr: &Expr) -> Result<(), Vec<LinearTypeError>> {
        self.check_expr(expr)?;
        self.check_all_variables_used()?;
        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(self.errors.clone())
        }
    }

    fn check_expr(&mut self, expr: &Expr) -> Result<(), Vec<LinearTypeError>> {
        match &expr.kind {
            ExprKind::Var(var_expr) => {
                self.check_variable_usage(&var_expr.name)?;
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
                for stmt in &block_expr.stmts {
                    self.check_stmt(stmt)?;
                }
                if let Some(expr) = &block_expr.expr {
                    self.check_expr(expr)?;
                }
            }
            ExprKind::Closure(closure_expr) => {
                for param in &closure_expr.params {
                    self.register_variable(&param.name, Type::Unit, false);
                }
                self.check_expr(&closure_expr.body)?;
            }
            _ => {}
        }
        Ok(())
    }

    fn check_stmt(&mut self, stmt: &Stmt) -> Result<(), Vec<LinearTypeError>> {
        match stmt {
            Stmt::Let(let_stmt) => {
                if let Some(expr) = &let_stmt.value {
                    self.check_expr(expr)?;
                }
                self.register_variable(&let_stmt.name, let_stmt.ty.clone(), let_stmt.is_linear);
            }
            Stmt::Expr(expr_stmt) => {
                self.check_expr(&expr_stmt.expr)?;
            }
            Stmt::Assign(assign_stmt) => {
                self.check_expr(&assign_stmt.value)?;
                self.check_variable_usage(&assign_stmt.target)?;
            }
            _ => {}
        }
        Ok(())
    }

    fn check_variable_usage(&mut self, var: &Ident) -> Result<(), Vec<LinearTypeError>> {
        if let Some(linear_ty) = self.variables.get(var) {
            if linear_ty.is_linear {
                let count = self.usage_counts.entry(var.clone()).or_insert(0);
                *count += 1;
                if *count > 1 {
                    self.errors.push(LinearTypeError {
                        kind: LinearTypeErrorKind::VariableUsedMultipleTimes(var.clone()),
                        span: Span::default(),
                    });
                }
            }
        }
        Ok(())
    }

    fn register_variable(&mut self, var: &Ident, ty: Type, is_linear: bool) {
        self.variables.insert(var.clone(), LinearType {
            base: ty,
            usage_count: 0,
            is_linear,
        });
        self.usage_counts.insert(var.clone(), 0);
    }

    fn check_all_variables_used(&mut self) -> Result<(), Vec<LinearTypeError>> {
        for (var, linear_ty) in &self.variables {
            if linear_ty.is_linear {
                let count = self.usage_counts.get(var).unwrap_or(&0);
                if *count == 0 {
                    self.errors.push(LinearTypeError {
                        kind: LinearTypeErrorKind::VariableNotUsed(var.clone()),
                        span: Span::default(),
                    });
                }
            }
        }
        Ok(())
    }

    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    pub fn clear_errors(&mut self) {
        self.errors.clear();
    }

    pub fn check_item(&mut self, item: &Item) -> Result<(), Vec<LinearTypeError>> {
        match item {
            Item::Function(func) => {
                for param in &func.params {
                    if param.is_linear {
                        self.register_variable(&param.name, param.ty.clone(), true);
                    }
                }
                for stmt in &func.body {
                    self.check_stmt(stmt)?;
                }
            }
            Item::Struct(struct_def) => {
                for field in &struct_def.fields {
                    if field.is_linear {
                        self.register_variable(&field.name, field.ty.clone(), true);
                    }
                }
            }
            _ => {}
        }
        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(self.errors.clone())
        }
    }
}

impl Default for LinearTypeChecker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linear_type_checker_creation() {
        let checker = LinearTypeChecker::new();
        assert!(!checker.has_errors());
    }

    #[test]
    fn test_variable_registration() {
        let mut checker = LinearTypeChecker::new();
        let var = Arc::from("x");
        checker.register_variable(&var, Type::Int, true);
        assert!(checker.variables.contains_key(&var));
    }

    #[test]
    fn test_error_clearing() {
        let mut checker = LinearTypeChecker::new();
        checker.errors.push(LinearTypeError {
            kind: LinearTypeErrorKind::VariableUsedMultipleTimes(Arc::from("x")),
            span: Span::default(),
        });
        assert!(checker.has_errors());
        checker.clear_errors();
        assert!(!checker.has_errors());
    }
}
