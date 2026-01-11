use crate::type_pool::{TypeId, TypeData, TypePool, Mutability};
use crate::ChimError;
use chim_span::Span;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BorrowId(usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct VariableId(usize);

#[derive(Debug, Clone, PartialEq)]
pub struct Borrow {
    pub from: VariableId,
    pub kind: BorrowKind,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BorrowKind {
    Shared,
    Mutable,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Variable {
    pub id: VariableId,
    pub name: String,
    pub ty: TypeId,
    pub mutability: Mutability,
    pub span: Span,
}

#[derive(Debug)]
pub struct BorrowChecker {
    variables: HashMap<VariableId, Variable>,
    borrows: HashMap<BorrowId, Borrow>,
    variable_count: usize,
    borrow_count: usize,
    errors: Vec<ChimError>,
}

impl BorrowChecker {
    pub fn new() -> Self {
        BorrowChecker {
            variables: HashMap::new(),
            borrows: HashMap::new(),
            variable_count: 0,
            borrow_count: 0,
            errors: Vec::new(),
        }
    }

    pub fn check_program(&mut self, _pool: &TypePool, _program: &crate::ast::Program) -> Result<(), Vec<ChimError>> {
        Ok(())
    }

    pub fn declare_variable(&mut self, name: &str, ty: TypeId, is_mut: bool, span: Span) -> VariableId {
        let id = VariableId(self.variable_count);
        self.variable_count += 1;
        self.variables.insert(id, Variable {
            id,
            name: name.to_string(),
            ty,
            mutability: if is_mut { Mutability::Mutable } else { Mutability::Immutable },
            span,
        });
        id
    }

    pub fn record_borrow(&mut self, _var_id: VariableId, _kind: BorrowKind, _span: Span) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_variable_creation() {
        let pool = TypePool::new();
        let mut checker = BorrowChecker::new();
        let var_id = checker.declare_variable("x", pool.builtin_types.i32, false, Span::new(chim_span::FileId(0), 0, 0, 0, 0));
        assert_eq!(var_id.0, 0);
    }

    #[test]
    fn test_borrow_recording() {
        let pool = TypePool::new();
        let mut checker = BorrowChecker::new();
        let var_id = checker.declare_variable("x", pool.builtin_types.i32, false, Span::new(chim_span::FileId(0), 0, 0, 0, 0));
        checker.record_borrow(var_id, BorrowKind::Shared, Span::new(chim_span::FileId(0), 0, 0, 0, 0));
    }
}
