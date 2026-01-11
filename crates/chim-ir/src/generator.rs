use crate::*;
use chim_ast::*;
use chim_semantic::{AnalyzedProgram, TypeId};
use chim_span::Span;
use std::collections::HashMap;

pub struct IRGenerator<'a> {
    module: IRModule,
    program: &'a AnalyzedProgram,
    builder: IRBuilder<'a>,
    value_map: HashMap<Ident, ValueId>,
    block_map: HashMap<Ident, BlockId>,
    next_value_id: usize,
    next_block_id: usize,
}

impl<'a> IRGenerator<'a> {
    pub fn new(program: &'a AnalyzedProgram) -> Self {
        let mut module = IRModule {
            functions: Vec::new(),
            globals: Vec::new(),
            structs: Vec::new(),
            enums: Vec::new(),
        };
        
        let builder = IRBuilder::new(&mut module);
        
        IRGenerator {
            module,
            program,
            builder,
            value_map: HashMap::new(),
            block_map: HashMap::new(),
            next_value_id: 0,
            next_block_id: 0,
        }
    }

    pub fn generate_module(&mut self, ast_program: &Program) -> IRModule {
        for item in &ast_program.items {
            self.generate_item(item);
        }
        
        self.module.clone()
    }

    fn generate_item(&mut self, item: &Item) {
        match item {
            Item::Function(func) => self.generate_function(func),
            Item::Struct(struct_def) => self.generate_struct(struct_def),
            Item::Enum(enum_def) => self.generate_enum(enum_def),
            Item::Trait(trait_def) => self.generate_trait(trait_def),
            Item::Impl(impl_def) => self.generate_impl(impl_def),
            Item::Use(_) => {}
            Item::Mod(mod_def) => self.generate_mod(mod_def),
            Item::Extern(extern_block) => self.generate_extern(extern_block),
            Item::Constant(const_def) => self.generate_constant(const_def),
            Item::Static(static_def) => self.generate_static(static_def),
        }
    }

    fn generate_function(&mut self, func: &Function) {
        let ir_func_id = IRFunctionId(self.module.functions.len());
        
        let params: Vec<IRParam> = func.params.iter().enumerate().map(|(i, param)| {
            IRParam {
                id: VarId(i),
                name: param.name.to_string(),
                ty: self.get_type_id(&param.ty),
            }
        }).collect();

        let return_type = func.return_type.as_ref()
            .map(|ty| self.get_type_id(ty))
            .unwrap_or(self.program.pool.builtin_types.unit);

        let ir_func = IRFunction {
            id: ir_func_id,
            name: func.name.to_string(),
            params,
            return_type,
            body: Vec::new(),
            span: func.span,
            is_pub: func.is_pub,
            is_extern: false,
            is_unsafe: false,
        };

        self.module.functions.push(ir_func);
        self.builder = IRBuilder::new(&mut self.module);
        
        let entry_block = self.create_block();
        self.builder.set_current_block(&mut self.module.functions[ir_func_id.0].body[0]);
        
        for (i, param) in func.params.iter().enumerate() {
            let value_id = ValueId(i);
            self.value_map.insert(param.name.clone(), value_id);
        }

        for stmt in &func.body {
            self.generate_stmt(stmt);
        }

        if self.module.functions[ir_func_id.0].body.last()
            .and_then(|block| block.terminator.is_return())
            .unwrap_or(false) == false {
            self.builder.emit_ret_void(func.span);
        }
    }

    fn generate_struct(&mut self, struct_def: &Struct) {
        let ir_struct_id = StructId(self.module.structs.len());
        
        let mut offset = 0usize;
        let fields: Vec<IRStructField> = struct_def.fields.iter().map(|field| {
            let ty_id = self.get_type_id(&field.ty);
            let size = self.program.type_size(ty_id);
            let align = self.program.type_align(ty_id);
            
            offset = (offset + align - 1) / align * align;
            
            let ir_field = IRStructField {
                name: field.name.to_string(),
                ty: ty_id,
                offset,
                size,
            };
            
            offset += size;
            ir_field
        }).collect();

        let size = offset;
        let align = struct_def.fields.iter()
            .map(|field| self.program.type_align(self.get_type_id(&field.ty)))
            .max()
            .unwrap_or(1);

        let ir_struct = IRStruct {
            id: ir_struct_id,
            name: struct_def.name.to_string(),
            fields,
            size,
            align,
            is_packed: false,
        };

        self.module.structs.push(ir_struct);
    }

    fn generate_enum(&mut self, enum_def: &Enum) {
        let ir_enum_id = EnumId(self.module.enums.len());
        
        let mut max_size = 0usize;
        let mut max_align = 1usize;
        
        let variants: Vec<IREnumVariant> = enum_def.variants.iter().enumerate().map(|(i, variant)| {
            let mut offset = 0usize;
            let fields: Vec<IRStructField> = variant.fields.iter().map(|field| {
                let ty_id = self.get_type_id(&field.ty);
                let size = self.program.type_size(ty_id);
                let align = self.program.type_align(ty_id);
                
                offset = (offset + align - 1) / align * align;
                
                let ir_field = IRStructField {
                    name: field.name.to_string(),
                    ty: ty_id,
                    offset,
                    size,
                };
                
                offset += size;
                max_size = max_size.max(offset);
                max_align = max_align.max(align);
                
                ir_field
            }).collect();

            IREnumVariant {
                name: variant.name.to_string(),
                discriminant: i as i128,
                fields,
                size: offset,
                align: max_align,
            }
        }).collect();

        let tag_size = 4;
        let size = max_size.max(tag_size);
        let align = max_align.max(tag_size);

        let ir_enum = IREnum {
            id: ir_enum_id,
            name: enum_def.name.to_string(),
            variants,
            size,
            align,
            tag_repr: TagRepresentation::U32,
        };

        self.module.enums.push(ir_enum);
    }

    fn generate_trait(&mut self, _trait_def: &Trait) {
    }

    fn generate_impl(&mut self, _impl_def: &Impl) {
    }

    fn generate_mod(&mut self, mod_def: &Mod) {
        for item in &mod_def.items {
            self.generate_item(item);
        }
    }

    fn generate_extern(&mut self, _extern_block: &ExternBlock) {
    }

    fn generate_constant(&mut self, _const_def: &Constant) {
    }

    fn generate_static(&mut self, _static_def: &Static) {
    }

    fn generate_stmt(&mut self, stmt: &Stmt) {
        match &stmt.kind {
            StmtKind::Let(let_stmt) => self.generate_let_stmt(let_stmt),
            StmtKind::Var(var_stmt) => self.generate_var_stmt(var_stmt),
            StmtKind::Expr(expr) => {
                self.generate_expr(expr);
            }
            StmtKind::Return(return_stmt) => self.generate_return_stmt(return_stmt, stmt.span),
            StmtKind::Break(break_stmt) => self.generate_break_stmt(break_stmt, stmt.span),
            StmtKind::Continue => self.generate_continue_stmt(stmt.span),
            StmtKind::Loop(loop_stmt) => self.generate_loop_stmt(loop_stmt),
            StmtKind::While(while_stmt) => self.generate_while_stmt(while_stmt),
            StmtKind::For(for_stmt) => self.generate_for_stmt(for_stmt),
            StmtKind::Match(match_stmt) => self.generate_match_stmt(match_stmt),
        }
    }

    fn generate_let_stmt(&mut self, let_stmt: &LetStmt) {
        let value_id = if let Some(init) = &let_stmt.initializer {
            self.generate_expr(init)
        } else {
            self.create_value()
        };

        if let Some(name) = self.get_identifier_from_pattern(&let_stmt.pattern) {
            self.value_map.insert(name, value_id);
        }
    }

    fn generate_var_stmt(&mut self, var_stmt: &VarStmt) {
        let value_id = if let Some(init) = &var_stmt.initializer {
            self.generate_expr(init)
        } else {
            self.create_value()
        };

        if let Some(name) = self.get_identifier_from_pattern(&var_stmt.pattern) {
            self.value_map.insert(name, value_id);
        }
    }

    fn generate_return_stmt(&mut self, return_stmt: &Option<Box<Expr>>, span: Span) {
        let value = if let Some(expr) = return_stmt {
            Some(self.generate_expr(expr))
        } else {
            None
        };
        self.builder.emit_ret(value, span);
    }

    fn generate_break_stmt(&mut self, break_stmt: &Option<Box<Expr>>, span: Span) {
        let value = if let Some(expr) = break_stmt {
            Some(self.generate_expr(expr))
        } else {
            None
        };
        self.builder.emit_br(BlockId(0), span);
    }

    fn generate_continue_stmt(&mut self, span: Span) {
        self.builder.emit_br(BlockId(0), span);
    }

    fn generate_loop_stmt(&mut self, loop_stmt: &LoopStmt) {
        let loop_block = self.create_block();
        let body_block = self.create_block();
        
        self.builder.emit_br(loop_block, loop_stmt.span);
        
        for stmt in &loop_stmt.body {
            self.generate_stmt(stmt);
        }
        
        self.builder.emit_br(loop_block, loop_stmt.span);
    }

    fn generate_while_stmt(&mut self, while_stmt: &WhileStmt) {
        let cond_block = self.create_block();
        let body_block = self.create_block();
        
        self.builder.emit_br(cond_block, while_stmt.span);
        
        let cond_value = self.generate_expr(&while_stmt.condition);
        
        for stmt in &while_stmt.body {
            self.generate_stmt(stmt);
        }
        
        self.builder.emit_cond_br(cond_value, body_block, BlockId(0), while_stmt.span);
    }

    fn generate_for_stmt(&mut self, for_stmt: &ForStmt) {
        let iterable_value = self.generate_expr(&for_stmt.iterable);
        let body_block = self.create_block();
        
        for stmt in &for_stmt.body {
            self.generate_stmt(stmt);
        }
        
        self.builder.emit_br(body_block, for_stmt.span);
    }

    fn generate_match_stmt(&mut self, match_stmt: &MatchStmt) {
        let value = self.generate_expr(&match_stmt.expr);
        
        for arm in &match_stmt.arms {
            let arm_block = self.create_block();
            
            if let Some(guard) = &arm.guard {
                let guard_value = self.generate_expr(guard);
            }
            
            let body_value = self.generate_expr(&arm.body);
            
            self.builder.emit_br(BlockId(0), match_stmt.span);
        }
    }

    fn generate_expr(&mut self, expr: &Expr) -> ValueId {
        match &*expr.kind {
            ExprKind::Identifier(name) => {
                *self.value_map.get(&name).unwrap_or(&ValueId(0))
            }
            ExprKind::Literal(lit) => self.generate_literal(lit, expr.span),
            ExprKind::Binary(bin_expr) => self.generate_binary_expr(bin_expr, expr.span),
            ExprKind::Unary(unary_expr) => self.generate_unary_expr(unary_expr, expr.span),
            ExprKind::Call(call_expr) => self.generate_call_expr(call_expr, expr.span),
            ExprKind::Block(block_expr) => self.generate_block_expr(block_expr),
            ExprKind::If(if_expr) => self.generate_if_expr(if_expr, expr.span),
            ExprKind::Match(match_expr) => self.generate_match_expr(match_expr, expr.span),
            ExprKind::Closure(closure_expr) => self.generate_closure_expr(closure_expr, expr.span),
            ExprKind::FieldAccess(field_access) => self.generate_field_access(field_access, expr.span),
            ExprKind::Index(index_expr) => self.generate_index_expr(index_expr, expr.span),
            ExprKind::Cast(cast_expr) => self.generate_cast_expr(cast_expr, expr.span),
            ExprKind::Assign(assign_expr) => self.generate_assign_expr(assign_expr, expr.span),
            ExprKind::AssignOp(assign_op_expr) => self.generate_assign_op_expr(assign_op_expr, expr.span),
            ExprKind::Range(range_expr) => self.generate_range_expr(range_expr, expr.span),
            ExprKind::Array(array_expr) => self.generate_array_expr(array_expr),
            ExprKind::Ternary(ternary_expr) => self.generate_ternary_expr(ternary_expr, expr.span),
        }
    }

    fn generate_literal(&mut self, lit: &Literal, span: Span) -> ValueId {
        let dest = self.create_value();
        let ty = match lit.kind {
            LiteralKind::Int(_) => self.program.pool.builtin_types.i32,
            LiteralKind::Float(_) => self.program.pool.builtin_types.f64,
            LiteralKind::String(_) | LiteralKind::RawString(_) => self.program.pool.builtin_types.string,
            LiteralKind::Char(_) => self.program.pool.builtin_types.char,
            LiteralKind::Byte(_) => self.program.pool.builtin_types.byte,
            LiteralKind::Bool(_) => self.program.pool.builtin_types.bool,
            LiteralKind::Null => self.program.pool.builtin_types.null,
            LiteralKind::Unit => self.program.pool.builtin_types.unit,
            LiteralKind::ByteString(_) => self.program.pool.builtin_types.bytes,
        };
        dest
    }

    fn generate_binary_expr(&mut self, bin_expr: &BinaryExpr, span: Span) -> ValueId {
        let left = self.generate_expr(&bin_expr.left);
        let right = self.generate_expr(&bin_expr.right);
        let dest = self.create_value();
        
        let op = match bin_expr.op {
            BinOp::Add => BinaryOp::Add,
            BinOp::Sub => BinaryOp::Sub,
            BinOp::Mul => BinaryOp::Mul,
            BinOp::Div => BinaryOp::Div,
            BinOp::Mod => BinaryOp::Rem,
            BinOp::And => BinaryOp::And,
            BinOp::Or => BinaryOp::Or,
            BinOp::BitAnd => BinaryOp::And,
            BinOp::BitOr => BinaryOp::Or,
            BinOp::BitXor => BinaryOp::Xor,
            BinOp::Shl => BinaryOp::Shl,
            BinOp::Shr => BinaryOp::Shr,
            BinOp::Eq => BinaryOp::Add,
            BinOp::Ne => BinaryOp::Sub,
            BinOp::Lt => BinaryOp::Mul,
            BinOp::Le => BinaryOp::Div,
            BinOp::Gt => BinaryOp::Rem,
            BinOp::Ge => BinaryOp::Shl,
        };

        let ty = self.program.pool.builtin_types.i32;
        self.builder.emit_binary(dest, op, left, right, ty, span);
        dest
    }

    fn generate_unary_expr(&mut self, unary_expr: &UnaryExpr, span: Span) -> ValueId {
        let operand = self.generate_expr(&unary_expr.expr);
        let dest = self.create_value();
        
        let op = match unary_expr.op {
            UnOp::Neg => UnaryOp::Neg,
            UnOp::Not => UnaryOp::Not,
            UnOp::FNeg => UnaryOp::FNeg,
            UnOp::Ref | UnOp::RefMut | UnOp::Deref => UnaryOp::Neg,
        };

        let ty = self.program.pool.builtin_types.i32;
        self.builder.emit_unary(dest, op, operand, ty, span);
        dest
    }

    fn generate_call_expr(&mut self, call_expr: &CallExpr, span: Span) -> ValueId {
        let func = self.generate_expr(&call_expr.func);
        let args: smallvec::SmallVec<[ValueId; 4]> = call_expr.args.iter()
            .map(|arg| self.generate_expr(arg))
            .collect();
        
        let dest = self.create_value();
        let ty = self.program.pool.builtin_types.i32;
        self.builder.emit_call(Some(dest), func, args, ty, span);
        dest
    }

    fn generate_block_expr(&mut self, block_expr: &BlockExpr) -> ValueId {
        for stmt in &block_expr.stmts {
            self.generate_stmt(stmt);
        }
        self.create_value()
    }

    fn generate_if_expr(&mut self, if_expr: &IfExpr, span: Span) -> ValueId {
        let cond = self.generate_expr(&if_expr.condition);
        let then_block = self.create_block();
        let else_block = self.create_block();
        
        let then_value = self.generate_expr(&if_expr.then_branch);
        let else_value = if let Some(else_branch) = &if_expr.else_branch {
            self.generate_expr(else_branch)
        } else {
            self.create_value()
        };
        
        let dest = self.create_value();
        let ty = self.program.pool.builtin_types.i32;
        self.builder.emit_select(dest, cond, then_value, else_value, ty, span);
        dest
    }

    fn generate_match_expr(&mut self, match_expr: &MatchExpr, span: Span) -> ValueId {
        let value = self.generate_expr(&match_expr.expr);
        let dest = self.create_value();
        
        for arm in &match_expr.arms {
            let arm_block = self.create_block();
            
            if let Some(guard) = &arm.guard {
                let guard_value = self.generate_expr(guard);
            }
            
            let body_value = self.generate_expr(&arm.body);
        }
        
        dest
    }

    fn generate_closure_expr(&mut self, closure_expr: &ClosureExpr, span: Span) -> ValueId {
        self.create_value()
    }

    fn generate_field_access(&mut self, field_access: &FieldAccessExpr, span: Span) -> ValueId {
        let obj = self.generate_expr(&field_access.expr);
        let dest = self.create_value();
        
        let indices: smallvec::SmallVec<[ValueId; 4]> = smallvec::smallvec![
            ValueId(0),
        ];
        
        let ty = self.program.pool.builtin_types.i32;
        self.builder.emit_get_element_ptr(dest, obj, indices, ty, span);
        self.builder.emit_load(dest, dest, ty, span);
        dest
    }

    fn generate_index_expr(&mut self, index_expr: &IndexExpr, span: Span) -> ValueId {
        let array = self.generate_expr(&index_expr.expr);
        let index = self.generate_expr(&index_expr.index);
        let dest = self.create_value();
        
        let indices: smallvec::SmallVec<[ValueId; 4]> = smallvec::smallvec![
            index,
        ];
        
        let ty = self.program.pool.builtin_types.i32;
        self.builder.emit_get_element_ptr(dest, array, indices, ty, span);
        self.builder.emit_load(dest, dest, ty, span);
        dest
    }

    fn generate_cast_expr(&mut self, cast_expr: &CastExpr, span: Span) -> ValueId {
        let value = self.generate_expr(&cast_expr.expr);
        let dest = self.create_value();
        
        let to_ty = self.get_type_id(&cast_expr.ty);
        let op = CastOp::BitCast;
        
        self.builder.emit_cast(dest, value, to_ty, op, span);
        dest
    }

    fn generate_assign_expr(&mut self, assign_expr: &AssignExpr, span: Span) -> ValueId {
        let left = self.generate_expr(&assign_expr.left);
        let right = self.generate_expr(&assign_expr.right);
        
        let ty = self.program.pool.builtin_types.i32;
        self.builder.emit_store(left, right, ty, span);
        right
    }

    fn generate_assign_op_expr(&mut self, assign_op_expr: &AssignOpExpr, span: Span) -> ValueId {
        let left = self.generate_expr(&assign_op_expr.left);
        let right = self.generate_expr(&assign_op_expr.right);
        let dest = self.create_value();
        
        let op = match assign_op_expr.op {
            BinOp::Add => BinaryOp::Add,
            BinOp::Sub => BinaryOp::Sub,
            BinOp::Mul => BinaryOp::Mul,
            BinOp::Div => BinaryOp::Div,
            BinOp::Mod => BinaryOp::Rem,
            BinOp::And => BinaryOp::And,
            BinOp::Or => BinaryOp::Or,
            BinOp::BitAnd => BinaryOp::And,
            BinOp::BitOr => BinaryOp::Or,
            BinOp::BitXor => BinaryOp::Xor,
            BinOp::Shl => BinaryOp::Shl,
            BinOp::Shr => BinaryOp::Shr,
            BinOp::Eq => BinaryOp::Add,
            BinOp::Ne => BinaryOp::Sub,
            BinOp::Lt => BinaryOp::Mul,
            BinOp::Le => BinaryOp::Div,
            BinOp::Gt => BinaryOp::Rem,
            BinOp::Ge => BinaryOp::Shl,
        };

        let ty = self.program.pool.builtin_types.i32;
        self.builder.emit_binary(dest, op, left, right, ty, span);
        self.builder.emit_store(left, dest, ty, span);
        dest
    }

    fn generate_range_expr(&mut self, range_expr: &RangeExpr, span: Span) -> ValueId {
        self.create_value()
    }

    fn generate_array_expr(&mut self, array_expr: &ArrayExpr, span: Span) -> ValueId {
        let dest = self.create_value();
        
        for elem in &array_expr.elements {
            let elem_value = self.generate_expr(elem);
        }
        
        dest
    }

    fn generate_ternary_expr(&mut self, ternary_expr: &TernaryExpr, span: Span) -> ValueId {
        let cond = self.generate_expr(&ternary_expr.condition);
        let then_value = self.generate_expr(&ternary_expr.then_branch);
        let else_value = self.generate_expr(&ternary_expr.else_branch);
        let dest = self.create_value();
        
        let ty = self.program.pool.builtin_types.i32;
        self.builder.emit_select(dest, cond, then_value, else_value, ty, span);
        dest
    }

    fn get_type_id(&self, ty: &Type) -> TypeId {
        match &*ty.kind {
            TypeKind::Infer => self.program.pool.builtin_types.unit,
            TypeKind::Path(path) => {
                if let Some(segment) = path.segments.first() {
                    match segment.ident.as_ref() {
                        "i32" | "int" => self.program.pool.builtin_types.i32,
                        "i64" => self.program.pool.builtin_types.i64,
                        "f32" | "float" => self.program.pool.builtin_types.f32,
                        "f64" => self.program.pool.builtin_types.f64,
                        "bool" => self.program.pool.builtin_types.bool,
                        "str" | "string" => self.program.pool.builtin_types.string,
                        _ => self.program.pool.builtin_types.unit,
                    }
                } else {
                    self.program.pool.builtin_types.unit
                }
            }
            _ => self.program.pool.builtin_types.unit,
        }
    }

    fn get_identifier_from_pattern(&self, pattern: &Pattern) -> Option<Ident> {
        match &pattern.kind {
            PatternKind::Identifier(name) => Some(name.clone()),
            _ => None,
        }
    }

    fn create_value(&mut self) -> ValueId {
        let id = ValueId(self.next_value_id);
        self.next_value_id += 1;
        id
    }

    fn create_block(&mut self) -> BlockId {
        let id = BlockId(self.next_block_id);
        self.next_block_id += 1;
        id
    }
}

pub fn generate_ir(ast_program: &Program, analyzed_program: &AnalyzedProgram) -> IRModule {
    let mut generator = IRGenerator::new(analyzed_program);
    generator.generate_module(ast_program)
}