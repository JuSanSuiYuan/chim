use chim_span::{FileId, Span};
use smallvec::SmallVec;
use std::sync::Arc;

pub type Ident = Arc<str>;

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub items: Vec<Item>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Item {
    Function(Function),
    Struct(Struct),
    Enum(Enum),
    Trait(Trait),
    Impl(Impl),
    Use(Use),
    Mod(Mod),
    Extern(ExternBlock),
    Constant(Constant),
    Static(Static),
    Macro(Macro),
    ForAll(ForAll),
    Default(Default),
    Sync(Sync),
    Sized(Sized),
    IntoIterator(IntoIterator),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    pub name: Ident,
    pub params: Vec<Param>,
    pub return_type: Option<Type>,
    pub body: Vec<Stmt>,
    pub span: Span,
    pub is_pub: bool,
    pub is_async: bool,
    pub lifetimes: Vec<LifetimeParam>,
    pub where_clauses: Vec<WhereClause>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Param {
    pub name: Ident,
    pub ty: Type,
    pub span: Span,
    pub is_mut: bool,
    pub is_ref: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Struct {
    pub name: Ident,
    pub fields: Vec<Field>,
    pub span: Span,
    pub is_pub: bool,
    pub generics: Vec<GenericParam>,
    pub where_clauses: Vec<WhereClause>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Field {
    pub name: Ident,
    pub ty: Type,
    pub span: Span,
    pub is_pub: bool,
    pub attributes: Vec<Attribute>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Enum {
    pub name: Ident,
    pub variants: Vec<Variant>,
    pub span: Span,
    pub is_pub: bool,
    pub generics: Vec<GenericParam>,
    pub where_clauses: Vec<WhereClause>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Variant {
    pub name: Ident,
    pub fields: Vec<Field>,
    pub span: Span,
    pub attributes: Vec<Attribute>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Trait {
    pub name: Ident,
    pub items: Vec<TraitItem>,
    pub span: Span,
    pub is_pub: bool,
    pub generics: Vec<GenericParam>,
    pub super_traits: Vec<Type>,
    pub where_clauses: Vec<WhereClause>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TraitItem {
    Function(FunctionSig),
    Const(TraitConst),
    Type(TypeBinding),
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionSig {
    pub name: Ident,
    pub params: Vec<Param>,
    pub return_type: Option<Type>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TraitConst {
    pub name: Ident,
    pub ty: Type,
    pub default: Option<Expr>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Impl {
    pub trait_name: Option<Type>,
    pub type_name: Type,
    pub items: Vec<ImplItem>,
    pub span: Span,
    pub generics: Vec<GenericParam>,
    pub where_clauses: Vec<WhereClause>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ImplItem {
    Function(Function),
    Const(Constant),
    Type(TypeBinding),
}

#[derive(Debug, Clone, PartialEq)]
pub struct TypeBinding {
    pub name: Ident,
    pub ty: Type,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Use {
    pub path: Path,
    pub alias: Option<Ident>,
    pub span: Span,
    pub is_pub: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Mod {
    pub name: Ident,
    pub items: Vec<Item>,
    pub span: Span,
    pub is_pub: bool,
    pub file_path: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExternBlock {
    pub abi: String,
    pub items: Vec<ExternItem>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExternItem {
    pub name: Ident,
    pub ty: Type,
    pub span: Span,
    pub is_pub: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Constant {
    pub name: Ident,
    pub ty: Option<Type>,
    pub value: Expr,
    pub span: Span,
    pub is_pub: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Static {
    pub name: Ident,
    pub ty: Type,
    pub value: Option<Expr>,
    pub span: Span,
    pub is_pub: bool,
    pub is_mut: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Macro {
    pub name: Ident,
    pub params: Vec<MacroParam>,
    pub body: MacroBody,
    pub span: Span,
    pub is_pub: bool,
    pub is_procedural: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MacroParam {
    pub name: Ident,
    pub ty: Option<Type>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MacroBody {
    Rules(Vec<MacroRule>),
    Procedural(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct MacroRule {
    pub pattern: MacroPattern,
    pub expansion: MacroExpansion,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MacroPattern {
    Token(MacroToken),
    Sequence(Vec<MacroPattern>),
    Repeat(Box<MacroPattern>, MacroRepeatKind),
    Choice(Vec<MacroPattern>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MacroRepeatKind {
    ZeroOrMore,
    OneOrMore,
    ZeroOrOne,
    Sep(Option<char>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum MacroToken {
    Ident(Ident),
    Literal(String),
    Punctuation(char),
    Group(Vec<MacroToken>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum MacroExpansion {
    Expr(Expr),
    Tokens(Vec<MacroToken>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForAll {
    pub name: Ident,
    pub params: Vec<Param>,
    pub return_type: Option<Type>,
    pub body: Vec<Stmt>,
    pub span: Span,
    pub is_pub: bool,
    pub generics: Vec<GenericParam>,
    pub where_clauses: Vec<WhereClause>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Default {
    pub name: Ident,
    pub ty: Type,
    pub value: Expr,
    pub span: Span,
    pub is_pub: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Sync {
    pub name: Ident,
    pub ty: Type,
    pub span: Span,
    pub is_pub: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Sized {
    pub name: Ident,
    pub ty: Type,
    pub span: Span,
    pub is_pub: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct IntoIterator {
    pub name: Ident,
    pub ty: Type,
    pub span: Span,
    pub is_pub: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Stmt {
    pub kind: StmtKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum StmtKind {
    Expr(Expr),
    Let(LetStmt),
    Var(VarStmt),
    Return(Option<Expr>),
    Break(Option<Expr>),
    Continue,
    Loop(LoopStmt),
    While(WhileStmt),
    For(ForStmt),
    Empty,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LetStmt {
    pub pattern: Pattern,
    pub ty: Option<Type>,
    pub initializer: Option<Expr>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct VarStmt {
    pub pattern: Pattern,
    pub ty: Option<Type>,
    pub initializer: Option<Expr>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LoopStmt {
    pub label: Option<Ident>,
    pub body: Vec<Stmt>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WhileStmt {
    pub label: Option<Ident>,
    pub condition: Expr,
    pub body: Vec<Stmt>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForStmt {
    pub label: Option<Ident>,
    pub pattern: Pattern,
    pub iterable: Expr,
    pub body: Vec<Stmt>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Expr {
    pub kind: Box<ExprKind>,
    pub span: Span,
    pub ty: Option<Type>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExprKind {
    Literal(Literal),
    Identifier(Ident),
    Path(Path),
    Binary(BinaryExpr),
    Unary(UnaryExpr),
    Call(CallExpr),
    MethodCall(MethodCallExpr),
    FieldAccess(FieldAccessExpr),
    Tuple(TupleExpr),
    Array(ArrayExpr),
    Index(IndexExpr),
    Slice(SliceExpr),
    Cast(CastExpr),
    If(IfExpr),
    Match(MatchExpr),
    Block(BlockExpr),
    Closure(ClosureExpr),
    AsyncBlock(AsyncBlockExpr),
    Continue,
    Break(Option<Label>, Option<Box<Expr>>),
    Return(Option<Box<Expr>>),
    Loop(LoopExpr),
    While(WhileExpr),
    For(ForExpr),
    Assign(AssignExpr),
    AssignOp(AssignOpExpr),
    Range(RangeExpr),
    Struct(StructExpr),
    Enum(EnumExpr),
    Field(StructField),
    AtomicLoad(AtomicLoadExpr),
    AtomicStore(AtomicStoreExpr),
    AtomicFetchAdd(AtomicFetchExpr),
    AtomicFetchSub(AtomicFetchExpr),
    AtomicFetchAnd(AtomicFetchExpr),
    AtomicFetchOr(AtomicFetchExpr),
    AtomicFetchXor(AtomicFetchExpr),
    AtomicCompareExchange(AtomicCompareExchangeExpr),
    AtomicExchange(AtomicExchangeExpr),
    AtomicFence(AtomicFenceExpr),
    Wait(WaitExpr),
    Notify(NotifyExpr),
    NotifyAll(NotifyAllExpr),
    EffectBlock(EffectBlockExpr),
    AbilityBlock(AbilityBlockExpr),
    LinkedList(LinkedListExpr),
    ListNode(ListNodeExpr),
    PushFront(PushFrontExpr),
    PushBack(PushBackExpr),
    PopFront(PopFrontExpr),
    PopBack(PopBackExpr),
    Front(FrontExpr),
    Back(BackExpr),
    Insert(InsertExpr),
    Erase(EraseExpr),
    Clear(ClearExpr),
    Splice(SpliceExpr),
    Merge(MergeExpr),
    Reverse(ReverseExpr),
    Sort(SortExpr),
    Unique(UniqueExpr),
    Remove(RemoveExpr),
    Error,
    Iterator(IteratorExpr),
    Next(NextExpr),
    Item(ItemExpr),
    Collect(CollectExpr),
    Chain(ChainExpr),
    Filter(FilterExpr),
    Fold(FoldExpr),
    Map(MapExpr),
    Result(ResultExpr),
    Ok(OkExpr),
    Err(ErrExpr),
    Try(TryExpr),
    Catch(CatchExpr),
    ErrorExpr(ErrorExpr),
    Context(ContextExpr),
    Throw(ThrowExpr),
    Future(FutureExpr),
    Yield(YieldExpr),
    Stream(StreamExpr),
    Unsafe(UnsafeExpr),
    Alloc(AllocExpr),
    AllocAligned(AllocAlignedExpr),
    Free(FreeExpr),
    Ptr(PtrExpr),
    PtrAdd(PtrAddExpr),
    PtrSub(PtrSubExpr),
    PtrLoad(PtrLoadExpr),
    PtrStore(PtrStoreExpr),
    PtrCast(PtrCastExpr),
    PtrOffsetOf(PtrOffsetOfExpr),
    PtrSizeOf(PtrSizeOfExpr),
    AlignOf(AlignOfExpr),
    Proof(ProofExpr),
    Theorem(TheoremExpr),
    Lemma(LemmaExpr),
    Induction(InductionExpr),
    Case(CaseExpr),
    Refl(ReflExpr),
    Cong(CongExpr),
    Sym(SymExpr),
    Trans(TransExpr),
    Rec(RecExpr),
    Fix(FixExpr),
    Class(ClassExpr),
    Instance(InstanceExpr),
    Where(WhereExpr),
    EqProp(EqPropExpr),
    ReflProp(ReflPropExpr),
    JMeq(JMeqExpr),
    Rewrite(RewriteExpr),
    With(WithExpr),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Literal {
    pub kind: LiteralKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LiteralKind {
    Int(i128),
    Float(f64),
    Bool(bool),
    Char(char),
    String(Ident),
    Unit,
    Byte,
    ByteString(Ident),
    Atomic(AtomicLiteral),
}

#[derive(Debug, Clone, PartialEq)]
pub enum AtomicLiteral {
    AtomicI32(i32),
    AtomicI64(i64),
    AtomicU32(u32),
    AtomicU64(u64),
    AtomicIsize(isize),
    AtomicUsize(usize),
}

#[derive(Debug, Clone, PartialEq)]
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
    Wait,
    Notify,
    NotifyAll,
    DataDependency,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Effect {
    IO,
    Exception,
    State,
    Async,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Ability {
    pub name: Ident,
    pub effects: Vec<Effect>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BinaryExpr {
    pub left: Box<Expr>,
    pub op: BinOp,
    pub right: Box<Expr>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    And,
    Or,
    BitAnd,
    BitOr,
    BitXor,
    Shl,
    Shr,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UnOp {
    Neg,
    Not,
    Deref,
    Ref,
    RefMut,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnaryExpr {
    pub op: UnOp,
    pub expr: Box<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CallExpr {
    pub func: Box<Expr>,
    pub args: SmallVec<[Box<Expr>; 4]>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MethodCallExpr {
    pub expr: Box<Expr>,
    pub method: Ident,
    pub args: SmallVec<[Box<Expr>; 4]>,
    pub generics: Vec<Type>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FieldAccessExpr {
    pub expr: Box<Expr>,
    pub field: Ident,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TupleExpr {
    pub elements: SmallVec<[Box<Expr>; 4]>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ArrayExpr {
    pub elements: SmallVec<[Box<Expr>; 4]>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct IndexExpr {
    pub expr: Box<Expr>,
    pub index: Box<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SliceExpr {
    pub expr: Box<Expr>,
    pub start: Option<Box<Expr>>,
    pub end: Option<Box<Expr>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CastExpr {
    pub expr: Box<Expr>,
    pub ty: Type,
}

#[derive(Debug, Clone, PartialEq)]
pub struct IfExpr {
    pub condition: Box<Expr>,
    pub then_branch: BlockExpr,
    pub else_branch: Option<Box<Expr>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MatchExpr {
    pub expr: Box<Expr>,
    pub arms: Vec<MatchArm>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub guard: Option<Expr>,
    pub body: Expr,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BlockExpr {
    pub label: Option<Ident>,
    pub stmts: Vec<Stmt>,
    pub ty: Option<Type>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClosureExpr {
    pub params: Vec<Param>,
    pub body: Box<Expr>,
    pub is_async: bool,
    pub is_move: bool,
    pub captures: Vec<Capture>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Capture {
    CaptureRef(Ident),
    CaptureValue(Ident),
}

#[derive(Debug, Clone, PartialEq)]
pub struct AsyncBlockExpr {
    pub body: BlockExpr,
    pub is_move: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AtomicLoadExpr {
    pub atomic: Box<Expr>,
    pub order: MemoryOrder,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AtomicStoreExpr {
    pub atomic: Box<Expr>,
    pub value: Box<Expr>,
    pub order: MemoryOrder,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AtomicFetchExpr {
    pub atomic: Box<Expr>,
    pub value: Box<Expr>,
    pub order: MemoryOrder,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AtomicCompareExchangeExpr {
    pub atomic: Box<Expr>,
    pub expected: Box<Expr>,
    pub desired: Box<Expr>,
    pub success_order: MemoryOrder,
    pub failure_order: MemoryOrder,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AtomicExchangeExpr {
    pub atomic: Box<Expr>,
    pub value: Box<Expr>,
    pub order: MemoryOrder,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AtomicFenceExpr {
    pub order: MemoryOrder,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EffectBlockExpr {
    pub effects: Vec<Effect>,
    pub body: Box<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AbilityBlockExpr {
    pub ability: Ability,
    pub body: Box<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LinkedListExpr {
    pub ty: Type,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ListNodeExpr {
    pub ty: Type,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PushFrontExpr {
    pub list: Box<Expr>,
    pub value: Box<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PushBackExpr {
    pub list: Box<Expr>,
    pub value: Box<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PopFrontExpr {
    pub list: Box<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PopBackExpr {
    pub list: Box<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FrontExpr {
    pub list: Box<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BackExpr {
    pub list: Box<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct InsertExpr {
    pub list: Box<Expr>,
    pub position: Box<Expr>,
    pub value: Box<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EraseExpr {
    pub list: Box<Expr>,
    pub value: Box<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClearExpr {
    pub list: Box<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SpliceExpr {
    pub list1: Box<Expr>,
    pub list2: Box<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MergeExpr {
    pub list1: Box<Expr>,
    pub list2: Box<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReverseExpr {
    pub list: Box<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SortExpr {
    pub list: Box<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UniqueExpr {
    pub list: Box<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RemoveExpr {
    pub list: Box<Expr>,
    pub value: Box<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WaitExpr {
    pub atomic: Box<Expr>,
    pub timeout: Option<Box<Expr>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NotifyExpr {
    pub atomic: Box<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NotifyAllExpr {
    pub atomic: Box<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct IteratorExpr {
    pub iterable: Box<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NextExpr {
    pub iterator: Box<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ItemExpr {
    pub iterator: Box<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CollectExpr {
    pub iterator: Box<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChainExpr {
    pub iterator1: Box<Expr>,
    pub iterator2: Box<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FilterExpr {
    pub iterator: Box<Expr>,
    pub predicate: Box<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FoldExpr {
    pub iterator: Box<Expr>,
    pub init: Box<Expr>,
    pub accumulator: Ident,
    pub body: Box<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MapExpr {
    pub iterator: Box<Expr>,
    pub mapper: Box<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResultExpr {
    pub ok_type: Type,
    pub err_type: Type,
}

#[derive(Debug, Clone, PartialEq)]
pub struct OkExpr {
    pub value: Box<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ErrExpr {
    pub error: Box<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TryExpr {
    pub expr: Box<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CatchExpr {
    pub try_expr: Box<Expr>,
    pub error_var: Ident,
    pub catch_expr: Box<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ErrorExpr {
    pub message: Box<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ContextExpr {
    pub context: Box<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ThrowExpr {
    pub error: Box<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FutureExpr {
    pub body: Box<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct YieldExpr {
    pub value: Option<Box<Expr>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StreamExpr {
    pub body: Box<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnsafeExpr {
    pub body: Box<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllocExpr {
    pub ty: Type,
    pub size: Option<Box<Expr>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllocAlignedExpr {
    pub ty: Type,
    pub size: Box<Expr>,
    pub alignment: Box<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreeExpr {
    pub ptr: Box<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PtrExpr {
    pub ty: Type,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PtrAddExpr {
    pub ptr: Box<Expr>,
    pub offset: Box<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PtrSubExpr {
    pub ptr1: Box<Expr>,
    pub ptr2: Box<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PtrLoadExpr {
    pub ptr: Box<Expr>,
    pub ty: Type,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PtrStoreExpr {
    pub ptr: Box<Expr>,
    pub value: Box<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PtrCastExpr {
    pub ptr: Box<Expr>,
    pub target_ty: Type,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PtrOffsetOfExpr {
    pub ptr: Box<Expr>,
    pub field: Ident,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PtrSizeOfExpr {
    pub ty: Type,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlignOfExpr {
    pub ty: Type,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProofExpr {
    pub proposition: Box<Expr>,
    pub proof: Box<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TheoremExpr {
    pub name: Ident,
    pub params: Vec<Param>,
    pub proposition: Box<Expr>,
    pub proof: Box<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LemmaExpr {
    pub name: Ident,
    pub params: Vec<Param>,
    pub proposition: Box<Expr>,
    pub proof: Box<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct InductionExpr {
    pub variable: Ident,
    pub base_case: Box<Expr>,
    pub inductive_step: Box<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CaseExpr {
    pub value: Box<Expr>,
    pub cases: Vec<MatchCase>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReflExpr {
    pub ty: Type,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CongExpr {
    pub ty: Type,
    pub expr1: Box<Expr>,
    pub expr2: Box<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SymExpr {
    pub ty: Type,
    pub expr: Box<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TransExpr {
    pub ty: Type,
    pub expr1: Box<Expr>,
    pub expr2: Box<Expr>,
    pub expr3: Box<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RecExpr {
    pub ty: Type,
    pub body: Box<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FixExpr {
    pub ty: Type,
    pub body: Box<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClassExpr {
    pub name: Ident,
    pub params: Vec<Param>,
    pub methods: Vec<Function>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct InstanceExpr {
    pub class_name: Ident,
    pub ty: Type,
    pub methods: Vec<Function>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WhereExpr {
    pub expr: Box<Expr>,
    pub constraints: Vec<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EqPropExpr {
    pub ty: Type,
    pub left: Box<Expr>,
    pub right: Box<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReflPropExpr {
    pub ty: Type,
    pub expr: Box<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct JMeqExpr {
    pub ty: Type,
    pub expr1: Box<Expr>,
    pub expr2: Box<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RewriteExpr {
    pub ty: Type,
    pub expr: Box<Expr>,
    pub rule: Box<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WithExpr {
    pub expr: Box<Expr>,
    pub bindings: Vec<(Ident, Expr)>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LoopExpr {
    pub label: Option<Ident>,
    pub body: BlockExpr,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WhileExpr {
    pub label: Option<Ident>,
    pub condition: Box<Expr>,
    pub body: BlockExpr,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForExpr {
    pub label: Option<Ident>,
    pub pattern: Pattern,
    pub iterable: Box<Expr>,
    pub body: BlockExpr,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AssignExpr {
    pub left: Box<Expr>,
    pub right: Box<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AssignOpExpr {
    pub left: Box<Expr>,
    pub op: BinOp,
    pub right: Box<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RangeExpr {
    pub start: Option<Box<Expr>>,
    pub end: Option<Box<Expr>>,
    pub inclusive: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StructExpr {
    pub path: Path,
    pub fields: Vec<StructField>,
    pub base: Option<Box<Expr>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StructField {
    pub name: Ident,
    pub expr: Expr,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EnumExpr {
    pub path: Path,
    pub variant: Ident,
    pub fields: Vec<StructField>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Pattern {
    pub kind: PatternKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PatternKind {
    Wildcard,
    Identifier(Ident),
    Literal(Literal),
    Tuple(Vec<Pattern>),
    Struct(Path, Vec<PatternField>),
    Enum(Path, Ident, Vec<PatternField>),
    Range(Option<Box<Pattern>>, Option<Box<Pattern>>),
    Slice(Vec<Pattern>),
    Or(Vec<Pattern>),
    Error,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PatternField {
    pub name: Ident,
    pub pattern: Option<Pattern>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Type {
    pub kind: Box<TypeKind>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TypeKind {
    Path(Path),
    Tuple(Vec<Type>),
    Array(Box<Type>, usize),
    Slice(Box<Type>),
    Pointer(Box<Type>, Mutability),
    Reference(Option<Lifetime>, Box<Type>, Mutability),
    Function(FunctionType),
    Never,
    Infer,
    Error,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionType {
    pub params: Vec<Type>,
    pub return_type: Box<Type>,
    pub is_async: bool,
}

impl Eq for FunctionType {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Mutability {
    Mutable,
    Immutable,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Path {
    pub segments: Vec<PathSegment>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PathSegment {
    pub ident: Ident,
    pub args: Vec<GenericArg>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GenericArg {
    pub kind: GenericArgKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum GenericArgKind {
    Type(Type),
    Lifetime(Lifetime),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Lifetime {
    pub name: Ident,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LifetimeParam {
    pub name: Ident,
    pub bounds: Vec<Lifetime>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WhereClause {
    pub predicates: Vec<WherePredicate>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WherePredicate {
    pub bounded_type: Type,
    pub bounds: Vec<WhereBound>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WhereBound {
    pub trait_ref: Type,
    pub lifetime_bounds: Vec<Lifetime>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Label {
    pub name: Ident,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Attribute {
    pub name: Ident,
    pub args: Vec<AttributeArg>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AttributeArg {
    pub expr: Expr,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GenericParam {
    pub name: Ident,
    pub bounds: Vec<Type>,
    pub span: Span,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_program_creation() {
        let program = Program {
            items: Vec::new(),
            span: Span::new(FileId(0), 0, 0, 0, 0),
        };
        assert!(program.items.is_empty());
    }

    #[test]
    fn test_function_creation() {
        let func = Function {
            name: Arc::from("main"),
            params: Vec::new(),
            return_type: None,
            body: Vec::new(),
            span: Span::new(FileId(0), 0, 0, 0, 0),
            is_pub: false,
            is_async: false,
            lifetimes: Vec::new(),
            where_clauses: Vec::new(),
        };
        assert_eq!(func.name.as_ref(), "main");
    }

    #[test]
    fn test_type_creation() {
        let ty = Type {
            kind: TypeKind::Infer,
            span: Span::new(FileId(0), 0, 0, 0, 0),
        };
        matches!(ty.kind, TypeKind::Infer);
    }

    #[test]
    fn test_expr_creation() {
        let expr = Expr {
            kind: ExprKind::Literal(Literal {
                kind: LiteralKind::Int(42),
                span: Span::new(FileId(0), 0, 2, 0, 0),
            }),
            span: Span::new(FileId(0), 0, 2, 0, 0),
            ty: None,
        };
        matches!(expr.kind, ExprKind::Literal(Literal { kind: LiteralKind::Int(42), .. }));
    }
}
