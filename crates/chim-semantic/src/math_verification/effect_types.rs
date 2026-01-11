use chim_ast::*;
use chim_span::Span;
use std::collections::HashMap;
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Effect {
    Read(Ident),
    Write(Ident),
    Modify(Ident),
    Allocate,
    Deallocate,
    Async,
    Throw,
    Catch,
    Yield,
    Await,
    Custom(String),
}

#[derive(Debug, Clone)]
pub struct EffectType {
    pub base: Type,
    pub effects: HashSet<Effect>,
}

#[derive(Debug, Clone)]
pub struct EffectTypeChecker {
    pub effect_contexts: Vec<HashMap<Ident, HashSet<Effect>>>,
    pub errors: Vec<EffectTypeError>,
}

#[derive(Debug, Clone)]
pub struct EffectTypeError {
    pub kind: EffectTypeErrorKind,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum EffectTypeErrorKind {
    EffectNotHandled(Effect),
    EffectViolation(String),
    EffectMismatch(Effect, Effect),
    InvalidEffectType(Type),
    EffectOrderViolation(Effect, Effect),
}

impl EffectTypeChecker {
    pub fn new() -> Self {
        EffectTypeChecker {
            effect_contexts: vec![HashMap::new()],
            errors: Vec::new(),
        }
    }

    pub fn check(&mut self, expr: &Expr) -> Result<(), Vec<EffectTypeError>> {
        self.check_expr(expr)?;
        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(self.errors.clone())
        }
    }

    fn check_expr(&mut self, expr: &Expr) -> Result<(), Vec<EffectTypeError>> {
        match &expr.kind {
            ExprKind::Var(var_expr) => {
                self.check_variable_effects(&var_expr.name)?;
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
                self.push_effect_context();
                for stmt in &block_expr.stmts {
                    self.check_stmt(stmt)?;
                }
                if let Some(expr) = &block_expr.expr {
                    self.check_expr(expr)?;
                }
                self.pop_effect_context();
            }
            ExprKind::EffectBlock(effect_block_expr) => {
                self.check_effect_block(effect_block_expr)?;
            }
            ExprKind::Throw(throw_expr) => {
                self.check_expr(&throw_expr.error)?;
                self.add_effect(Effect::Throw);
            }
            ExprKind::Yield(yield_expr) => {
                if let Some(value) = &yield_expr.value {
                    self.check_expr(value)?;
                }
                self.add_effect(Effect::Yield);
            }
            ExprKind::Future(future_expr) => {
                self.check_expr(&future_expr.body)?;
                self.add_effect(Effect::Async);
            }
            _ => {}
        }
        Ok(())
    }

    fn check_stmt(&mut self, stmt: &Stmt) -> Result<(), Vec<EffectTypeError>> {
        match stmt {
            Stmt::Let(let_stmt) => {
                if let Some(expr) = &let_stmt.value {
                    self.check_expr(expr)?;
                }
                self.register_variable_effects(&let_stmt.name, HashSet::new());
            }
            Stmt::Expr(expr_stmt) => {
                self.check_expr(&expr_stmt.expr)?;
            }
            Stmt::Assign(assign_stmt) => {
                self.check_expr(&assign_stmt.value)?;
                self.add_write_effect(&assign_stmt.target)?;
            }
            Stmt::EffectBlock(effect_block_stmt) => {
                self.check_effect_block(&effect_block_stmt.block)?;
            }
            _ => {}
        }
        Ok(())
    }

    fn check_effect_block(&mut self, effect_block: &EffectBlockExpr) -> Result<(), Vec<EffectTypeError>> {
        self.push_effect_context();
        for stmt in &effect_block.stmts {
            self.check_stmt(stmt)?;
        }
        self.pop_effect_context();
        Ok(())
    }

    fn check_variable_effects(&mut self, var: &Ident) -> Result<(), Vec<EffectTypeError>> {
        if let Some(context) = self.effect_contexts.last() {
            if let Some(effects) = context.get(var) {
                for effect in effects {
                    self.check_effect_handled(effect)?;
                }
            }
        }
        Ok(())
    }

    fn check_effect_handled(&mut self, effect: &Effect) -> Result<(), Vec<EffectTypeError>> {
        match effect {
            Effect::Read(_) | Effect::Write(_) | Effect::Modify(_) => {
                Ok(())
            }
            Effect::Allocate | Effect::Deallocate => {
                Ok(())
            }
            Effect::Async | Effect::Throw | Effect::Catch | Effect::Yield | Effect::Await => {
                self.errors.push(EffectTypeError {
                    kind: EffectTypeErrorKind::EffectNotHandled(effect.clone()),
                    span: Span::default(),
                });
                Err(self.errors.clone())
            }
            Effect::Custom(_) => {
                Ok(())
            }
        }
    }

    fn add_write_effect(&mut self, var: &Ident) {
        if let Some(context) = self.effect_contexts.last_mut() {
            context.entry(var.clone()).or_insert_with(HashSet::new).insert(Effect::Write(var.clone()));
        }
    }

    fn add_effect(&mut self, effect: Effect) {
        if let Some(context) = self.effect_contexts.last_mut() {
            for (_, effects) in context.iter_mut() {
                effects.insert(effect.clone());
            }
        }
    }

    fn register_variable_effects(&mut self, var: &Ident, effects: HashSet<Effect>) {
        if let Some(context) = self.effect_contexts.last_mut() {
            context.insert(var.clone(), effects);
        }
    }

    fn push_effect_context(&mut self) {
        self.effect_contexts.push(HashMap::new());
    }

    fn pop_effect_context(&mut self) {
        self.effect_contexts.pop();
    }

    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    pub fn clear_errors(&mut self) {
        self.errors.clear();
    }

    pub fn check_item(&mut self, item: &Item) -> Result<(), Vec<EffectTypeError>> {
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

impl Default for EffectTypeChecker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_effect_type_checker_creation() {
        let checker = EffectTypeChecker::new();
        assert!(!checker.has_errors());
    }

    #[test]
    fn test_effect_context_push_pop() {
        let mut checker = EffectTypeChecker::new();
        checker.push_effect_context();
        assert_eq!(checker.effect_contexts.len(), 2);
        checker.pop_effect_context();
        assert_eq!(checker.effect_contexts.len(), 1);
    }

    #[test]
    fn test_error_clearing() {
        let mut checker = EffectTypeChecker::new();
        checker.errors.push(EffectTypeError {
            kind: EffectTypeErrorKind::EffectNotHandled(Effect::Throw),
            span: Span::default(),
        });
        assert!(checker.has_errors());
        checker.clear_errors();
        assert!(!checker.has_errors());
    }
}
