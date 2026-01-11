use crate::type_pool::{TypeId, TypeData, TypePool, IntSize, UintSize, FloatSize, Mutability};
use crate::ChimError;
use chim_ast::*;
use chim_span::Span;
use chim_error::ErrorKind;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct TypeConstraints {
    constraints: Vec<TypeConstraint>,
}

#[derive(Debug, Clone)]
pub enum TypeConstraint {
    Equal(TypeId, TypeId),
    SubType(TypeId, TypeId),
}

#[derive(Debug)]
pub struct TypeInferencer {
    pool: TypePool,
    vars: HashMap<usize, TypeId>,
    constraints: TypeConstraints,
    errors: Vec<ChimError>,
    scope_stack: Vec<HashMap<Ident, TypeId>>,
    current_function: Option<Function>,
}

impl TypeInferencer {
    pub fn new() -> Self {
        TypeInferencer {
            pool: TypePool::new(),
            vars: HashMap::new(),
            constraints: TypeConstraints::new(),
            errors: Vec::new(),
            scope_stack: Vec::new(),
            current_function: None,
        }
    }

    fn enter_scope(&mut self) {
        self.scope_stack.push(HashMap::new());
    }

    fn exit_scope(&mut self) {
        self.scope_stack.pop();
    }

    fn insert_var(&mut self, name: Ident, ty: TypeId) {
        if let Some(scope) = self.scope_stack.last_mut() {
            scope.insert(name, ty);
        }
    }

    fn lookup_var(&self, name: &Ident) -> Option<TypeId> {
        for scope in self.scope_stack.iter().rev() {
            if let Some(&ty) = scope.get(name) {
                return Some(ty);
            }
        }
        None
    }

    fn fresh_type_var(&mut self) -> TypeId {
        let id = self.vars.len();
        let ty_id = self.pool.add_type(TypeData::TypeVar(id));
        self.vars.insert(id, ty_id);
        ty_id
    }

    fn unify(&mut self, ty1: TypeId, ty2: TypeId) -> Result<(), Vec<ChimError>> {
        let ty1_data = self.pool.get_type(ty1).clone();
        let ty2_data = self.pool.get_type(ty2).clone();

        match (ty1_data, ty2_data) {
            (TypeData::TypeVar(id1), TypeData::TypeVar(id2)) if id1 == id2 => Ok(()),
            (TypeData::TypeVar(id), _) => {
                self.vars.insert(id, ty2);
                Ok(())
            }
            (_, TypeData::TypeVar(id)) => {
                self.vars.insert(id, ty1);
                Ok(())
            }
            (TypeData::Int(s1), TypeData::Int(s2)) if s1 == s2 => Ok(()),
            (TypeData::Uint(s1), TypeData::Uint(s2)) if s1 == s2 => Ok(()),
            (TypeData::Float(s1), TypeData::Float(s2)) if s1 == s2 => Ok(()),
            (TypeData::Bool, TypeData::Bool) => Ok(()),
            (TypeData::String, TypeData::String) => Ok(()),
            (TypeData::Char, TypeData::Char) => Ok(()),
            (TypeData::Byte, TypeData::Byte) => Ok(()),
            (TypeData::Unit, TypeData::Unit) => Ok(()),
            (TypeData::Null, TypeData::Null) => Ok(()),
            (TypeData::Pointer(inner1, mut1), TypeData::Pointer(inner2, mut2)) if mut1 == mut2 => {
                self.unify(inner1, inner2)
            }
            (TypeData::Reference(lifetime1, inner1, mut1), TypeData::Reference(lifetime2, inner2, mut2)) if mut1 == mut2 => {
                let _ = (lifetime1, lifetime2);
                self.unify(inner1, inner2)
            }
            (TypeData::Array(elem1, size1), TypeData::Array(elem2, size2)) if size1 == size2 => {
                self.unify(elem1, elem2)
            }
            (TypeData::Slice(elem1), TypeData::Slice(elem2)) => {
                self.unify(elem1, elem2)
            }
            (TypeData::Tuple(elems1), TypeData::Tuple(elems2)) if elems1.len() == elems2.len() => {
                for (e1, e2) in elems1.iter().zip(elems2.iter()) {
                    self.unify(*e1, *e2)?;
                }
                Ok(())
            }
            (TypeData::Function(params1, ret1, _), TypeData::Function(params2, ret2, _)) if params1.len() == params2.len() => {
                for (p1, p2) in params1.iter().zip(params2.iter()) {
                    self.unify(*p1, *p2)?;
                }
                self.unify(*ret1, *ret2)
            }
            (TypeData::Infer, _) | (_, TypeData::Infer) => Ok(()),
            _ => {
                Err(vec![ChimError::new(
                    ErrorKind::TypeMismatch,
                    format!("cannot unify types {:?} and {:?}", ty1, ty2),
                )])
            }
        }
    }

    pub fn infer_program(&mut self, program: &Program) -> Result<(), Vec<ChimError>> {
        self.enter_scope();

        for item in &program.items {
            self.infer_item(item)?;
        }

        self.exit_scope();
        self.solve_constraints()?;

        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(std::mem::take(&mut self.errors))
        }
    }

    pub fn infer_item(&mut self, item: &Item) -> Result<(), Vec<ChimError>> {
        match item {
            Item::Function(func) => self.infer_function(func),
            Item::Struct(struct_def) => self.infer_struct(struct_def),
            Item::Enum(enum_def) => self.infer_enum(enum_def),
            Item::Trait(trait_def) => self.infer_trait(trait_def),
            Item::Impl(impl_def) => self.infer_impl(impl_def),
            Item::Use(_) => Ok(()),
            Item::Mod(mod_def) => self.infer_mod(mod_def),
            Item::Extern(extern_block) => self.infer_extern(extern_block),
            Item::Constant(const_def) => self.infer_constant(const_def),
            Item::Static(static_def) => self.infer_static(static_def),
            Item::Macro(macro_def) => self.infer_macro(macro_def),
            Item::ForAll(forall_def) => self.infer_forall(forall_def),
            Item::Default(default_def) => self.infer_default(default_def),
            Item::Sync(sync_def) => self.infer_sync(sync_def),
            Item::Sized(sized_def) => self.infer_sized(sized_def),
            Item::IntoIterator(intoiterator_def) => self.infer_intoiterator(intoiterator_def),
        }
    }

    pub fn infer_function(&mut self, func: &Function) -> Result<(), Vec<ChimError>> {
        self.current_function = Some(func.clone());
        self.enter_scope();

        for param in &func.params {
            let param_ty = self.infer_type(&param.ty)?;
            self.insert_var(param.name.clone(), param_ty);
        }

        for stmt in &func.body {
            self.infer_stmt(stmt)?;
        }

        self.exit_scope();
        self.current_function = None;
        Ok(())
    }

    pub fn infer_struct(&mut self, struct_def: &Struct) -> Result<(), Vec<ChimError>> {
        let _ = struct_def;
        Ok(())
    }

    pub fn infer_enum(&mut self, enum_def: &Enum) -> Result<(), Vec<ChimError>> {
        let _ = enum_def;
        Ok(())
    }

    pub fn infer_trait(&mut self, trait_def: &Trait) -> Result<(), Vec<ChimError>> {
        let _ = trait_def;
        Ok(())
    }

    pub fn infer_impl(&mut self, impl_def: &Impl) -> Result<(), Vec<ChimError>> {
        let _ = impl_def;
        Ok(())
    }

    pub fn infer_mod(&mut self, mod_def: &Mod) -> Result<(), Vec<ChimError>> {
        self.enter_scope();
        for item in &mod_def.items {
            self.infer_item(item)?;
        }
        self.exit_scope();
        Ok(())
    }

    pub fn infer_extern(&mut self, extern_block: &ExternBlock) -> Result<(), Vec<ChimError>> {
        let _ = extern_block;
        Ok(())
    }

    pub fn infer_constant(&mut self, const_def: &Constant) -> Result<(), Vec<ChimError>> {
        let _ = const_def;
        Ok(())
    }

    pub fn infer_static(&mut self, static_def: &Static) -> Result<(), Vec<ChimError>> {
        let _ = static_def;
        Ok(())
    }

    pub fn infer_macro(&mut self, macro_def: &Macro) -> Result<(), Vec<ChimError>> {
        let _ = macro_def;
        Ok(())
    }

    pub fn infer_forall(&mut self, forall_def: &ForAll) -> Result<(), Vec<ChimError>> {
        let _ = forall_def;
        Ok(())
    }

    pub fn infer_default(&mut self, default_def: &Default) -> Result<(), Vec<ChimError>> {
        let _ = default_def;
        Ok(())
    }

    pub fn infer_sync(&mut self, sync_def: &Sync) -> Result<(), Vec<ChimError>> {
        let _ = sync_def;
        Ok(())
    }

    pub fn infer_sized(&mut self, sized_def: &Sized) -> Result<(), Vec<ChimError>> {
        let _ = sized_def;
        Ok(())
    }

    pub fn infer_intoiterator(&mut self, intoiterator_def: &IntoIterator) -> Result<(), Vec<ChimError>> {
        let _ = intoiterator_def;
        Ok(())
    }

    pub fn infer_expr(&mut self, expr: &Expr) -> Result<TypeId, Vec<ChimError>> {
        match &*expr.kind {
            ExprKind::Identifier(name) => {
                if let Some(&ty) = self.lookup_var(name) {
                    Ok(ty)
                } else {
                    Err(vec![ChimError::new(
                        ErrorKind::UndefinedVariable,
                        format!("undefined variable: {}", name),
                    ).with_span(expr.span)])
                }
            }
            ExprKind::Literal(lit) => self.infer_literal(lit),
            ExprKind::Binary(bin_expr) => self.infer_binary_expr(bin_expr, expr.span),
            ExprKind::Unary(unary_expr) => self.infer_unary_expr(unary_expr, expr.span),
            ExprKind::Call(call_expr) => self.infer_call_expr(call_expr, expr.span),
            ExprKind::Block(block_expr) => self.infer_block_expr(block_expr),
            ExprKind::If(if_expr) => self.infer_if_expr(if_expr, expr.span),
            ExprKind::Match(match_expr) => self.infer_match_expr(match_expr, expr.span),
            ExprKind::Closure(closure_expr) => self.infer_closure_expr(closure_expr, expr.span),
            ExprKind::FieldAccess(field_access) => self.infer_field_access(field_access, expr.span),
            ExprKind::Index(index_expr) => self.infer_index_expr(index_expr, expr.span),
            ExprKind::Cast(cast_expr) => self.infer_cast_expr(cast_expr, expr.span),
            ExprKind::Assign(assign_expr) => self.infer_assign_expr(assign_expr, expr.span),
            ExprKind::AssignOp(assign_op_expr) => self.infer_assign_op_expr(assign_op_expr, expr.span),
            ExprKind::Range(range_expr) => self.infer_range_expr(range_expr, expr.span),
            ExprKind::Array(array_expr) => self.infer_array_expr(array_expr),
            ExprKind::Ternary(ternary_expr) => self.infer_ternary_expr(ternary_expr, expr.span),
            ExprKind::Wait(wait_expr) => self.infer_wait_expr(wait_expr, expr.span),
            ExprKind::Notify(notify_expr) => self.infer_notify_expr(notify_expr, expr.span),
            ExprKind::NotifyAll(notify_all_expr) => self.infer_notify_all_expr(notify_all_expr, expr.span),
            ExprKind::Iterator(iterator_expr) => self.infer_iterator_expr(iterator_expr, expr.span),
            ExprKind::Next(next_expr) => self.infer_next_expr(next_expr, expr.span),
            ExprKind::Item(item_expr) => self.infer_item_expr(item_expr, expr.span),
            ExprKind::Collect(collect_expr) => self.infer_collect_expr(collect_expr, expr.span),
            ExprKind::Chain(chain_expr) => self.infer_chain_expr(chain_expr, expr.span),
            ExprKind::Filter(filter_expr) => self.infer_filter_expr(filter_expr, expr.span),
            ExprKind::Fold(fold_expr) => self.infer_fold_expr(fold_expr, expr.span),
            ExprKind::Map(map_expr) => self.infer_map_expr(map_expr, expr.span),
            ExprKind::Result(result_expr) => self.infer_result_expr(result_expr, expr.span),
            ExprKind::Ok(ok_expr) => self.infer_ok_expr(ok_expr, expr.span),
            ExprKind::Err(err_expr) => self.infer_err_expr(err_expr, expr.span),
            ExprKind::Try(try_expr) => self.infer_try_expr(try_expr, expr.span),
            ExprKind::Catch(catch_expr) => self.infer_catch_expr(catch_expr, expr.span),
            ExprKind::ErrorExpr(error_expr) => self.infer_error_expr(error_expr, expr.span),
            ExprKind::Context(context_expr) => self.infer_context_expr(context_expr, expr.span),
            ExprKind::Throw(throw_expr) => self.infer_throw_expr(throw_expr, expr.span),
            ExprKind::Future(future_expr) => self.infer_future_expr(future_expr, expr.span),
            ExprKind::Yield(yield_expr) => self.infer_yield_expr(yield_expr, expr.span),
            ExprKind::Stream(stream_expr) => self.infer_stream_expr(stream_expr, expr.span),
        }
    }

    pub fn infer_literal(&mut self, lit: &Literal) -> Result<TypeId, Vec<ChimError>> {
        match lit.kind {
            LiteralKind::Int(_) => Ok(self.pool.builtin_types.i32),
            LiteralKind::Float(_) => Ok(self.pool.builtin_types.f64),
            LiteralKind::String(_) | LiteralKind::RawString(_) => Ok(self.pool.builtin_types.string),
            LiteralKind::Char(_) => Ok(self.pool.builtin_types.char),
            LiteralKind::Byte(_) => Ok(self.pool.builtin_types.byte),
            LiteralKind::Bool(_) => Ok(self.pool.builtin_types.bool),
            LiteralKind::Null => Ok(self.pool.builtin_types.null),
            LiteralKind::Unit => Ok(self.pool.builtin_types.unit),
            LiteralKind::ByteString(_) => Ok(self.pool.add_type(TypeData::Array(
                self.pool.builtin_types.byte,
                0,
            ))),
        }
    }

    pub fn infer_binary_expr(&mut self, bin_expr: &BinaryExpr, span: Span) -> Result<TypeId, Vec<ChimError>> {
        let left_ty = self.infer_expr(&bin_expr.left)?;
        let right_ty = self.infer_expr(&bin_expr.right)?;

        match bin_expr.op {
            BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Mod => {
                self.unify(left_ty, right_ty)?;
                Ok(left_ty)
            }
            BinOp::And | BinOp::Or | BinOp::BitAnd | BinOp::BitOr | BinOp::BitXor |
            BinOp::Shl | BinOp::Shr => {
                self.unify(left_ty, right_ty)?;
                Ok(left_ty)
            }
            BinOp::Eq | BinOp::Ne | BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge => {
                self.unify(left_ty, right_ty)?;
                Ok(self.pool.builtin_types.bool)
            }
        }
    }

    pub fn infer_unary_expr(&mut self, unary_expr: &UnaryExpr, span: Span) -> Result<TypeId, Vec<ChimError>> {
        let operand_ty = self.infer_expr(&unary_expr.expr)?;

        match unary_expr.op {
            UnOp::Neg | UnOp::Not | UnOp::FNeg => Ok(operand_ty),
            UnOp::Ref => Ok(self.pool.add_type(TypeData::Reference(
                None,
                operand_ty,
                Mutability::Immutable,
            ))),
            UnOp::RefMut => Ok(self.pool.add_type(TypeData::Reference(
                None,
                operand_ty,
                Mutability::Mutable,
            ))),
            UnOp::Deref => {
                if let TypeData::Pointer(inner, _) = self.pool.get_type(operand_ty).clone() {
                    Ok(inner)
                } else if let TypeData::Reference(_, inner, _) = self.pool.get_type(operand_ty).clone() {
                    Ok(inner)
                } else {
                    Err(vec![ChimError::new(
                        ErrorKind::TypeMismatch,
                        format!("cannot dereference non-pointer type {:?}", operand_ty),
                    ).with_span(span)])
                }
            }
        }
    }

    pub fn infer_call_expr(&mut self, call_expr: &CallExpr, span: Span) -> Result<TypeId, Vec<ChimError>> {
        let func_ty = self.infer_expr(&call_expr.func)?;
        let arg_tys: Result<Vec<_>, _> = call_expr.args.iter()
            .map(|arg| self.infer_expr(arg))
            .collect();

        let arg_tys = arg_tys?;

        if let TypeData::Function(param_tys, ret_ty, _) = self.pool.get_type(func_ty).clone() {
            if param_tys.len() != arg_tys.len() {
                return Err(vec![ChimError::new(
                    ErrorKind::TypeMismatch,
                    format!("expected {} arguments, got {}", param_tys.len(), arg_tys.len()),
                ).with_span(span)]);
            }

            for (param_ty, arg_ty) in param_tys.iter().zip(arg_tys.iter()) {
                self.unify(*param_ty, *arg_ty)?;
            }

            Ok(*ret_ty)
        } else {
            Err(vec![ChimError::new(
                ErrorKind::TypeMismatch,
                format!("cannot call non-function type {:?}", func_ty),
            ).with_span(span)])
        }
    }

    pub fn infer_block_expr(&mut self, block_expr: &BlockExpr) -> Result<TypeId, Vec<ChimError>> {
        self.enter_scope();

        let mut last_ty = self.pool.builtin_types.unit;
        for stmt in &block_expr.stmts {
            last_ty = self.infer_stmt(stmt)?;
        }

        self.exit_scope();
        Ok(last_ty)
    }

    pub fn infer_if_expr(&mut self, if_expr: &IfExpr, span: Span) -> Result<TypeId, Vec<ChimError>> {
        let cond_ty = self.infer_expr(&if_expr.condition)?;
        self.unify(cond_ty, self.pool.builtin_types.bool)?;

        let then_ty = self.infer_expr(&if_expr.then_branch)?;
        
        let else_ty = if let Some(else_branch) = &if_expr.else_branch {
            self.infer_expr(else_branch)?
        } else {
            self.pool.builtin_types.unit
        };

        self.unify(then_ty, else_ty)?;
        Ok(then_ty)
    }

    pub fn infer_match_expr(&mut self, match_expr: &MatchExpr, span: Span) -> Result<TypeId, Vec<ChimError>> {
        let value_ty = self.infer_expr(&match_expr.expr)?;
        let mut arm_tys = Vec::new();

        for arm in &match_expr.arms {
            let _ = self.infer_pattern(&arm.pattern, value_ty)?;
            if let Some(guard) = &arm.guard {
                let guard_ty = self.infer_expr(guard)?;
                self.unify(guard_ty, self.pool.builtin_types.bool)?;
            }
            let body_ty = self.infer_expr(&arm.body)?;
            arm_tys.push(body_ty);
        }

        for arm_ty in &arm_tys {
            self.unify(*arm_ty, arm_tys[0])?;
        }

        Ok(arm_tys.get(0).copied().unwrap_or(self.pool.builtin_types.unit))
    }

    pub fn infer_closure_expr(&mut self, closure_expr: &ClosureExpr, span: Span) -> Result<TypeId, Vec<ChimError>> {
        self.enter_scope();

        let param_tys: Result<Vec<_>, _> = closure_expr.params.iter()
            .map(|param| {
                let param_ty = self.infer_type(&param.ty)?;
                self.insert_var(param.name.clone(), param_ty);
                Ok(param_ty)
            })
            .collect();

        let param_tys = param_tys?;
        let body_ty = self.infer_expr(&closure_expr.body)?;

        self.exit_scope();

        let func_ty = self.pool.add_type(TypeData::Function(
            param_tys,
            Box::new(body_ty),
            closure_expr.is_async,
        ));

        Ok(func_ty)
    }

    pub fn infer_field_access(&mut self, field_access: &FieldAccessExpr, span: Span) -> Result<TypeId, Vec<ChimError>> {
        let obj_ty = self.infer_expr(&field_access.expr)?;

        match self.pool.get_type(obj_ty).clone() {
            TypeData::Struct(struct_id) => {
                if let Some(struct_data) = self.pool.get_struct(struct_id) {
                    for field in &struct_data.fields {
                        if field.name == *field_access.field {
                            return Ok(field.ty);
                        }
                    }
                }
                Err(vec![ChimError::new(
                    ErrorKind::TypeMismatch,
                    format!("struct has no field named {}", field_access.field),
                ).with_span(span)])
            }
            _ => Err(vec![ChimError::new(
                ErrorKind::TypeMismatch,
                format!("cannot access field on non-struct type {:?}", obj_ty),
            ).with_span(span)])
        }
    }

    pub fn infer_index_expr(&mut self, index_expr: &IndexExpr, span: Span) -> Result<TypeId, Vec<ChimError>> {
        let array_ty = self.infer_expr(&index_expr.expr)?;
        let index_ty = self.infer_expr(&index_expr.index)?;

        self.unify(index_ty, self.pool.builtin_types.i32)?;

        match self.pool.get_type(array_ty).clone() {
            TypeData::Array(elem_ty, _) | TypeData::Slice(elem_ty) => Ok(elem_ty),
            _ => Err(vec![ChimError::new(
                ErrorKind::TypeMismatch,
                format!("cannot index non-array type {:?}", array_ty),
            ).with_span(span)])
        }
    }

    pub fn infer_cast_expr(&mut self, cast_expr: &CastExpr, span: Span) -> Result<TypeId, Vec<ChimError>> {
        let _ = self.infer_expr(&cast_expr.expr)?;
        Ok(self.infer_type(&cast_expr.ty)?)
    }

    pub fn infer_assign_expr(&mut self, assign_expr: &AssignExpr, span: Span) -> Result<TypeId, Vec<ChimError>> {
        let left_ty = self.infer_expr(&assign_expr.left)?;
        let right_ty = self.infer_expr(&assign_expr.right)?;

        self.unify(left_ty, right_ty)?;
        Ok(left_ty)
    }

    pub fn infer_assign_op_expr(&mut self, assign_op_expr: &AssignOpExpr, span: Span) -> Result<TypeId, Vec<ChimError>> {
        let left_ty = self.infer_expr(&assign_op_expr.left)?;
        let right_ty = self.infer_expr(&assign_op_expr.right)?;

        self.unify(left_ty, right_ty)?;
        Ok(left_ty)
    }

    pub fn infer_range_expr(&mut self, range_expr: &RangeExpr, span: Span) -> Result<TypeId, Vec<ChimError>> {
        let _ = range_expr;
        Ok(self.pool.add_type(TypeData::Range))
    }

    pub fn infer_array_expr(&mut self, array_expr: &ArrayExpr, span: Span) -> Result<TypeId, Vec<ChimError>> {
        if array_expr.elements.is_empty() {
            return Ok(self.pool.add_type(TypeData::Array(
                self.pool.builtin_types.unit,
                0,
            )));
        }

        let elem_tys: Result<Vec<_>, _> = array_expr.elements.iter()
            .map(|elem| self.infer_expr(elem))
            .collect();

        let elem_tys = elem_tys?;

        for elem_ty in &elem_tys {
            self.unify(*elem_ty, elem_tys[0])?;
        }

        Ok(self.pool.add_type(TypeData::Array(
            elem_tys[0],
            elem_tys.len(),
        )))
    }

    pub fn infer_ternary_expr(&mut self, ternary_expr: &TernaryExpr, span: Span) -> Result<TypeId, Vec<ChimError>> {
        let cond_ty = self.infer_expr(&ternary_expr.condition)?;
        self.unify(cond_ty, self.pool.builtin_types.bool)?;

        let then_ty = self.infer_expr(&ternary_expr.then_branch)?;
        let else_ty = self.infer_expr(&ternary_expr.else_branch)?;

        self.unify(then_ty, else_ty)?;
        Ok(then_ty)
    }

    pub fn infer_wait_expr(&mut self, wait_expr: &WaitExpr, span: Span) -> Result<TypeId, Vec<ChimError>> {
        let atomic_ty = self.infer_expr(&wait_expr.atomic)?;
        if let Some(timeout) = &wait_expr.timeout {
            let timeout_ty = self.infer_expr(timeout)?;
            self.unify(timeout_ty, self.pool.builtin_types.i64)?;
        }
        Ok(self.pool.builtin_types.unit)
    }

    pub fn infer_notify_expr(&mut self, notify_expr: &NotifyExpr, span: Span) -> Result<TypeId, Vec<ChimError>> {
        let _ = self.infer_expr(&notify_expr.atomic)?;
        Ok(self.pool.builtin_types.unit)
    }

    pub fn infer_notify_all_expr(&mut self, notify_all_expr: &NotifyAllExpr, span: Span) -> Result<TypeId, Vec<ChimError>> {
        let _ = self.infer_expr(&notify_all_expr.atomic)?;
        Ok(self.pool.builtin_types.unit)
    }

    pub fn infer_iterator_expr(&mut self, iterator_expr: &IteratorExpr, span: Span) -> Result<TypeId, Vec<ChimError>> {
        let _ = self.infer_expr(&iterator_expr.iterable)?;
        Ok(self.pool.builtin_types.unit)
    }

    pub fn infer_next_expr(&mut self, next_expr: &NextExpr, span: Span) -> Result<TypeId, Vec<ChimError>> {
        let iterator_ty = self.infer_expr(&next_expr.iterator)?;
        Ok(self.fresh_type_var())
    }

    pub fn infer_item_expr(&mut self, item_expr: &ItemExpr, span: Span) -> Result<TypeId, Vec<ChimError>> {
        let iterator_ty = self.infer_expr(&item_expr.iterator)?;
        Ok(self.fresh_type_var())
    }

    pub fn infer_collect_expr(&mut self, collect_expr: &CollectExpr, span: Span) -> Result<TypeId, Vec<ChimError>> {
        let iterator_ty = self.infer_expr(&collect_expr.iterator)?;
        Ok(self.fresh_type_var())
    }

    pub fn infer_chain_expr(&mut self, chain_expr: &ChainExpr, span: Span) -> Result<TypeId, Vec<ChimError>> {
        let _ = self.infer_expr(&chain_expr.iterator1)?;
        let _ = self.infer_expr(&chain_expr.iterator2)?;
        Ok(self.fresh_type_var())
    }

    pub fn infer_filter_expr(&mut self, filter_expr: &FilterExpr, span: Span) -> Result<TypeId, Vec<ChimError>> {
        let _ = self.infer_expr(&filter_expr.iterator)?;
        let predicate_ty = self.infer_expr(&filter_expr.predicate)?;
        self.unify(predicate_ty, self.pool.builtin_types.bool)?;
        Ok(self.fresh_type_var())
    }

    pub fn infer_fold_expr(&mut self, fold_expr: &FoldExpr, span: Span) -> Result<TypeId, Vec<ChimError>> {
        let _ = self.infer_expr(&fold_expr.iterator)?;
        let _ = self.infer_expr(&fold_expr.init)?;
        let _ = self.infer_expr(&fold_expr.body)?;
        Ok(self.fresh_type_var())
    }

    pub fn infer_map_expr(&mut self, map_expr: &MapExpr, span: Span) -> Result<TypeId, Vec<ChimError>> {
        let _ = self.infer_expr(&map_expr.iterator)?;
        let _ = self.infer_expr(&map_expr.mapper)?;
        Ok(self.fresh_type_var())
    }

    pub fn infer_result_expr(&mut self, result_expr: &ResultExpr, span: Span) -> Result<TypeId, Vec<ChimError>> {
        let _ = self.infer_type(&result_expr.ok_type)?;
        let _ = self.infer_type(&result_expr.err_type)?;
        Ok(self.fresh_type_var())
    }

    pub fn infer_ok_expr(&mut self, ok_expr: &OkExpr, span: Span) -> Result<TypeId, Vec<ChimError>> {
        let _ = self.infer_expr(&ok_expr.value)?;
        Ok(self.fresh_type_var())
    }

    pub fn infer_err_expr(&mut self, err_expr: &ErrExpr, span: Span) -> Result<TypeId, Vec<ChimError>> {
        let _ = self.infer_expr(&err_expr.error)?;
        Ok(self.fresh_type_var())
    }

    pub fn infer_try_expr(&mut self, try_expr: &TryExpr, span: Span) -> Result<TypeId, Vec<ChimError>> {
        let _ = self.infer_expr(&try_expr.expr)?;
        Ok(self.fresh_type_var())
    }

    pub fn infer_catch_expr(&mut self, catch_expr: &CatchExpr, span: Span) -> Result<TypeId, Vec<ChimError>> {
        let _ = self.infer_expr(&catch_expr.try_expr)?;
        let _ = self.infer_expr(&catch_expr.catch_expr)?;
        Ok(self.fresh_type_var())
    }

    pub fn infer_error_expr(&mut self, error_expr: &ErrorExpr, span: Span) -> Result<TypeId, Vec<ChimError>> {
        let _ = self.infer_expr(&error_expr.message)?;
        Ok(self.pool.builtin_types.unit)
    }

    pub fn infer_context_expr(&mut self, context_expr: &ContextExpr, span: Span) -> Result<TypeId, Vec<ChimError>> {
        let _ = self.infer_expr(&context_expr.context)?;
        Ok(self.pool.builtin_types.unit)
    }

    pub fn infer_throw_expr(&mut self, throw_expr: &ThrowExpr, span: Span) -> Result<TypeId, Vec<ChimError>> {
        let _ = self.infer_expr(&throw_expr.error)?;
        Ok(self.pool.builtin_types.never)
    }

    pub fn infer_future_expr(&mut self, future_expr: &FutureExpr, span: Span) -> Result<TypeId, Vec<ChimError>> {
        let _ = self.infer_expr(&future_expr.body)?;
        Ok(self.fresh_type_var())
    }

    pub fn infer_yield_expr(&mut self, yield_expr: &YieldExpr, span: Span) -> Result<TypeId, Vec<ChimError>> {
        if let Some(value) = &yield_expr.value {
            let _ = self.infer_expr(value)?;
        }
        Ok(self.pool.builtin_types.unit)
    }

    pub fn infer_stream_expr(&mut self, stream_expr: &StreamExpr, span: Span) -> Result<TypeId, Vec<ChimError>> {
        let _ = self.infer_expr(&stream_expr.body)?;
        Ok(self.fresh_type_var())
    }

    pub fn infer_stmt(&mut self, stmt: &Stmt) -> Result<TypeId, Vec<ChimError>> {
        match &stmt.kind {
            StmtKind::Let(let_stmt) => self.infer_let_stmt(let_stmt, stmt.span),
            StmtKind::Var(var_stmt) => self.infer_var_stmt(var_stmt, stmt.span),
            StmtKind::Expr(expr) => self.infer_expr(expr),
            StmtKind::Return(return_stmt) => self.infer_return_stmt(return_stmt, stmt.span),
            StmtKind::Break(break_stmt) => self.infer_break_stmt(break_stmt, stmt.span),
            StmtKind::Continue => Ok(self.pool.builtin_types.unit),
            StmtKind::Loop(loop_stmt) => self.infer_loop_stmt(loop_stmt, stmt.span),
            StmtKind::While(while_stmt) => self.infer_while_stmt(while_stmt, stmt.span),
            StmtKind::For(for_stmt) => self.infer_for_stmt(for_stmt, stmt.span),
            StmtKind::Match(match_stmt) => {
                let _ = self.infer_match_expr(&MatchExpr {
                    expr: match_stmt.expr.clone(),
                    arms: match_stmt.arms.clone(),
                }, stmt.span)?;
                Ok(self.pool.builtin_types.unit)
            }
        }
    }

    pub fn infer_let_stmt(&mut self, let_stmt: &LetStmt, span: Span) -> Result<TypeId, Vec<ChimError>> {
        let pattern_ty = self.infer_pattern(&let_stmt.pattern, self.fresh_type_var())?;
        
        if let Some(ty) = &let_stmt.ty {
            let explicit_ty = self.infer_type(ty)?;
            self.unify(pattern_ty, explicit_ty)?;
        }

        if let Some(init) = &let_stmt.initializer {
            let init_ty = self.infer_expr(init)?;
            self.unify(pattern_ty, init_ty)?;
        }

        self.insert_var_from_pattern(&let_stmt.pattern, pattern_ty);
        Ok(self.pool.builtin_types.unit)
    }

    pub fn infer_var_stmt(&mut self, var_stmt: &VarStmt, span: Span) -> Result<TypeId, Vec<ChimError>> {
        let pattern_ty = self.infer_pattern(&var_stmt.pattern, self.fresh_type_var())?;
        
        if let Some(ty) = &var_stmt.ty {
            let explicit_ty = self.infer_type(ty)?;
            self.unify(pattern_ty, explicit_ty)?;
        }

        if let Some(init) = &var_stmt.initializer {
            let init_ty = self.infer_expr(init)?;
            self.unify(pattern_ty, init_ty)?;
        }

        self.insert_var_from_pattern(&var_stmt.pattern, pattern_ty);
        Ok(self.pool.builtin_types.unit)
    }

    pub fn infer_return_stmt(&mut self, return_stmt: &Option<Box<Expr>>, span: Span) -> Result<TypeId, Vec<ChimError>> {
        if let Some(value) = return_stmt {
            let value_ty = self.infer_expr(value)?;
            if let Some(func) = &self.current_function {
                if let Some(return_ty) = &func.return_type {
                    let explicit_return_ty = self.infer_type(return_ty)?;
                    self.unify(value_ty, explicit_return_ty)?;
                }
            }
            Ok(value_ty)
        } else {
            Ok(self.pool.builtin_types.unit)
        }
    }

    pub fn infer_break_stmt(&mut self, break_stmt: &Option<Box<Expr>>, span: Span) -> Result<TypeId, Vec<ChimError>> {
        if let Some(value) = break_stmt {
            self.infer_expr(value)
        } else {
            Ok(self.pool.builtin_types.unit)
        }
    }

    pub fn infer_loop_stmt(&mut self, loop_stmt: &LoopStmt, span: Span) -> Result<TypeId, Vec<ChimError>> {
        self.enter_scope();
        for stmt in &loop_stmt.body {
            self.infer_stmt(stmt)?;
        }
        self.exit_scope();
        Ok(self.pool.builtin_types.unit)
    }

    pub fn infer_while_stmt(&mut self, while_stmt: &WhileStmt, span: Span) -> Result<TypeId, Vec<ChimError>> {
        let cond_ty = self.infer_expr(&while_stmt.condition)?;
        self.unify(cond_ty, self.pool.builtin_types.bool)?;

        self.enter_scope();
        for stmt in &while_stmt.body {
            self.infer_stmt(stmt)?;
        }
        self.exit_scope();

        Ok(self.pool.builtin_types.unit)
    }

    pub fn infer_for_stmt(&mut self, for_stmt: &ForStmt, span: Span) -> Result<TypeId, Vec<ChimError>> {
        let iterable_ty = self.infer_expr(&for_stmt.iterable)?;
        let elem_ty = self.fresh_type_var();

        match self.pool.get_type(iterable_ty).clone() {
            TypeData::Array(array_elem_ty, _) | TypeData::Slice(array_elem_ty) => {
                self.unify(elem_ty, array_elem_ty)?;
            }
            TypeData::Range => {
                self.unify(elem_ty, self.pool.builtin_types.i32)?;
            }
            _ => {
                return Err(vec![ChimError::new(
                    ErrorKind::TypeMismatch,
                    format!("cannot iterate over type {:?}", iterable_ty),
                ).with_span(span)]);
            }
        }

        self.enter_scope();
        self.insert_var_from_pattern(&for_stmt.pattern, elem_ty);
        for stmt in &for_stmt.body {
            self.infer_stmt(stmt)?;
        }
        self.exit_scope();

        Ok(self.pool.builtin_types.unit)
    }

    pub fn infer_pattern(&mut self, pattern: &Pattern, expected_ty: TypeId) -> Result<TypeId, Vec<ChimError>> {
        match &pattern.kind {
            PatternKind::Identifier(name) => {
                self.unify(expected_ty, self.fresh_type_var())?;
                Ok(expected_ty)
            }
            PatternKind::Wildcard => Ok(expected_ty),
            PatternKind::Tuple(patterns) => {
                let tuple_ty = self.pool.add_type(TypeData::Tuple(
                    patterns.iter().map(|_| expected_ty).collect(),
                ));
                self.unify(tuple_ty, expected_ty)?;
                Ok(expected_ty)
            }
            PatternKind::Slice(patterns) => {
                let slice_ty = self.pool.add_type(TypeData::Slice(expected_ty));
                self.unify(slice_ty, expected_ty)?;
                Ok(expected_ty)
            }
        }
    }

    fn insert_var_from_pattern(&mut self, pattern: &Pattern, ty: TypeId) {
        match &pattern.kind {
            PatternKind::Identifier(name) => {
                self.insert_var(name.clone(), ty);
            }
            PatternKind::Tuple(patterns) => {
                for sub_pattern in patterns {
                    self.insert_var_from_pattern(sub_pattern, ty);
                }
            }
            PatternKind::Slice(patterns) => {
                for sub_pattern in patterns {
                    self.insert_var_from_pattern(sub_pattern, ty);
                }
            }
            PatternKind::Wildcard => {}
        }
    }

    pub fn infer_type(&mut self, ty: &Type) -> Result<TypeId, Vec<ChimError>> {
        match &*ty.kind {
            TypeKind::Infer => Ok(self.fresh_type_var()),
            TypeKind::Path(path) => {
                if let Some(segment) = path.segments.first() {
                    match segment.ident.as_ref() {
                        "i32" | "int" => Ok(self.pool.builtin_types.i32),
                        "i64" => Ok(self.pool.builtin_types.i64),
                        "i16" => Ok(self.pool.builtin_types.i16),
                        "i8" => Ok(self.pool.builtin_types.i8),
                        "u32" | "uint" => Ok(self.pool.builtin_types.u32),
                        "u64" => Ok(self.pool.builtin_types.u64),
                        "u16" => Ok(self.pool.builtin_types.u16),
                        "u8" => Ok(self.pool.builtin_types.u8),
                        "f32" | "float" => Ok(self.pool.builtin_types.f32),
                        "f64" => Ok(self.pool.builtin_types.f64),
                        "bool" => Ok(self.pool.builtin_types.bool),
                        "str" | "string" => Ok(self.pool.builtin_types.string),
                        "char" => Ok(self.pool.builtin_types.char),
                        "byte" => Ok(self.pool.builtin_types.byte),
                        "void" | "unit" | "()" => Ok(self.pool.builtin_types.unit),
                        _ => Ok(self.fresh_type_var()),
                    }
                } else {
                    Ok(self.fresh_type_var())
                }
            }
            TypeKind::Tuple(types) => {
                let elem_tys: Result<Vec<_>, _> = types.iter()
                    .map(|t| self.infer_type(t))
                    .collect();
                let elem_tys = elem_tys?;
                Ok(self.pool.add_type(TypeData::Tuple(elem_tys)))
            }
            TypeKind::Array(inner, size) => {
                let inner_ty = self.infer_type(inner)?;
                Ok(self.pool.add_type(TypeData::Array(inner_ty, *size)))
            }
            TypeKind::Slice(inner) => {
                let inner_ty = self.infer_type(inner)?;
                Ok(self.pool.add_type(TypeData::Slice(inner_ty)))
            }
            TypeKind::Pointer(inner, mutability) => {
                let inner_ty = self.infer_type(inner)?;
                Ok(self.pool.add_type(TypeData::Pointer(inner_ty, *mutability)))
            }
            TypeKind::Reference(lifetime, inner, mutability) => {
                let inner_ty = self.infer_type(inner)?;
                Ok(self.pool.add_type(TypeData::Reference(lifetime.clone(), inner_ty, *mutability)))
            }
            TypeKind::Function(func_ty) => {
                let params: Result<Vec<_>, _> = func_ty.params.iter()
                    .map(|t| self.infer_type(t))
                    .collect();
                let params = params?;
                let return_ty = self.infer_type(&func_ty.return_type.unwrap_or(Type {
                    kind: Box::new(TypeKind::Infer),
                    span: ty.span,
                }))?;
                Ok(self.pool.add_type(TypeData::Function(params, Box::new(return_ty), func_ty.is_async)))
            }
        }
    }

    pub fn add_constraint(&mut self, constraint: TypeConstraint) {
        self.constraints.add(constraint);
    }

    pub fn solve_constraints(&mut self) -> Result<(), Vec<ChimError>> {
        for constraint in &self.constraints.constraints {
            match constraint {
                TypeConstraint::Equal(ty1, ty2) => {
                    self.unify(*ty1, *ty2)?;
                }
                TypeConstraint::SubType(ty1, ty2) => {
                    self.unify(*ty1, *ty2)?;
                }
            }
        }
        Ok(())
    }

    pub fn take_pool(self) -> TypePool {
        self.pool
    }
}

impl TypeConstraints {
    pub fn new() -> Self {
        TypeConstraints {
            constraints: Vec::new(),
        }
    }

    pub fn add(&mut self, constraint: TypeConstraint) {
        self.constraints.push(constraint);
    }

    pub fn add_equal(&mut self, left: TypeId, right: TypeId) {
        self.constraints.push(TypeConstraint::Equal(left, right));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_inference() {
        let mut inferencer = TypeInferencer::new();

        let expr = crate::ast::Expr {
            kind: Box::new(crate::ast::ExprKind::Literal(crate::ast::Literal {
                kind: crate::ast::LiteralKind::Int(42),
                span: Span::new(chim_span::FileId(0), 0, 2, 0, 0),
            })),
            span: Span::new(chim_span::FileId(0), 0, 2, 0, 0),
            ty: None,
        };

        let ty = inferencer.infer_expr(&expr);
        assert!(ty.is_ok());
    }

    #[test]
    fn test_binary_op_inference() {
        let mut inferencer = TypeInferencer::new();

        let left = crate::ast::Expr {
            kind: Box::new(crate::ast::ExprKind::Literal(crate::ast::Literal {
                kind: crate::ast::LiteralKind::Int(1),
                span: Span::new(chim_span::FileId(0), 0, 1, 0, 0),
            })),
            span: Span::new(chim_span::FileId(0), 0, 1, 0, 0),
            ty: None,
        };

        let right = crate::ast::Expr {
            kind: Box::new(crate::ast::ExprKind::Literal(crate::ast::Literal {
                kind: crate::ast::LiteralKind::Int(2),
                span: Span::new(chim_span::FileId(0), 4, 5, 0, 4),
            })),
            span: Span::new(chim_span::FileId(0), 4, 5, 0, 4),
            ty: None,
        };

        let ty = inferencer.infer_expr(&left);
        assert!(ty.is_ok());
    }
}
