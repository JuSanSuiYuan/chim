use chim_ast::*;
use chim_span::Span;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct DependentType {
    pub base: Type,
    pub constraints: Vec<TypeConstraint>,
}

#[derive(Debug, Clone)]
pub enum TypeConstraint {
    Equality(Type, Type),
    Inequality(Type, Type),
    GreaterThan(Type, Type),
    LessThan(Type, Type),
    GreaterThanOrEqual(Type, Type),
    LessThanOrEqual(Type, Type),
}

#[derive(Debug, Clone)]
pub struct DependentTypeChecker {
    pub constraints: HashMap<Ident, Vec<TypeConstraint>>,
    pub errors: Vec<DependentTypeError>,
}

#[derive(Debug, Clone)]
pub struct DependentTypeError {
    pub kind: DependentTypeErrorKind,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum DependentTypeErrorKind {
    ConstraintViolation(String),
    TypeMismatch(Type, Type),
    UnresolvedConstraint(Ident),
    InvalidDependentType(Type),
}

impl DependentTypeChecker {
    pub fn new() -> Self {
        DependentTypeChecker {
            constraints: HashMap::new(),
            errors: Vec::new(),
        }
    }

    pub fn check(&mut self, dependent_type: &DependentType) -> Result<(), Vec<DependentTypeError>> {
        self.check_type(&dependent_type.base)?;
        for constraint in &dependent_type.constraints {
            self.check_constraint(constraint)?;
        }
        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(self.errors.clone())
        }
    }

    fn check_type(&mut self, ty: &Type) -> Result<(), Vec<DependentTypeError>> {
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
            Type::Dependent(dep_ty) => {
                self.check_dependent_type(dep_ty)?;
            }
            _ => {}
        }
        Ok(())
    }

    fn check_dependent_type(&mut self, dep_ty: &DependentType) -> Result<(), Vec<DependentTypeError>> {
        self.check_type(&dep_ty.base)?;
        for constraint in &dep_ty.constraints {
            self.check_constraint(constraint)?;
        }
        Ok(())
    }

    fn check_constraint(&mut self, constraint: &TypeConstraint) -> Result<(), Vec<DependentTypeError>> {
        match constraint {
            TypeConstraint::Equality(ty1, ty2) => {
                self.check_type(ty1)?;
                self.check_type(ty2)?;
            }
            TypeConstraint::Inequality(ty1, ty2) => {
                self.check_type(ty1)?;
                self.check_type(ty2)?;
            }
            TypeConstraint::GreaterThan(ty1, ty2) => {
                self.check_type(ty1)?;
                self.check_type(ty2)?;
            }
            TypeConstraint::LessThan(ty1, ty2) => {
                self.check_type(ty1)?;
                self.check_type(ty2)?;
            }
            TypeConstraint::GreaterThanOrEqual(ty1, ty2) => {
                self.check_type(ty1)?;
                self.check_type(ty2)?;
            }
            TypeConstraint::LessThanOrEqual(ty1, ty2) => {
                self.check_type(ty1)?;
                self.check_type(ty2)?;
            }
        }
        Ok(())
    }

    pub fn add_constraint(&mut self, var: Ident, constraint: TypeConstraint) {
        self.constraints.entry(var).or_insert_with(Vec::new).push(constraint);
    }

    pub fn get_constraints(&self, var: &Ident) -> Option<&Vec<TypeConstraint>> {
        self.constraints.get(var)
    }

    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    pub fn clear_errors(&mut self) {
        self.errors.clear();
    }

    pub fn check_item(&mut self, item: &Item) -> Result<(), Vec<DependentTypeError>> {
        match item {
            Item::Function(func) => {
                for param in &func.params {
                    self.check_type(&param.ty)?;
                }
                if let Some(return_ty) = &func.return_type {
                    self.check_type(return_ty)?;
                }
                for stmt in &func.body {
                    self.check_stmt(stmt)?;
                }
            }
            Item::Struct(struct_def) => {
                for field in &struct_def.fields {
                    self.check_type(&field.ty)?;
                }
            }
            Item::Enum(enum_def) => {
                for variant in &enum_def.variants {
                    for field in &variant.fields {
                        self.check_type(&field.ty)?;
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

    fn check_stmt(&mut self, stmt: &Stmt) -> Result<(), Vec<DependentTypeError>> {
        match stmt {
            Stmt::Let(let_stmt) => {
                self.check_type(&let_stmt.ty)?;
                if let Some(expr) = &let_stmt.value {
                    self.check_expr(expr)?;
                }
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

    fn check_expr(&mut self, expr: &Expr) -> Result<(), Vec<DependentTypeError>> {
        match &expr.kind {
            ExprKind::Var(_) => {}
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
            _ => {}
        }
        Ok(())
    }
}

impl Default for DependentTypeChecker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dependent_type_checker_creation() {
        let checker = DependentTypeChecker::new();
        assert!(!checker.has_errors());
    }

    #[test]
    fn test_constraint_addition() {
        let mut checker = DependentTypeChecker::new();
        let var = Arc::from("n");
        let constraint = TypeConstraint::GreaterThan(
            Type::Int,
            Type::Int,
        );
        checker.add_constraint(var.clone(), constraint);
        let constraints = checker.get_constraints(&var);
        assert!(constraints.is_some());
        assert_eq!(constraints.unwrap().len(), 1);
    }

    #[test]
    fn test_error_clearing() {
        let mut checker = DependentTypeChecker::new();
        checker.errors.push(DependentTypeError {
            kind: DependentTypeErrorKind::ConstraintViolation("test".to_string()),
            span: Span::default(),
        });
        assert!(checker.has_errors());
        checker.clear_errors();
        assert!(!checker.has_errors());
    }
}
