use crate::type_pool::{TypeId, TypeData, TypePool, IntSize, UintSize, FloatSize, Mutability, FunctionSig, FunctionData};
use crate::ChimError;
use chim_span::Span;
use std::collections::HashMap;

/// 类型约束
#[derive(Debug, Clone)]
pub enum TypeConstraint {
    Equal(TypeId, TypeId),
    SubType(TypeId, TypeId),
}

/// 类型变量
#[derive(Debug, Clone)]
pub struct TypeVar {
    pub id: usize,
    pub kind: TypeKind,
    pub constraints: Vec<TypeConstraint>,
    pub origin: Option<Span>,
}

/// 类型种类
#[derive(Debug, Clone, PartialEq)]
pub enum TypeKind {
    Mono,
    Poly(Vec<Kind>),
    Star,
}

/// 种类
#[derive(Debug, Clone, PartialEq)]
pub enum Kind {
    Type,
    Row,
    Effect,
}

/// 替换
#[derive(Debug, Clone)]
pub struct Substitution {
    pub mapping: HashMap<usize, TypeId>,
}

impl Substitution {
    pub fn new() -> Self {
        Substitution {
            mapping: HashMap::new(),
        }
    }

    pub fn apply(&self, ty: TypeId) -> TypeId {
        ty
    }

    pub fn compose(&self, other: Substitution) -> Substitution {
        let mut mapping = self.mapping.clone();
        for (var, ty) in other.mapping {
            mapping.insert(var, self.apply(ty));
        }
        Substitution { mapping }
    }
}

/// 推导配置
#[derive(Debug, Clone)]
pub struct InferenceConfig {
    pub enable_generalization: bool,
    pub enable_instantiation: bool,
    pub enable_polymorphism: bool,
}

impl Default for InferenceConfig {
    fn default() -> Self {
        InferenceConfig {
            enable_generalization: true,
            enable_instantiation: true,
            enable_polymorphism: true,
        }
    }
}

/// 增强的类型推导器
pub struct EnhancedTypeInferencer {
    pool: TypePool,
    type_vars: HashMap<usize, TypeVar>,
    constraints: TypeConstraints,
    substitution: Substitution,
    errors: Vec<ChimError>,
    config: InferenceConfig,
}

impl EnhancedTypeInferencer {
    pub fn new(config: InferenceConfig) -> Self {
        EnhancedTypeInferencer {
            pool: TypePool::new(),
            type_vars: HashMap::new(),
            constraints: TypeConstraints::new(),
            substitution: Substitution::new(),
            errors: Vec::new(),
            config,
        }
    }

    /// 推导表达式类型
    pub fn infer_expr(&mut self, expr: &crate::ast::Expr) -> Result<TypeId, Vec<ChimError>> {
        match &expr.kind {
            crate::ast::ExprKind::Literal(lit) => self.infer_literal(lit),
            crate::ast::ExprKind::Identifier(name) => self.infer_identifier(name),
            crate::ast::ExprKind::BinaryOp { op, left, right } => {
                self.infer_binary_op(op, left, right)
            }
            crate::ast::ExprKind::UnaryOp { op, operand } => {
                self.infer_unary_op(op, operand)
            }
            crate::ast::ExprKind::Call { func, args } => {
                self.infer_call(func, args)
            }
            crate::ast::ExprKind::Lambda { params, body, .. } => {
                self.infer_lambda(params, body)
            }
            crate::ast::ExprKind::If { condition, then_branch, else_branch } => {
                self.infer_if(condition, then_branch, else_branch)
            }
            crate::ast::ExprKind::Match { scrutinee, arms } => {
                self.infer_match(scrutinee, arms)
            }
            crate::ast::ExprKind::Block(stmts) => {
                self.infer_block(stmts)
            }
            _ => Err(vec![ChimError::UnsupportedExpression]),
        }
    }

    /// 推导字面量类型
    fn infer_literal(&mut self, lit: &crate::ast::Literal) -> Result<TypeId, Vec<ChimError>> {
        match &lit.kind {
            crate::ast::LiteralKind::Int(_) => Ok(self.pool.builtin_types.i32),
            crate::ast::LiteralKind::Float(_) => Ok(self.pool.builtin_types.f64),
            crate::ast::LiteralKind::Bool(_) => Ok(self.pool.builtin_types.bool),
            crate::ast::LiteralKind::String(_) => Ok(self.pool.builtin_types.str),
            crate::ast::LiteralKind::Char(_) => Ok(self.pool.builtin_types.char),
        }
    }

    /// 推导标识符类型
    fn infer_identifier(&mut self, name: &str) -> Result<TypeId, Vec<ChimError>> {
        Err(vec![ChimError::UndefinedVariable(name.to_string())])
    }

    /// 推导二元操作类型
    fn infer_binary_op(
        &mut self,
        op: &crate::ast::BinaryOperator,
        left: &crate::ast::Expr,
        right: &crate::ast::Expr,
    ) -> Result<TypeId, Vec<ChimError>> {
        let left_ty = self.infer_expr(left)?;
        let right_ty = self.infer_expr(right)?;

        self.unify_types(left_ty, right_ty)?;

        match op {
            crate::ast::BinaryOperator::Add
            | crate::ast::BinaryOperator::Sub
            | crate::ast::BinaryOperator::Mul
            | crate::ast::BinaryOperator::Div => Ok(left_ty),
            crate::ast::BinaryOperator::Eq
            | crate::ast::BinaryOperator::Ne
            | crate::ast::BinaryOperator::Lt
            | crate::ast::BinaryOperator::Le
            | crate::ast::BinaryOperator::Gt
            | crate::ast::BinaryOperator::Ge => Ok(self.pool.builtin_types.bool),
            crate::ast::BinaryOperator::And | crate::ast::BinaryOperator::Or => Ok(self.pool.builtin_types.bool),
        }
    }

    /// 推导一元操作类型
    fn infer_unary_op(
        &mut self,
        op: &crate::ast::UnaryOperator,
        operand: &crate::ast::Expr,
    ) -> Result<TypeId, Vec<ChimError>> {
        let ty = self.infer_expr(operand)?;
        Ok(ty)
    }

    /// 推导函数调用类型
    fn infer_call(
        &mut self,
        func: &crate::ast::Expr,
        args: &[crate::ast::Expr],
    ) -> Result<TypeId, Vec<ChimError>> {
        let func_ty = self.infer_expr(func)?;
        let arg_tys: Result<Vec<_>, _> = args.iter()
            .map(|arg| self.infer_expr(arg))
            .collect();

        let arg_tys = arg_tys?;

        match self.pool.get_type(func_ty) {
            TypeData::Function(func_id) => {
                let func_sig = self.pool.get_function(*func_id);
                self.unify_function_args(&func_sig.params, &arg_tys)?;
                Ok(func_sig.return_type)
            }
            _ => Err(vec![ChimError::NotAFunction]),
        }
    }

    /// 推导Lambda类型
    fn infer_lambda(
        &mut self,
        params: &[crate::ast::Param],
        body: &crate::ast::Expr,
    ) -> Result<TypeId, Vec<ChimError>> {
        let param_tys: Result<Vec<_>, _> = params.iter()
            .map(|p| self.infer_type(&p.ty))
            .collect();

        let param_tys = param_tys?;
        let return_ty = self.infer_expr(body)?;

        let func_sig = FunctionSig {
            params: param_tys,
            return_type: return_ty,
            is_async: false,
        };

        let func_id = self.pool.intern_function(FunctionData {
            name: std::sync::Arc::from("lambda"),
            sig: func_sig.clone(),
            is_generic: false,
        });

        Ok(self.pool.intern_type(TypeData::Function(func_id)))
    }

    /// 推导If表达式类型
    fn infer_if(
        &mut self,
        condition: &crate::ast::Expr,
        then_branch: &crate::ast::Expr,
        else_branch: &Option<Box<crate::ast::Expr>>,
    ) -> Result<TypeId, Vec<ChimError>> {
        let cond_ty = self.infer_expr(condition)?;
        self.unify_types(cond_ty, self.pool.builtin_types.bool)?;

        let then_ty = self.infer_expr(then_branch)?;
        let else_ty = match else_branch {
            Some(else_expr) => self.infer_expr(else_expr)?,
            None => self.pool.builtin_types.unit,
        };

        self.unify_types(then_ty, else_ty)
    }

    /// 推导Match表达式类型
    fn infer_match(
        &mut self,
        scrutinee: &crate::ast::Expr,
        arms: &[crate::ast::MatchArm],
    ) -> Result<TypeId, Vec<ChimError>> {
        let scrutinee_ty = self.infer_expr(scrutinee)?;

        let mut arm_tys = Vec::new();
        for arm in arms {
            let arm_ty = self.infer_expr(&arm.body)?;
            arm_tys.push(arm_ty);
        }

        let first_ty = arm_tys.first()
            .ok_or_else(|| vec![ChimError::EmptyMatch])?
            .clone();

        for ty in arm_tys.iter().skip(1) {
            self.unify_types(first_ty, *ty)?;
        }

        Ok(first_ty)
    }

    /// 推导Block类型
    fn infer_block(&mut self, stmts: &[crate::ast::Stmt]) -> Result<TypeId, Vec<ChimError>> {
        let mut last_ty = self.pool.builtin_types.unit;

        for stmt in stmts {
            last_ty = self.infer_stmt(stmt)?;
        }

        Ok(last_ty)
    }

    /// 推导语句类型
    pub fn infer_stmt(&mut self, stmt: &crate::ast::Stmt) -> Result<TypeId, Vec<ChimError>> {
        match &stmt.kind {
            crate::ast::StmtKind::Let { .. } => Ok(self.pool.builtin_types.unit),
            crate::ast::StmtKind::Assign { .. } => Ok(self.pool.builtin_types.unit),
            crate::ast::StmtKind::Expr(expr) => self.infer_expr(expr),
            crate::ast::StmtKind::Return(expr) => {
                match expr {
                    Some(e) => self.infer_expr(e),
                    None => Ok(self.pool.builtin_types.unit),
                }
            }
            _ => Ok(self.pool.builtin_types.unit),
        }
    }

    /// 推导类型
    pub fn infer_type(&mut self, ty: &crate::ast::Type) -> Result<TypeId, Vec<ChimError>> {
        match &ty.kind {
            crate::ast::TypeKind::Named(name) => {
                Err(vec![ChimError::UndefinedType(name.clone())])
            }
            crate::ast::TypeKind::Array { element, length } => {
                let element_ty = self.infer_type(element)?;
                Ok(self.pool.intern_type(TypeData::Array(element_ty, *length)))
            }
            crate::ast::TypeKind::Tuple(types) => {
                let type_ids: Result<Vec<_>, _> = types.iter()
                    .map(|t| self.infer_type(t))
                    .collect();
                let type_ids = type_ids?;
                Ok(self.pool.intern_type(TypeData::Tuple(type_ids)))
            }
            crate::ast::TypeKind::Function { params, return_type } => {
                let param_tys: Result<Vec<_>, _> = params.iter()
                    .map(|p| self.infer_type(p))
                    .collect();
                let param_tys = param_tys?;
                let return_ty = self.infer_type(return_type)?;
                let func_sig = FunctionSig {
                    params: param_tys,
                    return_type,
                    is_async: false,
                };
                let func_id = self.pool.intern_function(FunctionData {
                    name: std::sync::Arc::from("function"),
                    sig: func_sig.clone(),
                    is_generic: false,
                });
                Ok(self.pool.intern_type(TypeData::Function(func_id)))
            }
            _ => Err(vec![ChimError::UnsupportedType]),
        }
    }

    /// 统一类型
    pub fn unify_types(&mut self, ty1: TypeId, ty2: TypeId) -> Result<(), Vec<ChimError>> {
        let ty1_data = self.pool.get_type(ty1);
        let ty2_data = self.pool.get_type(ty2);

        match (ty1_data, ty2_data) {
            (TypeData::Infer(kind1), TypeData::Infer(kind2)) => {
                self.unify_type_vars(ty1, ty2, kind1, kind2)
            }
            (TypeData::Infer(_), _) => {
                self.substitute_type_var(ty1, ty2)
            }
            (_, TypeData::Infer(_)) => {
                self.substitute_type_var(ty2, ty1)
            }
            _ => {
                if ty1 == ty2 {
                    Ok(())
                } else {
                    Err(vec![ChimError::TypeMismatch {
                        expected: ty1,
                        found: ty2,
                    }])
                }
            }
        }
    }

    /// 统一类型变量
    fn unify_type_vars(
        &mut self,
        var1: TypeId,
        var2: TypeId,
        kind1: &crate::type_pool::InferKind,
        kind2: &crate::type_pool::InferKind,
    ) -> Result<(), Vec<ChimError>> {
        if var1 == var2 {
            Ok(())
        } else {
            if kind1 == kind2 {
                self.substitution.mapping.insert(var1.0, var2);
                Ok(())
            } else {
                Err(vec![ChimError::KindMismatch])
            }
        }
    }

    /// 替换类型变量
    fn substitute_type_var(&mut self, var: TypeId, ty: TypeId) -> Result<(), Vec<ChimError>> {
        if self.occurs_in(var, ty) {
            Err(vec![ChimError::OccursCheckFailed])
        } else {
            self.substitution.mapping.insert(var.0, ty);
            Ok(())
        }
    }

    /// 检查出现
    fn occurs_in(&self, var: TypeId, ty: TypeId) -> bool {
        let ty_data = self.pool.get_type(ty);
        match ty_data {
            TypeData::Infer(_) => ty == var,
            TypeData::Array(element, _) => self.occurs_in(var, *element),
            TypeData::Tuple(types) => types.iter().any(|t| self.occurs_in(var, *t)),
            TypeData::Function(func_id) => {
                let func_sig = self.pool.get_function(*func_id);
                func_sig.params.iter().any(|p| self.occurs_in(var, *p))
                    || self.occurs_in(var, func_sig.return_type)
            }
            _ => false,
        }
    }

    /// 统一函数参数
    fn unify_function_args(
        &mut self,
        expected: &[TypeId],
        found: &[TypeId],
    ) -> Result<(), Vec<ChimError>> {
        if expected.len() != found.len() {
            return Err(vec![ChimError::ArityMismatch {
                expected: expected.len(),
                found: found.len(),
            }]);
        }

        for (exp, fnd) in expected.iter().zip(found.iter()) {
            self.unify_types(*exp, *fnd)?;
        }

        Ok(())
    }

    /// 求解约束
    pub fn solve_constraints(&mut self) -> Result<(), Vec<ChimError>> {
        for constraint in &self.constraints.constraints {
            match constraint {
                TypeConstraint::Equal(ty1, ty2) => {
                    self.unify_types(*ty1, *ty2)?;
                }
                TypeConstraint::SubType(ty1, ty2) => {
                    self.check_subtype(*ty1, *ty2)?;
                }
            }
        }

        self.apply_substitution();

        Ok(())
    }

    /// 检查子类型
    fn check_subtype(&mut self, ty1: TypeId, ty2: TypeId) -> Result<(), Vec<ChimError>> {
        let ty1_data = self.pool.get_type(ty1);
        let ty2_data = self.pool.get_type(ty2);

        match (ty1_data, ty2_data) {
            (TypeData::MutRef(t1, _), TypeData::Ref(t2, _)) => {
                self.unify_types(*t1, *t2)
            }
            _ => {
                if ty1 != ty2 {
                    Err(vec![ChimError::NotSubtype {
                        subtype: ty1,
                        supertype: ty2,
                    }])
                } else {
                    Ok(())
                }
            }
        }
    }

    /// 应用替换
    fn apply_substitution(&mut self) {
        for (var, ty) in &self.substitution.mapping {
            // 应用替换到所有类型变量
        }
    }
}

/// 类型约束集合
#[derive(Debug, Clone)]
pub struct TypeConstraints {
    pub constraints: Vec<TypeConstraint>,
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
    fn test_type_inference() {
        let config = InferenceConfig::default();
        let mut inferencer = EnhancedTypeInferencer::new(config);

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
        let config = InferenceConfig::default();
        let mut inferencer = EnhancedTypeInferencer::new(config);

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

        let left_ty = inferencer.infer_expr(&left);
        let right_ty = inferencer.infer_expr(&right);

        assert!(left_ty.is_ok());
        assert!(right_ty.is_ok());
    }
}
