use crate::type_pool::{TypeId, TypeData, TypePool};
use crate::ChimError;
use chim_span::Span;

/// 边界检查类型
#[derive(Debug, Clone, PartialEq)]
pub enum BoundaryCheckType {
    ArrayIndex,
    SliceIndex,
    StringIndex,
}

/// 边界检查
#[derive(Debug, Clone)]
pub struct BoundaryCheck {
    pub check_type: BoundaryCheckType,
    pub span: Span,
    pub ty: TypeId,
    pub index: TypeId,
}

/// 边界检查器
pub struct BoundaryChecker {
    pool: TypePool,
    checks: Vec<BoundaryCheck>,
}

impl BoundaryChecker {
    pub fn new(pool: TypePool) -> Self {
        BoundaryChecker {
            pool,
            checks: Vec::new(),
        }
    }

    /// 检查表达式
    pub fn check_expr(&mut self, expr: &crate::ast::Expr) -> Result<(), Vec<ChimError>> {
        match &expr.kind {
            crate::ast::ExprKind::Index { object, index } => {
                self.check_index(object, index)
            }
            _ => Ok(()),
        }
    }

    /// 检查索引
    fn check_index(
        &mut self,
        object: &crate::ast::Expr,
        index: &crate::ast::Expr,
    ) -> Result<(), Vec<ChimError>> {
        let obj_ty = self.infer_type(object)?;
        let idx_ty = self.infer_type(index)?;

        self.unify_types(idx_ty, self.pool.builtin_types.usize)?;

        match self.pool.get_type(obj_ty) {
            TypeData::Array(_, length) => {
                self.checks.push(BoundaryCheck {
                    check_type: BoundaryCheckType::ArrayIndex,
                    span: expr.span,
                    ty: obj_ty,
                    index: idx_ty,
                });
            }
            TypeData::Slice(_) => {
                self.checks.push(BoundaryCheck {
                    check_type: BoundaryCheckType::SliceIndex,
                    span: expr.span,
                    ty: obj_ty,
                    index: idx_ty,
                });
            }
            TypeData::Str => {
                self.checks.push(BoundaryCheck {
                    check_type: BoundaryCheckType::StringIndex,
                    span: expr.span,
                    ty: obj_ty,
                    index: idx_ty,
                });
            }
            _ => {
                return Err(vec![ChimError::NotIndexable]);
            }
        }

        Ok(())
    }

    fn infer_type(&mut self, expr: &crate::ast::Expr) -> TypeId {
        self.pool.builtin_types.unit
    }

    fn unify_types(&mut self, ty1: TypeId, ty2: TypeId) -> Result<(), Vec<ChimError>> {
        Ok(())
    }

    /// 生成边界检查代码
    pub fn generate_checks(&self) -> String {
        let mut code = String::new();

        for check in &self.checks {
            match check.check_type {
                BoundaryCheckType::ArrayIndex => {
                    code.push_str(&format!(
                        "if index >= array.len() {{ panic!(\"Array index out of bounds\"); }}\n"
                    ));
                }
                BoundaryCheckType::SliceIndex => {
                    code.push_str(&format!(
                        "if index >= slice.len() {{ panic!(\"Slice index out of bounds\"); }}\n"
                    ));
                }
                BoundaryCheckType::StringIndex => {
                    code.push_str(&format!(
                        "if index >= string.len() {{ panic!(\"String index out of bounds\"); }}\n"
                    ));
                }
            }
        }

        code
    }
}

/// 类型转换检查
pub struct CastChecker {
    pool: TypePool,
    casts: Vec<CastCheck>,
}

/// 类型转换检查
#[derive(Debug, Clone)]
pub struct CastCheck {
    pub from: TypeId,
    pub to: TypeId,
    pub span: Span,
    pub is_safe: bool,
}

impl CastChecker {
    pub fn new(pool: TypePool) -> Self {
        CastChecker {
            pool,
            casts: Vec::new(),
        }
    }

    /// 检查类型转换
    pub fn check_cast(
        &mut self,
        from: TypeId,
        to: TypeId,
        span: Span,
    ) -> Result<(), Vec<ChimError>> {
        let from_data = self.pool.get_type(from);
        let to_data = self.pool.get_type(to);

        let is_safe = self.is_safe_cast(from_data, to_data);

        self.casts.push(CastCheck {
            from,
            to,
            span,
            is_safe,
        });

        if !is_safe {
            Err(vec![ChimError::UnsafeCast {
                from,
                to,
                span,
            }])
        } else {
            Ok(())
        }
    }

    /// 检查是否为安全转换
    fn is_safe_cast(&self, from: &TypeData, to: &TypeData) -> bool {
        match (from, to) {
            (a, b) if a == b => true,

            (TypeData::Int(IntSize::I8), TypeData::Int(IntSize::I16)) => true,
            (TypeData::Int(IntSize::I8), TypeData::Int(IntSize::I32)) => true,
            (TypeData::Int(IntSize::I8), TypeData::Int(IntSize::I64)) => true,
            (TypeData::Int(IntSize::I16), TypeData::Int(IntSize::I32)) => true,
            (TypeData::Int(IntSize::I16), TypeData::Int(IntSize::I64)) => true,
            (TypeData::Int(IntSize::I32), TypeData::Int(IntSize::I64)) => true,

            (TypeData::Uint(UintSize::U8), TypeData::Uint(UintSize::U16)) => true,
            (TypeData::Uint(UintSize::U8), TypeData::Uint(UintSize::U32)) => true,
            (TypeData::Uint(UintSize::U8), TypeData::Uint(UintSize::U64)) => true,
            (TypeData::Uint(UintSize::U16), TypeData::Uint(UintSize::U32)) => true,
            (TypeData::Uint(UintSize::U16), TypeData::Uint(UintSize::U64)) => true,
            (TypeData::Uint(UintSize::U32), TypeData::Uint(UintSize::U64)) => true,

            (TypeData::Float(FloatSize::F32), TypeData::Float(FloatSize::F64)) => true,

            (TypeData::Pointer(_, _), TypeData::Pointer(_, _)) => true,
            (TypeData::Pointer(_, _), TypeData::Uint(UintSize::Usize)) => true,
            (TypeData::Uint(UintSize::Usize), TypeData::Pointer(_, _)) => true,

            (TypeData::Ref(t1, _), TypeData::Ref(t2, _)) if t1 == t2 => true,
            (TypeData::MutRef(t1, _), TypeData::MutRef(t2, _)) if t1 == t2 => true,

            _ => false,
        }
    }
}

/// 线性类型
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum LinearType {
    Linear(TypeId),
    NonLinear(TypeId),
}

/// 使用信息
#[derive(Debug, Clone)]
pub struct UsageInfo {
    pub name: String,
    pub ty: TypeId,
    pub used: bool,
    pub span: Span,
}

/// 线性类型检查器
pub struct LinearTypeChecker {
    pool: TypePool,
    usage: HashMap<String, UsageInfo>,
}

impl LinearTypeChecker {
    pub fn new(pool: TypePool) -> Self {
        LinearTypeChecker {
            pool,
            usage: HashMap::new(),
        }
    }

    /// 检查表达式
    pub fn check_expr(&mut self, expr: &crate::ast::Expr) -> Result<(), Vec<ChimError>> {
        match &expr.kind {
            crate::ast::ExprKind::Identifier(name) => {
                self.check_identifier(name, expr.span)
            }
            crate::ast::ExprKind::Call { func, args } => {
                self.check_call(func, args)
            }
            _ => Ok(()),
        }
    }

    /// 检查标识符
    fn check_identifier(&mut self, name: &str, span: Span) -> Result<(), Vec<ChimError>> {
        if let Some(info) = self.usage.get(name) {
            let ty = self.pool.get_type(info.ty);

            if let TypeData::Infer(_) = ty {
                if info.used {
                    return Err(vec![ChimError::LinearTypeUsedTwice {
                        name: name.to_string(),
                        span,
                    }]);
                }
            }
        }

        Ok(())
    }

    /// 检查函数调用
    fn check_call(
        &mut self,
        func: &crate::ast::Expr,
        args: &[crate::ast::Expr],
    ) -> Result<(), Vec<ChimError>> {
        for arg in args {
            self.check_expr(arg)?;
        }

        for arg in args {
            if let crate::ast::ExprKind::Identifier(name) = &arg.kind {
                if let Some(info) = self.usage.get_mut(name) {
                    let ty = self.pool.get_type(info.ty);
                    if let TypeData::Infer(_) = ty {
                        info.used = true;
                    }
                }
            }
        }

        Ok(())
    }

    /// 检查所有线性类型是否被使用
    pub fn check_all_used(&self) -> Result<(), Vec<ChimError>> {
        let mut errors = Vec::new();

        for info in self.usage.values() {
            let ty = self.pool.get_type(info.ty);
            if let TypeData::Infer(_) = ty {
                if !info.used {
                    errors.push(ChimError::LinearTypeNotUsed {
                        name: info.name.clone(),
                        span: info.span,
                    });
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

/// Null安全类型
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum NullableType {
    NonNull(TypeId),
    Nullable(TypeId),
}

/// Null安全检查器
pub struct NullSafetyChecker {
    pool: TypePool,
    errors: Vec<ChimError>,
}

impl NullSafetyChecker {
    pub fn new(pool: TypePool) -> Self {
        NullSafetyChecker {
            pool,
            errors: Vec::new(),
        }
    }

    /// 检查表达式
    pub fn check_expr(&mut self, expr: &crate::ast::Expr) -> Result<(), Vec<ChimError>> {
        match &expr.kind {
            crate::ast::ExprKind::FieldAccess { object, field } => {
                self.check_field_access(object, field)
            }
            crate::ast::ExprKind::MethodCall { receiver, method, args } => {
                self.check_method_call(receiver, method, args)
            }
            crate::ast::ExprKind::UnaryOp { op, operand } => {
                self.check_unary_op(op, operand)
            }
            _ => Ok(()),
        }
    }

    /// 检查字段访问
    fn check_field_access(
        &mut self,
        object: &crate::ast::Expr,
        field: &str,
    ) -> Result<(), Vec<ChimError>> {
        let obj_ty = self.infer_type(object)?;

        match self.pool.get_type(obj_ty) {
            TypeData::Infer(_) => {
                Ok(())
            }
            TypeData::Struct(struct_id) => {
                let struct_data = self.pool.get_struct(*struct_id);
                if struct_data.fields.iter().any(|f| f.name.as_ref() == field) {
                    Ok(())
                } else {
                    Err(vec![ChimError::FieldNotFound {
                        struct_name: struct_data.name.to_string(),
                        field_name: field.to_string(),
                    }])
                }
            }
            _ => Err(vec![ChimError::NotAStruct]),
        }
    }

    /// 检查方法调用
    fn check_method_call(
        &mut self,
        receiver: &crate::ast::Expr,
        method: &str,
        args: &[crate::ast::Expr],
    ) -> Result<(), Vec<ChimError>> {
        let receiver_ty = self.infer_type(receiver)?;

        match self.pool.get_type(receiver_ty) {
            TypeData::Infer(_) => Ok(()),
            TypeData::Struct(struct_id) => {
                let struct_data = self.pool.get_struct(*struct_id);
                if struct_data.fields.iter().any(|f| f.name.as_ref() == method) {
                    Ok(())
                } else {
                    Err(vec![ChimError::MethodNotFound {
                        struct_name: struct_data.name.to_string(),
                        method_name: method.to_string(),
                    }])
                }
            }
            _ => Err(vec![ChimError::NotAStruct]),
        }
    }

    /// 检查一元操作
    fn check_unary_op(
        &mut self,
        op: &crate::ast::UnaryOperator,
        operand: &crate::ast::Expr,
    ) -> Result<(), Vec<ChimError>> {
        match op {
            crate::ast::UnaryOperator::Deref => {
                let ty = self.infer_type(operand)?;
                self.check_nullable(ty, operand.span)
            }
            _ => Ok(()),
        }
    }

    /// 检查可空类型
    fn check_nullable(&mut self, ty: TypeId, span: Span) -> Result<(), Vec<ChimError>> {
        match self.pool.get_type(ty) {
            TypeData::Infer(_) => Ok(()),
            TypeData::Option(inner) => {
                Err(vec![ChimError::NullableDereference {
                    span,
                    ty,
                }])
            }
            _ => Ok(()),
        }
    }

    fn infer_type(&mut self, expr: &crate::ast::Expr) -> TypeId {
        self.pool.builtin_types.unit
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_boundary_check() {
        let pool = TypePool::new();
        let mut checker = BoundaryChecker::new(pool);

        let expr = crate::ast::Expr {
            kind: Box::new(crate::ast::ExprKind::Index {
                object: Box::new(crate::ast::Expr {
                    kind: Box::new(crate::ast::ExprKind::Identifier("array".to_string())),
                    span: Span::new(chim_span::FileId(0), 0, 5, 0, 0),
                    ty: None,
                }),
                index: Box::new(crate::ast::Expr {
                    kind: Box::new(crate::ast::ExprKind::Literal(crate::ast::Literal {
                        kind: crate::ast::LiteralKind::Int(0),
                        span: Span::new(chim_span::FileId(0), 6, 7, 0, 6),
                    })),
                    span: Span::new(chim_span::FileId(0), 6, 7, 0, 6),
                    ty: None,
                }),
            }),
            span: Span::new(chim_span::FileId(0), 0, 8, 0, 0),
            ty: None,
        };

        let result = checker.check_expr(&expr);
        assert!(result.is_ok());
        assert!(!checker.checks.is_empty());
    }

    #[test]
    fn test_cast_check() {
        let pool = TypePool::new();
        let mut checker = CastChecker::new(pool);

        let i32_type = pool.builtin_types.i32;
        let i64_type = pool.builtin_types.i64;

        let result = checker.check_cast(
            i32_type,
            i64_type,
            Span::new(chim_span::FileId(0), 0, 0, 0, 0),
        );

        assert!(result.is_ok());
        assert!(checker.casts.last().unwrap().is_safe);
    }

    #[test]
    fn test_linear_type_check() {
        let pool = TypePool::new();
        let mut checker = LinearTypeChecker::new(pool);

        let expr = crate::ast::Expr {
            kind: Box::new(crate::ast::ExprKind::Identifier("x".to_string())),
            span: Span::new(chim_span::FileId(0), 0, 1, 0, 0),
            ty: None,
        };

        let result = checker.check_expr(&expr);
        assert!(result.is_ok());
    }

    #[test]
    fn test_null_safety_check() {
        let pool = TypePool::new();
        let mut checker = NullSafetyChecker::new(pool);

        let expr = crate::ast::Expr {
            kind: Box::new(crate::ast::ExprKind::Identifier("obj".to_string())),
            span: Span::new(chim_span::FileId(0), 0, 3, 0, 0),
            ty: None,
        };

        let result = checker.check_expr(&expr);
        assert!(result.is_ok());
    }
}
