use chim_semantic::{TypeId, TypeData, StructId, EnumId, FunctionId, VarId};
use chim_span::Span;
use smallvec::SmallVec;
use std::sync::Arc;

pub mod generator;

pub use generator::{IRGenerator, generate_ir};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ValueId(usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockId(usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FunctionId as IRFunctionId(usize);

#[derive(Debug, Clone, PartialEq)]
pub struct IRModule {
    pub functions: Vec<IRFunction>,
    pub globals: Vec<Global>,
    pub structs: Vec<IRStruct>,
    pub enums: Vec<IREnum>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct IRFunction {
    pub id: FunctionId,
    pub name: String,
    pub params: Vec<IRParam>,
    pub return_type: TypeId,
    pub body: Vec<BasicBlock>,
    pub span: Span,
    pub is_pub: bool,
    pub is_extern: bool,
    pub is_unsafe: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct IRParam {
    pub id: VarId,
    pub name: String,
    pub ty: TypeId,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BasicBlock {
    pub id: BlockId,
    pub instructions: Vec<IRInst>,
    pub terminator: Terminator,
    pub predecessors: Vec<BlockId>,
    pub successors: Vec<BlockId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MemoryOrder {
    Relaxed,
    Consume,
    Acquire,
    Release,
    AcqRel,
    SeqCst,
    HappensBefore,
    Volatile,
    MemoryBarrier,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CaptureKind {
    CaptureRef,
    CaptureValue,
}

#[derive(Debug, Clone, PartialEq)]
pub enum IRInst {
    Alloca {
        dest: VarId,
        ty: TypeId,
        span: Span,
    },
    Load {
        dest: VarId,
        src: ValueId,
        ty: TypeId,
        span: Span,
    },
    Store {
        dest: ValueId,
        src: ValueId,
        ty: TypeId,
        span: Span,
    },
    GetElementPtr {
        dest: VarId,
        ptr: ValueId,
        indices: SmallVec<[ValueId; 4]>,
        ty: TypeId,
        span: Span,
    },
    Binary {
        dest: VarId,
        op: BinaryOp,
        left: ValueId,
        right: ValueId,
        ty: TypeId,
        span: Span,
    },
    Unary {
        dest: VarId,
        op: UnaryOp,
        operand: ValueId,
        ty: TypeId,
        span: Span,
    },
    Call {
        dest: Option<VarId>,
        func: ValueId,
        args: SmallVec<[ValueId; 4]>,
        ty: TypeId,
        span: Span,
    },
    Invoke {
        dest: Option<VarId>,
        func: ValueId,
        args: SmallVec<[ValueId; 4]>,
        normal_block: BlockId,
        unwind_block: BlockId,
        ty: TypeId,
        span: Span,
    },
    Br {
        target: BlockId,
        span: Span,
    },
    CondBr {
        condition: ValueId,
        true_block: BlockId,
        false_block: BlockId,
        span: Span,
    },
    Ret {
        value: Option<ValueId>,
        span: Span,
    },
    RetVoid {
        span: Span,
    },
    Switch {
        value: ValueId,
        default_block: BlockId,
        cases: Vec<(ValueId, BlockId)>,
        span: Span,
    },
    Select {
        dest: VarId,
        condition: ValueId,
        true_val: ValueId,
        false_val: ValueId,
        ty: TypeId,
        span: Span,
    },
    ExtractValue {
        dest: VarId,
        aggregate: ValueId,
        indices: SmallVec<[u32; 4]>,
        ty: TypeId,
        span: Span,
    },
    InsertValue {
        dest: VarId,
        aggregate: ValueId,
        value: ValueId,
        indices: SmallVec<[u32; 4]>,
        ty: TypeId,
        span: Span,
    },
    Cast {
        dest: VarId,
        value: ValueId,
        to_ty: TypeId,
        op: CastOp,
        span: Span,
    },
    Phi {
        dest: VarId,
        values: SmallVec<[(BlockId, ValueId); 4]>,
        ty: TypeId,
        span: Span,
    },
    Skip {
        span: Span,
    },
    Debug {
        location: String,
        span: Span,
    },
    AtomicLoad {
        dest: VarId,
        src: ValueId,
        order: MemoryOrder,
        ty: TypeId,
        span: Span,
    },
    AtomicStore {
        dest: ValueId,
        src: ValueId,
        order: MemoryOrder,
        ty: TypeId,
        span: Span,
    },
    AtomicFetchAdd {
        dest: VarId,
        src: ValueId,
        value: ValueId,
        order: MemoryOrder,
        ty: TypeId,
        span: Span,
    },
    AtomicFetchSub {
        dest: VarId,
        src: ValueId,
        value: ValueId,
        order: MemoryOrder,
        ty: TypeId,
        span: Span,
    },
    AtomicFetchAnd {
        dest: VarId,
        src: ValueId,
        value: ValueId,
        order: MemoryOrder,
        ty: TypeId,
        span: Span,
    },
    AtomicFetchOr {
        dest: VarId,
        src: ValueId,
        value: ValueId,
        order: MemoryOrder,
        ty: TypeId,
        span: Span,
    },
    AtomicFetchXor {
        dest: VarId,
        src: ValueId,
        value: ValueId,
        order: MemoryOrder,
        ty: TypeId,
        span: Span,
    },
    AtomicCompareExchange {
        dest: VarId,
        src: ValueId,
        expected: ValueId,
        desired: ValueId,
        success_order: MemoryOrder,
        failure_order: MemoryOrder,
        ty: TypeId,
        span: Span,
    },
    AtomicExchange {
        dest: VarId,
        src: ValueId,
        value: ValueId,
        order: MemoryOrder,
        ty: TypeId,
        span: Span,
    },
    AtomicFence {
        order: MemoryOrder,
        span: Span,
    },
    Wait {
        atomic: ValueId,
        timeout: Option<ValueId>,
        span: Span,
    },
    Notify {
        atomic: ValueId,
        span: Span,
    },
    NotifyAll {
        atomic: ValueId,
        span: Span,
    },
    MemoryBarrier {
        span: Span,
    },
    DataDependency {
        src: ValueId,
        dest: ValueId,
        span: Span,
    },
    MacroExpand {
        dest: VarId,
        macro_name: ValueId,
        args: SmallVec<[ValueId; 4]>,
        span: Span,
    },
    ClosureCreate {
        dest: VarId,
        params: SmallVec<[VarId; 4]>,
        body: BlockId,
        captures: SmallVec<(VarId, CaptureKind); 4]>,
        span: Span,
    },
    IteratorNext {
        dest: VarId,
        iterator: ValueId,
        span: Span,
    },
    IteratorCollect {
        dest: VarId,
        iterator: ValueId,
        span: Span,
    },
    IteratorChain {
        dest: VarId,
        iterator1: ValueId,
        iterator2: ValueId,
        span: Span,
    },
    IteratorFilter {
        dest: VarId,
        iterator: ValueId,
        predicate: ValueId,
        span: Span,
    },
    IteratorFold {
        dest: VarId,
        iterator: ValueId,
        init: ValueId,
        accumulator: VarId,
        body: BlockId,
        span: Span,
    },
    IteratorMap {
        dest: VarId,
        iterator: ValueId,
        mapper: ValueId,
        span: Span,
    },
    ResultOk {
        dest: VarId,
        value: ValueId,
        ok_type: TypeId,
        err_type: TypeId,
        span: Span,
    },
    ResultErr {
        dest: VarId,
        error: ValueId,
        ok_type: TypeId,
        err_type: TypeId,
        span: Span,
    },
    TryCatch {
        dest: VarId,
        try_expr: ValueId,
        catch_block: BlockId,
        error_var: VarId,
        span: Span,
    },
    Throw {
        error: ValueId,
        span: Span,
    },
    FutureAwait {
        dest: VarId,
        future: ValueId,
        span: Span,
    },
    Yield {
        value: Option<ValueId>,
        span: Span,
    },
    StreamYield {
        value: Option<ValueId>,
        span: Span,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    Shl,
    Shr,
    And,
    Or,
    Xor,
    FAdd,
    FSub,
    FMul,
    FDiv,
    FRem,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UnaryOp {
    Neg,
    Not,
    FNeg,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CastOp {
    Trunc,
    ZExt,
    SExt,
    IntToPtr,
    PtrToInt,
    BitCast,
    FToUi,
    FToSi,
    UiToF,
    SiToF,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Terminator {
    Return(Option<ValueId>),
    Branch(BlockId),
    ConditionalBranch { condition: ValueId, true_block: BlockId, false_block: BlockId },
    Unreachable,
    Invoke { func: ValueId, args: SmallVec<[ValueId; 4]>, normal_block: BlockId, unwind_block: BlockId },
    Switch { value: ValueId, default_block: BlockId, cases: Vec<(ValueId, BlockId)> },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Global {
    pub name: String,
    pub ty: TypeId,
    pub initializer: Option<IRConstant>,
    pub is_pub: bool,
    pub is_const: bool,
    pub align: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum IRConstant {
    Int(i128),
    Float(f64),
    Bool(bool),
    Char(char),
    String(String),
    Null,
    Aggregate(AggregateConstant),
    GlobalReference(TypeId),
}

#[derive(Debug, Clone, PartialEq)]
pub enum AggregateConstant {
    Vector(Vec<IRConstant>),
    Array(Vec<IRConstant>),
    Struct(Vec<IRConstant>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct IRStruct {
    pub id: StructId,
    pub name: String,
    pub fields: Vec<IRStructField>,
    pub size: usize,
    pub align: usize,
    pub is_packed: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct IRStructField {
    pub name: String,
    pub ty: TypeId,
    pub offset: usize,
    pub size: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct IREnum {
    pub id: EnumId,
    pub name: String,
    pub variants: Vec<IREnumVariant>,
    pub size: usize,
    pub align: usize,
    pub tag_repr: TagRepresentation,
}

#[derive(Debug, Clone, PartialEq)]
pub struct IREnumVariant {
    pub name: String,
    pub discriminant: i128,
    pub fields: Vec<IRStructField>,
    pub size: usize,
    pub align: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TagRepresentation {
    U8,
    U16,
    U32,
    U64,
    Usize,
}

#[derive(Debug)]
pub struct IRBuilder<'a> {
    pub module: &'a mut IRModule,
    pub current_function: Option<&'a mut IRFunction>,
    pub current_block: Option<&'a mut BasicBlock>,
    pub next_value_id: usize,
    pub next_block_id: usize,
}

impl<'a> IRBuilder<'a> {
    pub fn new(module: &'a mut IRModule) -> IRBuilder<'a> {
        IRBuilder {
            module,
            current_function: None,
            current_block: None,
            next_value_id: 0,
            next_block_id: 0,
        }
    }

    pub fn create_value(&mut self) -> ValueId {
        let id = ValueId(self.next_value_id);
        self.next_value_id += 1;
        id
    }

    pub fn create_block(&mut self) -> BlockId {
        let id = BlockId(self.next_block_id);
        self.next_block_id += 1;
        id
    }

    pub fn set_current_block(&mut self, block: &'a mut BasicBlock) {
        self.current_block = Some(block);
    }

    pub fn append_inst(&mut self, inst: IRInst) {
        if let Some(block) = &mut self.current_block {
            block.instructions.push(inst);
        }
    }

    pub fn emit_alloca(&mut self, ty: TypeId, span: Span) -> VarId {
        let dest = self.create_value();
        self.append_inst(IRInst::Alloca { dest, ty, span });
        dest
    }

    pub fn emit_load(&mut self, dest: VarId, src: ValueId, ty: TypeId, span: Span) {
        self.append_inst(IRInst::Load { dest, src, ty, span });
    }

    pub fn emit_store(&mut self, dest: ValueId, src: ValueId, ty: TypeId, span: Span) {
        self.append_inst(IRInst::Store { dest, src, ty, span });
    }

    pub fn emit_binary(&mut self, dest: VarId, op: BinaryOp, left: ValueId, right: ValueId, ty: TypeId, span: Span) {
        self.append_inst(IRInst::Binary { dest, op, left, right, ty, span });
    }

    pub fn emit_call(&mut self, dest: Option<VarId>, func: ValueId, args: SmallVec<[ValueId; 4]>, ty: TypeId, span: Span) {
        self.append_inst(IRInst::Call { dest, func, args, ty, span });
    }

    pub fn emit_ret(&mut self, value: Option<ValueId>, span: Span) {
        self.append_inst(IRInst::Ret { value, span });
    }

    pub fn emit_ret_void(&mut self, span: Span) {
        self.append_inst(IRInst::RetVoid { span });
    }

    pub fn emit_br(&mut self, target: BlockId, span: Span) {
        self.append_inst(IRInst::Br { target, span });
    }

    pub fn emit_cond_br(&mut self, condition: ValueId, true_block: BlockId, false_block: BlockId, span: Span) {
        self.append_inst(IRInst::CondBr { condition, true_block, false_block, span });
    }

    pub fn emit_atomic_load(&mut self, dest: VarId, src: ValueId, order: MemoryOrder, ty: TypeId, span: Span) {
        self.append_inst(IRInst::AtomicLoad { dest, src, order, ty, span });
    }

    pub fn emit_atomic_store(&mut self, dest: ValueId, src: ValueId, order: MemoryOrder, ty: TypeId, span: Span) {
        self.append_inst(IRInst::AtomicStore { dest, src, order, ty, span });
    }

    pub fn emit_atomic_fetch_add(&mut self, dest: VarId, src: ValueId, value: ValueId, order: MemoryOrder, ty: TypeId, span: Span) {
        self.append_inst(IRInst::AtomicFetchAdd { dest, src, value, order, ty, span });
    }

    pub fn emit_atomic_fetch_sub(&mut self, dest: VarId, src: ValueId, value: ValueId, order: MemoryOrder, ty: TypeId, span: Span) {
        self.append_inst(IRInst::AtomicFetchSub { dest, src, value, order, ty, span });
    }

    pub fn emit_atomic_fetch_and(&mut self, dest: VarId, src: ValueId, value: ValueId, order: MemoryOrder, ty: TypeId, span: Span) {
        self.append_inst(IRInst::AtomicFetchAnd { dest, src, value, order, ty, span });
    }

    pub fn emit_atomic_fetch_or(&mut self, dest: VarId, src: ValueId, value: ValueId, order: MemoryOrder, ty: TypeId, span: Span) {
        self.append_inst(IRInst::AtomicFetchOr { dest, src, value, order, ty, span });
    }

    pub fn emit_atomic_fetch_xor(&mut self, dest: VarId, src: ValueId, value: ValueId, order: MemoryOrder, ty: TypeId, span: Span) {
        self.append_inst(IRInst::AtomicFetchXor { dest, src, value, order, ty, span });
    }

    pub fn emit_atomic_compare_exchange(&mut self, dest: VarId, src: ValueId, expected: ValueId, desired: ValueId, success_order: MemoryOrder, failure_order: MemoryOrder, ty: TypeId, span: Span) {
        self.append_inst(IRInst::AtomicCompareExchange { dest, src, expected, desired, success_order, failure_order, ty, span });
    }

    pub fn emit_atomic_exchange(&mut self, dest: VarId, src: ValueId, value: ValueId, order: MemoryOrder, ty: TypeId, span: Span) {
        self.append_inst(IRInst::AtomicExchange { dest, src, value, order, ty, span });
    }

    pub fn emit_atomic_fence(&mut self, order: MemoryOrder, span: Span) {
        self.append_inst(IRInst::AtomicFence { order, span });
    }

    pub fn emit_wait(&mut self, atomic: ValueId, timeout: Option<ValueId>, span: Span) {
        self.append_inst(IRInst::Wait { atomic, timeout, span });
    }

    pub fn emit_notify(&mut self, atomic: ValueId, span: Span) {
        self.append_inst(IRInst::Notify { atomic, span });
    }

    pub fn emit_notify_all(&mut self, atomic: ValueId, span: Span) {
        self.append_inst(IRInst::NotifyAll { atomic, span });
    }

    pub fn emit_memory_barrier(&mut self, span: Span) {
        self.append_inst(IRInst::MemoryBarrier { span });
    }

    pub fn emit_data_dependency(&mut self, src: ValueId, dest: ValueId, span: Span) {
        self.append_inst(IRInst::DataDependency { src, dest, span });
    }

    pub fn emit_macro_expand(&mut self, dest: VarId, macro_name: ValueId, args: SmallVec<[ValueId; 4]>, span: Span) {
        self.append_inst(IRInst::MacroExpand { dest, macro_name, args, span });
    }

    pub fn emit_closure_create(&mut self, dest: VarId, params: SmallVec<[VarId; 4]>, body: BlockId, captures: SmallVec<(VarId, CaptureKind); 4]>, span: Span) {
        self.append_inst(IRInst::ClosureCreate { dest, params, body, captures, span });
    }

    pub fn emit_iterator_next(&mut self, dest: VarId, iterator: ValueId, span: Span) {
        self.append_inst(IRInst::IteratorNext { dest, iterator, span });
    }

    pub fn emit_iterator_collect(&mut self, dest: VarId, iterator: ValueId, span: Span) {
        self.append_inst(IRInst::IteratorCollect { dest, iterator, span });
    }

    pub fn emit_iterator_chain(&mut self, dest: VarId, iterator1: ValueId, iterator2: ValueId, span: Span) {
        self.append_inst(IRInst::IteratorChain { dest, iterator1, iterator2, span });
    }

    pub fn emit_iterator_filter(&mut self, dest: VarId, iterator: ValueId, predicate: ValueId, span: Span) {
        self.append_inst(IRInst::IteratorFilter { dest, iterator, predicate, span });
    }

    pub fn emit_iterator_fold(&mut self, dest: VarId, iterator: ValueId, init: ValueId, accumulator: VarId, body: BlockId, span: Span) {
        self.append_inst(IRInst::IteratorFold { dest, iterator, init, accumulator, body, span });
    }

    pub fn emit_iterator_map(&mut self, dest: VarId, iterator: ValueId, mapper: ValueId, span: Span) {
        self.append_inst(IRInst::IteratorMap { dest, iterator, mapper, span });
    }

    pub fn emit_result_ok(&mut self, dest: VarId, value: ValueId, ok_type: TypeId, err_type: TypeId, span: Span) {
        self.append_inst(IRInst::ResultOk { dest, value, ok_type, err_type, span });
    }

    pub fn emit_result_err(&mut self, dest: VarId, error: ValueId, ok_type: TypeId, err_type: TypeId, span: Span) {
        self.append_inst(IRInst::ResultErr { dest, error, ok_type, err_type, span });
    }

    pub fn emit_try_catch(&mut self, dest: VarId, try_expr: ValueId, catch_block: BlockId, error_var: VarId, span: Span) {
        self.append_inst(IRInst::TryCatch { dest, try_expr, catch_block, error_var, span });
    }

    pub fn emit_throw(&mut self, error: ValueId, span: Span) {
        self.append_inst(IRInst::Throw { error, span });
    }

    pub fn emit_future_await(&mut self, dest: VarId, future: ValueId, span: Span) {
        self.append_inst(IRInst::FutureAwait { dest, future, span });
    }

    pub fn emit_yield(&mut self, value: Option<ValueId>, span: Span) {
        self.append_inst(IRInst::Yield { value, span });
    }

    pub fn emit_stream_yield(&mut self, value: Option<ValueId>, span: Span) {
        self.append_inst(IRInst::StreamYield { value, span });
    }
}

pub trait IRGenerator {
    fn generate_module(&mut self, program: &chim_ast::Program) -> IRModule;
    fn generate_function(&mut self, func: &chim_ast::Function) -> IRFunction;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_creation() {
        let mut module = IRModule {
            functions: Vec::new(),
            globals: Vec::new(),
            structs: Vec::new(),
            enums: Vec::new(),
        };
        assert!(module.functions.is_empty());
    }

    #[test]
    fn test_value_creation() {
        let mut builder = IRBuilder::new(&mut IRModule {
            functions: Vec::new(),
            globals: Vec::new(),
            structs: Vec::new(),
            enums: Vec::new(),
        });
        let value = builder.create_value();
        assert_eq!(value.0, 0);
    }

    #[test]
    fn test_block_creation() {
        let mut builder = IRBuilder::new(&mut IRModule {
            functions: Vec::new(),
            globals: Vec::new(),
            structs: Vec::new(),
            enums: Vec::new(),
        });
        let block = builder.create_block();
        assert_eq!(block.0, 0);
    }
}
