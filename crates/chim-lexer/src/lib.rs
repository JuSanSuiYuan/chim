use chim_span::{FileId, Span};
use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone)]
pub struct LexerError {
    pub message: String,
    pub span: Span,
}

impl LexerError {
    pub fn new(message: String, span: Span) -> Self {
        LexerError { message, span }
    }
}

impl fmt::Display for LexerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} at {}:{}:{}", self.message, self.span.file_id.0, self.span.start_line, self.span.start_col)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Identifier(Spur);

impl Identifier {
    pub fn as_spur(&self) -> Spur {
        self.0
    }

    pub fn resolve<'a>(&self, interner: &'a Rodeo) -> &'a str {
        interner.resolve(&self.0)
    }
}

impl fmt::Display for Identifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Token {
    Let,
    Var,
    Const,
    Func,
    Return,
    If,
    Else,
    Loop,
    Break,
    Continue,
    Struct,
    Enum,
    Trait,
    Impl,
    For,
    Match,
    While,
    Pub,
    Priv,
    Use,
    Mod,
    Extern,
    Async,
    Await,
    Move,
    Clone,
    Ref,
    Mut,
    SelfKeyword,
    True,
    False,
    Null,
    Unit,
    Type,
    As,
    Where,
    SelfRef,
    Static,
    LetAlt,
    Is,
    Not,
    Or,
    And,
    Pattern,
    Range,
    Guard,
    Generic,
    ForAll,
    Default,
    Sync,
    Sized,
    IntoIterator,
    Macro,
    MacroRules,
    Procedural,
    Functional,
    Attribute,
    Derive,
    Closure,
    Capture,
    CaptureRef,
    CaptureValue,
    Iterator,
    Next,
    Item,
    Collect,
    Chain,
    Filter,
    Fold,
    Map,
    Result,
    Ok,
    Err,
    Try,
    Catch,
    Error,
    Context,
    Throw,
    Future,
    Yield,
    Stream,
    Ecs,
    Entity,
    Component,
    System,
    Actor,
    Message,
    Send,
    Receive,
    Concurrency,
    Atomic,
    AtomicLoad,
    AtomicStore,
    AtomicFetchAdd,
    AtomicFetchSub,
    AtomicFetchAnd,
    AtomicFetchOr,
    AtomicFetchXor,
    AtomicCompareExchange,
    AtomicExchange,
    AtomicFence,
    Relaxed,
    Acquire,
    Release,
    AcqRel,
    SeqCst,
    Consume,
    HappensBefore,
    Volatile,
    MemoryBarrier,
    Wait,
    Notify,
    NotifyAll,
    DataDependency,
    Effect,
    Ability,
    IO,
    Exception,
    State,
    AsyncEffect,
    LinkedList,
    ListNode,
    PushFront,
    PushBack,
    PopFront,
    PopBack,
    Front,
    Back,
    Insert,
    Erase,
    Clear,
    Splice,
    Merge,
    Reverse,
    Sort,
    Unique,
    Remove,
    End,
    Underscore,
    Unsafe,
    Alloc,
    AllocAligned,
    Free,
    Ptr,
    PtrAdd,
    PtrSub,
    PtrLoad,
    PtrStore,
    PtrCast,
    PtrOffsetOf,
    PtrSizeOf,
    AlignOf,
    Proof,
    Theorem,
    Lemma,
    Induction,
    Case,
    Refl,
    Cong,
    Sym,
    Trans,
    Rec,
    Fix,
    Class,
    Instance,
    Where,
    EqProp,
    ReflProp,
    JMeq,
    Rewrite,
    With,
    Underscore,
    Identifier,
    Int,
    Float,
    String,
    RawString,
    ByteString,
    Char,
    Byte,
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Eq,
    EqEq,
    Neq,
    Lt,
    Lte,
    Gt,
    Gte,
    AndAnd,
    OrOr,
    Bang,
    Ampersand,
    Pipe,
    Caret,
    LShift,
    RShift,
    PlusEq,
    MinusEq,
    StarEq,
    SlashEq,
    PercentEq,
    AndEq,
    PipeEq,
    CaretEq,
    LShiftEq,
    RShiftEq,
    Arrow,
    ThinArrow,
    PathSep,
    Colon,
    DoubleColon,
    Semicolon,
    Comma,
    Dot,
    DotDot,
    DotDotDot,
    Hash,
    At,
    Dollar,
    Question,
    DoubleQuestion,
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    LAngle,
    RAngle,
    Comment,
    BlockComment,
    DocComment,
    Whitespace,
    Unknown,
    Eof,
}

impl Token {
    pub fn is_keyword(&self) -> bool {
        matches!(self,
            Token::Let | Token::Var | Token::Const | Token::Func | Token::Return | Token::If | Token::Else |
            Token::Loop | Token::Break | Token::Continue | Token::Struct | Token::Enum |
            Token::Trait | Token::Impl | Token::For | Token::Match | Token::While |
            Token::Pub | Token::Priv | Token::Use | Token::Mod | Token::Extern |
            Token::Async | Token::Await | Token::Move | Token::Clone | Token::Ref |
            Token::Mut | Token::SelfKeyword | Token::True | Token::False | Token::Null |
            Token::Unit | Token::Type | Token::As | Token::Where | Token::SelfRef |
            Token::Static | Token::LetAlt | Token::Is | Token::Not | Token::Or | Token::And |
            Token::Pattern | Token::Range | Token::Guard | Token::Generic | Token::ForAll |
            Token::Default | Token::Sync | Token::Sized | Token::IntoIterator |
            Token::Macro | Token::MacroRules | Token::Procedural | Token::Functional |
            Token::Attribute | Token::Derive | Token::Closure | Token::Capture |
            Token::CaptureRef | Token::CaptureValue |
            Token::Iterator | Token::Next | Token::Item | Token::Collect |
            Token::Chain | Token::Filter | Token::Fold | Token::Map |
            Token::Result | Token::Ok | Token::Err | Token::Try | Token::Catch |
            Token::Error | Token::Context | Token::Throw |
            Token::Future | Token::Yield | Token::Stream |
            Token::Ecs | Token::Entity | Token::Component | Token::System | Token::Actor |
            Token::Message | Token::Send | Token::Receive | Token::Concurrency | Token::Atomic |
            Token::Relaxed | Token::Acquire | Token::Release | Token::AcqRel | Token::SeqCst |
            Token::Consume | Token::HappensBefore | Token::Volatile | Token::MemoryBarrier |
            Token::Wait | Token::Notify | Token::NotifyAll | Token::DataDependency |
            Token::Effect | Token::Ability | Token::IO | Token::Exception | Token::State | Token::AsyncEffect |
            Token::LinkedList | Token::ListNode | Token::PushFront | Token::PushBack | Token::PopFront | Token::PopBack |
            Token::Front | Token::Back | Token::Insert | Token::Erase | Token::Clear | Token::Splice |
            Token::Merge | Token::Reverse | Token::Sort | Token::Unique | Token::Remove | Token::End |
            Token::Unsafe | Token::Alloc | Token::AllocAligned | Token::Free | Token::Ptr |
            Token::PtrAdd | Token::PtrSub | Token::PtrLoad | Token::PtrStore | Token::PtrCast |
            Token::PtrOffsetOf | Token::PtrSizeOf | Token::AlignOf |
            Token::Proof | Token::Theorem | Token::Lemma | Token::Induction | Token::Case |
            Token::Refl | Token::Cong | Token::Sym | Token::Trans | Token::Rec | Token::Fix |
            Token::Class | Token::Instance | Token::Where | Token::EqProp | Token::ReflProp | Token::JMeq |
            Token::Rewrite | Token::With
        )
    }

    pub fn is_literal(&self) -> bool {
        matches!(self,
            Token::Int | Token::Float | Token::String | Token::RawString |
            Token::ByteString | Token::Char | Token::Byte | Token::True | Token::False
        )
    }

    pub fn is_operator(&self) -> bool {
        matches!(self,
            Token::Plus | Token::Minus | Token::Star | Token::Slash | Token::Percent |
            Token::Eq | Token::EqEq | Token::Neq | Token::Lt | Token::Lte | Token::Gt | Token::Gte |
            Token::AndAnd | Token::OrOr | Token::Bang | Token::Ampersand | Token::Pipe |
            Token::Caret | Token::LShift | Token::RShift | Token::PlusEq | Token::MinusEq |
            Token::StarEq | Token::SlashEq | Token::PercentEq | Token::AndEq | Token::PipeEq |
            Token::CaretEq | Token::LShiftEq | Token::RShiftEq | Token::Arrow | Token::ThinArrow |
            Token::PathSep | Token::DoubleColon | Token::Dot | Token::DotDot | Token::DotDotDot
        )
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SpannedToken {
    pub token: Token,
    pub span: Span,
}

impl SpannedToken {
    pub fn new(token: Token, span: Span) -> Self {
        SpannedToken { token, span }
    }
}

#[derive(Debug)]
pub struct Lexer<'a> {
    source: &'a str,
    file_id: FileId,
    pos: usize,
    line: usize,
    line_start: usize,
    keyword_map: HashMap<&'static str, Token>,
    errors: Vec<LexerError>,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str, _file_id: FileId) -> Self {
        let mut keyword_map = HashMap::new();
        
        keyword_map.insert("let", Token::Let);
        keyword_map.insert("var", Token::Var);
        keyword_map.insert("const", Token::Const);
        keyword_map.insert("fn", Token::Func);
        keyword_map.insert("return", Token::Return);
        keyword_map.insert("if", Token::If);
        keyword_map.insert("else", Token::Else);
        keyword_map.insert("loop", Token::Loop);
        keyword_map.insert("break", Token::Break);
        keyword_map.insert("continue", Token::Continue);
        keyword_map.insert("struct", Token::Struct);
        keyword_map.insert("enum", Token::Enum);
        keyword_map.insert("trait", Token::Trait);
        keyword_map.insert("impl", Token::Impl);
        keyword_map.insert("for", Token::For);
        keyword_map.insert("match", Token::Match);
        keyword_map.insert("while", Token::While);
        keyword_map.insert("pub", Token::Pub);
        keyword_map.insert("priv", Token::Priv);
        keyword_map.insert("use", Token::Use);
        keyword_map.insert("mod", Token::Mod);
        keyword_map.insert("extern", Token::Extern);
        keyword_map.insert("async", Token::Async);
        keyword_map.insert("await", Token::Await);
        keyword_map.insert("move", Token::Move);
        keyword_map.insert("clone", Token::Clone);
        keyword_map.insert("ref", Token::Ref);
        keyword_map.insert("mut", Token::Mut);
        keyword_map.insert("self", Token::SelfKeyword);
        keyword_map.insert("true", Token::True);
        keyword_map.insert("false", Token::False);
        keyword_map.insert("null", Token::Null);
        keyword_map.insert("unit", Token::Unit);
        keyword_map.insert("type", Token::Type);
        keyword_map.insert("as", Token::As);
        keyword_map.insert("where", Token::Where);
        keyword_map.insert("is", Token::Is);
        keyword_map.insert("not", Token::Not);
        keyword_map.insert("or", Token::Or);
        keyword_map.insert("and", Token::And);
        keyword_map.insert("pattern", Token::Pattern);
        keyword_map.insert("range", Token::Range);
        keyword_map.insert("guard", Token::Guard);
        keyword_map.insert("generic", Token::Generic);
        keyword_map.insert("forall", Token::ForAll);
        keyword_map.insert("default", Token::Default);
        keyword_map.insert("sync", Token::Sync);
        keyword_map.insert("sized", Token::Sized);
        keyword_map.insert("intoiterator", Token::IntoIterator);
        keyword_map.insert("macro", Token::Macro);
        keyword_map.insert("macrorules", Token::MacroRules);
        keyword_map.insert("procedural", Token::Procedural);
        keyword_map.insert("functional", Token::Functional);
        keyword_map.insert("attribute", Token::Attribute);
        keyword_map.insert("derive", Token::Derive);
        keyword_map.insert("closure", Token::Closure);
        keyword_map.insert("capture", Token::Capture);
        keyword_map.insert("captureref", Token::CaptureRef);
        keyword_map.insert("capturevalue", Token::CaptureValue);
        keyword_map.insert("iterator", Token::Iterator);
        keyword_map.insert("next", Token::Next);
        keyword_map.insert("item", Token::Item);
        keyword_map.insert("collect", Token::Collect);
        keyword_map.insert("chain", Token::Chain);
        keyword_map.insert("filter", Token::Filter);
        keyword_map.insert("fold", Token::Fold);
        keyword_map.insert("map", Token::Map);
        keyword_map.insert("result", Token::Result);
        keyword_map.insert("ok", Token::Ok);
        keyword_map.insert("err", Token::Err);
        keyword_map.insert("try", Token::Try);
        keyword_map.insert("catch", Token::Catch);
        keyword_map.insert("error", Token::Error);
        keyword_map.insert("context", Token::Context);
        keyword_map.insert("throw", Token::Throw);
        keyword_map.insert("future", Token::Future);
        keyword_map.insert("yield", Token::Yield);
        keyword_map.insert("stream", Token::Stream);
        keyword_map.insert("_", Token::Underscore);
        keyword_map.insert("unsafe", Token::Unsafe);
        keyword_map.insert("alloc", Token::Alloc);
        keyword_map.insert("alloc_aligned", Token::AllocAligned);
        keyword_map.insert("free", Token::Free);
        keyword_map.insert("ptr", Token::Ptr);
        keyword_map.insert("ptr_add", Token::PtrAdd);
        keyword_map.insert("ptr_sub", Token::PtrSub);
        keyword_map.insert("ptr_load", Token::PtrLoad);
        keyword_map.insert("ptr_store", Token::PtrStore);
        keyword_map.insert("ptr_cast", Token::PtrCast);
        keyword_map.insert("ptr_offsetof", Token::PtrOffsetOf);
        keyword_map.insert("ptr_sizeof", Token::PtrSizeOf);
        keyword_map.insert("alignof", Token::AlignOf);
        keyword_map.insert("proof", Token::Proof);
        keyword_map.insert("theorem", Token::Theorem);
        keyword_map.insert("lemma", Token::Lemma);
        keyword_map.insert("induction", Token::Induction);
        keyword_map.insert("case", Token::Case);
        keyword_map.insert("refl", Token::Refl);
        keyword_map.insert("cong", Token::Cong);
        keyword_map.insert("sym", Token::Sym);
        keyword_map.insert("trans", Token::Trans);
        keyword_map.insert("rec", Token::Rec);
        keyword_map.insert("fix", Token::Fix);
        keyword_map.insert("class", Token::Class);
        keyword_map.insert("instance", Token::Instance);
        keyword_map.insert("where", Token::Where);
        keyword_map.insert("eqprop", Token::EqProp);
        keyword_map.insert("reflprop", Token::ReflProp);
        keyword_map.insert("jmeq", Token::JMeq);
        keyword_map.insert("rewrite", Token::Rewrite);
        keyword_map.insert("with", Token::With);
        keyword_map.insert("令", Token::Let);
        keyword_map.insert("设", Token::Var);
        keyword_map.insert("常量", Token::Const);
        keyword_map.insert("函数", Token::Func);
        keyword_map.insert("返回", Token::Return);
        keyword_map.insert("如果", Token::If);
        keyword_map.insert("否则", Token::Else);
        keyword_map.insert("循环", Token::Loop);
        keyword_map.insert("中断", Token::Break);
        keyword_map.insert("继续", Token::Continue);
        keyword_map.insert("结构体", Token::Struct);
        keyword_map.insert("枚举", Token::Enum);
        keyword_map.insert("特征", Token::Trait);
        keyword_map.insert("实现", Token::Impl);
        keyword_map.insert("为", Token::For);
        keyword_map.insert("匹配", Token::Match);
        keyword_map.insert("当", Token::While);
        keyword_map.insert("公有的", Token::Pub);
        keyword_map.insert("私有的", Token::Priv);
        keyword_map.insert("引入", Token::Use);
        keyword_map.insert("模组", Token::Mod);
        keyword_map.insert("外部", Token::Extern);
        keyword_map.insert("异步", Token::Async);
        keyword_map.insert("等待", Token::Await);
        keyword_map.insert("移动", Token::Move);
        keyword_map.insert("克隆", Token::Clone);
        keyword_map.insert("借用", Token::Ref);
        keyword_map.insert("可变", Token::Mut);
        keyword_map.insert("自身", Token::SelfKeyword);
        keyword_map.insert("真", Token::True);
        keyword_map.insert("假", Token::False);
        keyword_map.insert("空", Token::Null);
        keyword_map.insert("单位", Token::Unit);
        keyword_map.insert("类型", Token::Type);
        keyword_map.insert("令", Token::LetAlt);
        keyword_map.insert("是", Token::Is);
        keyword_map.insert("不是", Token::Not);
        keyword_map.insert("或", Token::Or);
        keyword_map.insert("且", Token::And);
        keyword_map.insert("模式", Token::Pattern);
        keyword_map.insert("范围", Token::Range);
        keyword_map.insert("守卫", Token::Guard);
        keyword_map.insert("泛型", Token::Generic);
        keyword_map.insert("全称", Token::ForAll);
        keyword_map.insert("默认", Token::Default);
        keyword_map.insert("同步", Token::Sync);
        keyword_map.insert("大小", Token::Sized);
        keyword_map.insert("迭代器", Token::IntoIterator);
        keyword_map.insert("宏", Token::Macro);
        keyword_map.insert("宏规则", Token::MacroRules);
        keyword_map.insert("过程式", Token::Procedural);
        keyword_map.insert("函数式", Token::Functional);
        keyword_map.insert("属性", Token::Attribute);
        keyword_map.insert("派生", Token::Derive);
        keyword_map.insert("闭包", Token::Closure);
        keyword_map.insert("捕获", Token::Capture);
        keyword_map.insert("捕获引用", Token::CaptureRef);
        keyword_map.insert("捕获值", Token::CaptureValue);
        keyword_map.insert("迭代", Token::Iterator);
        keyword_map.insert("下一个", Token::Next);
        keyword_map.insert("项", Token::Item);
        keyword_map.insert("收集", Token::Collect);
        keyword_map.insert("链", Token::Chain);
        keyword_map.insert("过滤", Token::Filter);
        keyword_map.insert("折叠", Token::Fold);
        keyword_map.insert("映射", Token::Map);
        keyword_map.insert("结果", Token::Result);
        keyword_map.insert("成功", Token::Ok);
        keyword_map.insert("错误", Token::Err);
        keyword_map.insert("尝试", Token::Try);
        keyword_map.insert("捕获异常", Token::Catch);
        keyword_map.insert("异常类型", Token::Error);
        keyword_map.insert("上下文", Token::Context);
        keyword_map.insert("抛出", Token::Throw);
        keyword_map.insert("未来", Token::Future);
        keyword_map.insert("产出", Token::Yield);
        keyword_map.insert("流", Token::Stream);
        keyword_map.insert("ECS", Token::Ecs);
        keyword_map.insert("实体", Token::Entity);
        keyword_map.insert("组件", Token::Component);
        keyword_map.insert("系统", Token::System);
        keyword_map.insert("Actor", Token::Actor);
        keyword_map.insert("消息", Token::Message);
        keyword_map.insert("发送", Token::Send);
        keyword_map.insert("接收", Token::Receive);
        keyword_map.insert("并发", Token::Concurrency);
        keyword_map.insert("atomic", Token::Atomic);
        keyword_map.insert("relaxed", Token::Relaxed);
        keyword_map.insert("acquire", Token::Acquire);
        keyword_map.insert("release", Token::Release);
        keyword_map.insert("acqrel", Token::AcqRel);
        keyword_map.insert("seqcst", Token::SeqCst);
        keyword_map.insert("consume", Token::Consume);
        keyword_map.insert("happensbefore", Token::HappensBefore);
        keyword_map.insert("volatile", Token::Volatile);
        keyword_map.insert("memorybarrier", Token::MemoryBarrier);
        keyword_map.insert("wait", Token::Wait);
        keyword_map.insert("notify", Token::Notify);
        keyword_map.insert("notifyall", Token::NotifyAll);
        keyword_map.insert("datadependency", Token::DataDependency);
        keyword_map.insert("effect", Token::Effect);
        keyword_map.insert("ability", Token::Ability);
        keyword_map.insert("io", Token::IO);
        keyword_map.insert("exception", Token::Exception);
        keyword_map.insert("state", Token::State);
        keyword_map.insert("asynceffect", Token::AsyncEffect);
        keyword_map.insert("原子", Token::Atomic);
        keyword_map.insert("松弛", Token::Relaxed);
        keyword_map.insert("获取", Token::Acquire);
        keyword_map.insert("释放", Token::Release);
        keyword_map.insert("获取释放", Token::AcqRel);
        keyword_map.insert("顺序一致", Token::SeqCst);
        keyword_map.insert("消费", Token::Consume);
        keyword_map.insert("发生前", Token::HappensBefore);
        keyword_map.insert("易变", Token::Volatile);
        keyword_map.insert("内存屏障", Token::MemoryBarrier);
        keyword_map.insert("等待", Token::Wait);
        keyword_map.insert("通知", Token::Notify);
        keyword_map.insert("通知全部", Token::NotifyAll);
        keyword_map.insert("数据依赖", Token::DataDependency);
        keyword_map.insert("效果", Token::Effect);
        keyword_map.insert("能力", Token::Ability);
        keyword_map.insert("输入输出", Token::IO);
        keyword_map.insert("异常", Token::Exception);
        keyword_map.insert("状态", Token::State);
        keyword_map.insert("异步效果", Token::AsyncEffect);
        keyword_map.insert("linkedlist", Token::LinkedList);
        keyword_map.insert("listnode", Token::ListNode);
        keyword_map.insert("pushfront", Token::PushFront);
        keyword_map.insert("pushback", Token::PushBack);
        keyword_map.insert("popfront", Token::PopFront);
        keyword_map.insert("popback", Token::PopBack);
        keyword_map.insert("front", Token::Front);
        keyword_map.insert("back", Token::Back);
        keyword_map.insert("insert", Token::Insert);
        keyword_map.insert("erase", Token::Erase);
        keyword_map.insert("clear", Token::Clear);
        keyword_map.insert("splice", Token::Splice);
        keyword_map.insert("merge", Token::Merge);
        keyword_map.insert("reverse", Token::Reverse);
        keyword_map.insert("sort", Token::Sort);
        keyword_map.insert("unique", Token::Unique);
        keyword_map.insert("remove", Token::Remove);
        keyword_map.insert("双链表", Token::LinkedList);
        keyword_map.insert("链表节点", Token::ListNode);
        keyword_map.insert("前推", Token::PushFront);
        keyword_map.insert("后推", Token::PushBack);
        keyword_map.insert("前弹", Token::PopFront);
        keyword_map.insert("后弹", Token::PopBack);
        keyword_map.insert("前端", Token::Front);
        keyword_map.insert("后端", Token::Back);
        keyword_map.insert("插入", Token::Insert);
        keyword_map.insert("擦除", Token::Erase);
        keyword_map.insert("清空", Token::Clear);
        keyword_map.insert("拼接", Token::Splice);
        keyword_map.insert("合并", Token::Merge);
        keyword_map.insert("反转", Token::Reverse);
        keyword_map.insert("排序", Token::Sort);
        keyword_map.insert("唯一", Token::Unique);
        keyword_map.insert("移除", Token::Remove);
        keyword_map.insert("结束", Token::End);

        Lexer {
            source,
            file_id: _file_id,
            pos: 0,
            line: 1,
            line_start: 0,
            keyword_map,
            errors: Vec::new(),
        }
    }

    pub fn errors(&self) -> &[LexerError] {
        &self.errors
    }

    pub fn take_errors(&mut self) -> Vec<LexerError> {
        std::mem::take(&mut self.errors)
    }

    fn report_error(&mut self, message: String, span: Span) {
        self.errors.push(LexerError::new(message, span));
    }

    fn report_unknown_char(&mut self, c: char, start: usize, start_line: usize, start_col: usize) {
        let span = Span::new(self.file_id, start, start + c.len_utf8(), start_line, start_col);
        let message = format!("unknown character '{}'", c);
        self.report_error(message, span);
    }

    pub fn tokenize(&mut self) -> Vec<SpannedToken> {
        let mut tokens = Vec::new();
        while let Some(token) = self.next_token() {
            if token.token != Token::Whitespace && token.token != Token::Comment && token.token != Token::BlockComment {
                tokens.push(token);
            }
        }
        tokens
    }

    fn next_token(&mut self) -> Option<SpannedToken> {
        if self.pos >= self.source.len() {
            let span = Span::new(self.file_id, self.pos, self.pos, self.line, self.pos - self.line_start);
            return Some(SpannedToken::new(Token::Eof, span));
        }

        let start = self.pos;
        let start_line = self.line;
        let start_col = self.pos - self.line_start;

        let c = self.source[self.pos..].chars().next()?;

        match c {
            '+' => {
                self.pos += 1;
                let token = if self.source[self.pos..].starts_with('=') {
                    self.pos += 1;
                    Token::PlusEq
                } else {
                    Token::Plus
                };
                let span = Span::new(self.file_id, start, self.pos, start_line, start_col);
                Some(SpannedToken::new(token, span))
            }
            '-' => {
                self.pos += 1;
                let token = if self.source[self.pos..].starts_with('=') {
                    self.pos += 1;
                    Token::MinusEq
                } else if self.source[self.pos..].starts_with('>') {
                    self.pos += 1;
                    Token::Arrow
                } else {
                    Token::Minus
                };
                let span = Span::new(self.file_id, start, self.pos, start_line, start_col);
                Some(SpannedToken::new(token, span))
            }
            '*' => {
                self.pos += 1;
                let token = if self.source[self.pos..].starts_with('=') {
                    self.pos += 1;
                    Token::StarEq
                } else {
                    Token::Star
                };
                let span = Span::new(self.file_id, start, self.pos, start_line, start_col);
                Some(SpannedToken::new(token, span))
            }
            '/' => {
                self.pos += 1;
                if self.source[self.pos..].starts_with('/') {
                    self.skip_line_comment();
                    return self.next_token();
                } else if self.source[self.pos..].starts_with('*') {
                    self.pos += 1;
                    self.skip_block_comment();
                    return self.next_token();
                }
                let token = if self.source[self.pos..].starts_with('=') {
                    self.pos += 1;
                    Token::SlashEq
                } else {
                    Token::Slash
                };
                let span = Span::new(self.file_id, start, self.pos, start_line, start_col);
                Some(SpannedToken::new(token, span))
            }
            '%' => {
                self.pos += 1;
                let token = if self.source[self.pos..].starts_with('=') {
                    self.pos += 1;
                    Token::PercentEq
                } else {
                    Token::Percent
                };
                let span = Span::new(self.file_id, start, self.pos, start_line, start_col);
                Some(SpannedToken::new(token, span))
            }
            '=' => {
                self.pos += 1;
                let token = if self.source[self.pos..].starts_with('=') {
                    self.pos += 1;
                    Token::EqEq
                } else if self.source[self.pos..].starts_with('>') {
                    self.pos += 1;
                    Token::Arrow
                } else {
                    Token::Eq
                };
                let span = Span::new(self.file_id, start, self.pos, start_line, start_col);
                Some(SpannedToken::new(token, span))
            }
            '!' => {
                self.pos += 1;
                let token = if self.source[self.pos..].starts_with('=') {
                    self.pos += 1;
                    Token::Neq
                } else {
                    Token::Bang
                };
                let span = Span::new(self.file_id, start, self.pos, start_line, start_col);
                Some(SpannedToken::new(token, span))
            }
            '<' => {
                self.pos += 1;
                let token = if self.source[self.pos..].starts_with('=') {
                    self.pos += 1;
                    Token::Lte
                } else if self.source[self.pos..].starts_with('<') {
                    self.pos += 1;
                    if self.source[self.pos..].starts_with('=') {
                        self.pos += 1;
                        Token::LShiftEq
                    } else {
                        Token::LShift
                    }
                } else {
                    Token::Lt
                };
                let span = Span::new(self.file_id, start, self.pos, start_line, start_col);
                Some(SpannedToken::new(token, span))
            }
            '>' => {
                self.pos += 1;
                let token = if self.source[self.pos..].starts_with('=') {
                    self.pos += 1;
                    Token::Gte
                } else if self.source[self.pos..].starts_with('>') {
                    self.pos += 1;
                    if self.source[self.pos..].starts_with('=') {
                        self.pos += 1;
                        Token::RShiftEq
                    } else {
                        Token::RShift
                    }
                } else {
                    Token::Gt
                };
                let span = Span::new(self.file_id, start, self.pos, start_line, start_col);
                Some(SpannedToken::new(token, span))
            }
            '&' => {
                self.pos += 1;
                let token = if self.source[self.pos..].starts_with('&') {
                    self.pos += 1;
                    Token::AndAnd
                } else if self.source[self.pos..].starts_with('=') {
                    self.pos += 1;
                    Token::AndEq
                } else {
                    Token::Ampersand
                };
                let span = Span::new(self.file_id, start, self.pos, start_line, start_col);
                Some(SpannedToken::new(token, span))
            }
            '|' => {
                self.pos += 1;
                let token = if self.source[self.pos..].starts_with('|') {
                    self.pos += 1;
                    Token::OrOr
                } else if self.source[self.pos..].starts_with('=') {
                    self.pos += 1;
                    Token::PipeEq
                } else if self.source[self.pos..].starts_with('>') {
                    self.pos += 1;
                    Token::ThinArrow
                } else {
                    Token::Pipe
                };
                let span = Span::new(self.file_id, start, self.pos, start_line, start_col);
                Some(SpannedToken::new(token, span))
            }
            '^' => {
                self.pos += 1;
                let token = if self.source[self.pos..].starts_with('=') {
                    self.pos += 1;
                    Token::CaretEq
                } else {
                    Token::Caret
                };
                let span = Span::new(self.file_id, start, self.pos, start_line, start_col);
                Some(SpannedToken::new(token, span))
            }
            ':' => {
                self.pos += 1;
                let token = if self.source[self.pos..].starts_with(':') {
                    self.pos += 1;
                    Token::DoubleColon
                } else {
                    Token::Colon
                };
                let span = Span::new(self.file_id, start, self.pos, start_line, start_col);
                Some(SpannedToken::new(token, span))
            }
            ';' => self.single_token(Token::Semicolon, start, start_line, start_col),
            ',' => self.single_token(Token::Comma, start, start_line, start_col),
            '.' => {
                self.pos += 1;
                let token = if self.source[self.pos..].starts_with('.') {
                    self.pos += 1;
                    if self.source[self.pos..].starts_with('.') {
                        self.pos += 1;
                        Token::DotDotDot
                    } else {
                        Token::DotDot
                    }
                } else {
                    Token::Dot
                };
                let span = Span::new(self.file_id, start, self.pos, start_line, start_col);
                Some(SpannedToken::new(token, span))
            }
            '(' => self.single_token(Token::LParen, start, start_line, start_col),
            ')' => self.single_token(Token::RParen, start, start_line, start_col),
            '{' => self.single_token(Token::LBrace, start, start_line, start_col),
            '}' => self.single_token(Token::RBrace, start, start_line, start_col),
            '[' => self.single_token(Token::LBracket, start, start_line, start_col),
            ']' => self.single_token(Token::RBracket, start, start_line, start_col),
            '<' => self.single_token(Token::LAngle, start, start_line, start_col),
            '>' => self.single_token(Token::RAngle, start, start_line, start_col),
            '#' => self.skip_line_comment(),
            '@' => self.single_token(Token::At, start, start_line, start_col),
            '$' => self.single_token(Token::Dollar, start, start_line, start_col),
            '?' => {
                self.pos += 1;
                let token = if self.source[self.pos..].starts_with('?') {
                    self.pos += 1;
                    Token::DoubleQuestion
                } else {
                    Token::Question
                };
                let span = Span::new(self.file_id, start, self.pos, start_line, start_col);
                Some(SpannedToken::new(token, span))
            }
            '"' => self.read_string(start, start_line, start_col),
            '\'' => self.read_char(start, start_line, start_col),
            'r' if self.source[self.pos..].starts_with("r#") => self.read_raw_string(start, start_line, start_col),
            'b' if self.source[self.pos..].starts_with("b\"") => self.read_byte_string(start, start_line, start_col),
            'b' if self.source[self.pos..].starts_with("b'") => self.read_byte(start, start_line, start_col),
            c if c.is_whitespace() => {
                self.skip_whitespace();
                self.next_token()
            }
            c if c.is_alphabetic() || c == '_' => self.read_identifier(start, start_line, start_col),
            c if c.is_ascii_digit() => self.read_number(start, start_line, start_col),
            _ => {
                self.report_unknown_char(c, start, start_line, start_col);
                self.pos += 1;
                let span = Span::new(self.file_id, start, self.pos, start_line, start_col);
                Some(SpannedToken::new(Token::Unknown, span))
            }
        }
    }

    fn single_token(&mut self, token: Token, start: usize, start_line: usize, start_col: usize) -> Option<SpannedToken> {
        self.pos += 1;
        let span = Span::new(self.file_id, start, self.pos, start_line, start_col);
        Some(SpannedToken::new(token, span))
    }

    fn skip_whitespace(&mut self) {
        while self.pos < self.source.len() {
            let c = self.source[self.pos..].chars().next().unwrap();
            if c.is_whitespace() {
                if c == '\n' {
                    self.line += 1;
                    self.line_start = self.pos + 1;
                }
                self.pos += 1;
            } else {
                break;
            }
        }
    }

    fn skip_line_comment(&mut self) -> Option<SpannedToken> {
        while self.pos < self.source.len() && !self.source[self.pos..].starts_with('\n') {
            self.pos += 1;
        }
        self.next_token()
    }

    fn skip_block_comment(&mut self) {
        let mut depth = 1;
        while depth > 0 && self.pos < self.source.len() {
            if self.source[self.pos..].starts_with("/*") {
                depth += 1;
                self.pos += 2;
            } else if self.source[self.pos..].starts_with("*/") {
                depth -= 1;
                self.pos += 2;
            } else {
                self.pos += 1;
            }
        }
    }

    fn read_string(&mut self, start: usize, start_line: usize, start_col: usize) -> Option<SpannedToken> {
        self.pos += 1;
        while self.pos < self.source.len() && !self.source[self.pos..].starts_with('"') {
            if self.source[self.pos..].starts_with('\\') {
                self.pos += 2;
            } else {
                self.pos += 1;
            }
        }
        if self.pos >= self.source.len() {
            let span = Span::new(self.file_id, start, self.pos, start_line, start_col);
            self.report_error("unterminated string literal".to_string(), span);
        } else {
            self.pos += 1;
        }
        let span = Span::new(self.file_id, start, self.pos, start_line, start_col);
        Some(SpannedToken::new(Token::String, span))
    }

    fn read_raw_string(&mut self, start: usize, start_line: usize, start_col: usize) -> Option<SpannedToken> {
        self.pos += 2;
        let hash_count = self.count_leading_hashes();
        
        while self.pos < self.source.len() {
            if self.source[self.pos..].starts_with('"') {
                let end_pos = self.pos + 1;
                if self.count_hashes_at(end_pos) == hash_count {
                    self.pos = end_pos + hash_count;
                    let span = Span::new(self.file_id, start, self.pos, start_line, start_col);
                    return Some(SpannedToken::new(Token::RawString, span));
                }
            }
            self.pos += 1;
        }
        
        let span = Span::new(self.file_id, start, self.pos, start_line, start_col);
        self.report_error("unterminated raw string literal".to_string(), span);
        Some(SpannedToken::new(Token::RawString, span))
    }

    fn count_leading_hashes(&mut self) -> usize {
        let mut count = 0;
        while self.pos < self.source.len() && self.source[self.pos..].starts_with('#') {
            count += 1;
            self.pos += 1;
        }
        count
    }

    fn count_hashes_at(&self, pos: usize) -> usize {
        let mut count = 0;
        let mut pos = pos;
        while pos < self.source.len() && self.source[pos..].starts_with('#') {
            count += 1;
            pos += 1;
        }
        count
    }

    fn read_byte_string(&mut self, start: usize, start_line: usize, start_col: usize) -> Option<SpannedToken> {
        self.pos += 2;
        while self.pos < self.source.len() && !self.source[self.pos..].starts_with('"') {
            if self.source[self.pos..].starts_with('\\') {
                self.pos += 2;
            } else {
                self.pos += 1;
            }
        }
        if self.pos >= self.source.len() {
            let span = Span::new(self.file_id, start, self.pos, start_line, start_col);
            self.report_error("unterminated byte string literal".to_string(), span);
        } else {
            self.pos += 1;
        }
        let span = Span::new(self.file_id, start, self.pos, start_line, start_col);
        Some(SpannedToken::new(Token::ByteString, span))
    }

    fn read_char(&mut self, start: usize, start_line: usize, start_col: usize) -> Option<SpannedToken> {
        self.pos += 1;
        if self.pos < self.source.len() && self.source[self.pos..].starts_with('\\') {
            self.pos += 2;
        } else if self.pos < self.source.len() {
            self.pos += 1;
        }
        if self.pos >= self.source.len() || !self.source[self.pos..].starts_with('\'') {
            let span = Span::new(self.file_id, start, self.pos, start_line, start_col);
            self.report_error("unterminated character literal".to_string(), span);
        } else {
            self.pos += 1;
        }
        let span = Span::new(self.file_id, start, self.pos, start_line, start_col);
        Some(SpannedToken::new(Token::Char, span))
    }

    fn read_byte(&mut self, start: usize, start_line: usize, start_col: usize) -> Option<SpannedToken> {
        self.pos += 2;
        if self.pos < self.source.len() && self.source[self.pos..].starts_with('\\') {
            self.pos += 2;
        } else if self.pos < self.source.len() {
            self.pos += 1;
        }
        if self.pos >= self.source.len() || !self.source[self.pos..].starts_with('\'') {
            let span = Span::new(self.file_id, start, self.pos, start_line, start_col);
            self.report_error("unterminated byte literal".to_string(), span);
        } else {
            self.pos += 1;
        }
        let span = Span::new(self.file_id, start, self.pos, start_line, start_col);
        Some(SpannedToken::new(Token::Byte, span))
    }

    fn read_identifier(&mut self, start: usize, start_line: usize, start_col: usize) -> Option<SpannedToken> {
        let mut end = self.pos;
        while end < self.source.len() {
            let c = self.source[end..].chars().next().unwrap();
            if c.is_alphanumeric() || c == '_' || c == '$' || c as u32 > 0x4E00 {
                end += c.len_utf8();
            } else {
                break;
            }
        }

        let text = &self.source[self.pos..end];
        self.pos = end;

        let token = if let Some(&keyword) = self.keyword_map.get(text) {
            keyword
        } else {
            Token::Identifier
        };

        let span = Span::new(self.file_id, start, self.pos, start_line, start_col);
        Some(SpannedToken::new(token, span))
    }

    fn read_number(&mut self, start: usize, start_line: usize, start_col: usize) -> Option<SpannedToken> {
        let original_pos = self.pos;
        
        if self.source[self.pos..].starts_with("0x") || self.source[self.pos..].starts_with("0X") {
            self.pos += 2;
            while self.pos < self.source.len() {
                let c = self.source[self.pos..].chars().next().unwrap();
                if c.is_ascii_hexdigit() || c == '_' {
                    self.pos += 1;
                } else {
                    break;
                }
            }
            let span = Span::new(self.file_id, start, self.pos, start_line, start_col);
            return Some(SpannedToken::new(Token::Int, span));
        }
        
        if self.source[self.pos..].starts_with("0b") || self.source[self.pos..].starts_with("0B") {
            self.pos += 2;
            while self.pos < self.source.len() {
                let c = self.source[self.pos..].chars().next().unwrap();
                if c == '0' || c == '1' || c == '_' {
                    self.pos += 1;
                } else {
                    break;
                }
            }
            let span = Span::new(self.file_id, start, self.pos, start_line, start_col);
            return Some(SpannedToken::new(Token::Int, span));
        }
        
        if self.source[self.pos..].starts_with("0o") || self.source[self.pos..].starts_with("0O") {
            self.pos += 2;
            while self.pos < self.source.len() {
                let c = self.source[self.pos..].chars().next().unwrap();
                if ('0'..='7').contains(&c) || c == '_' {
                    self.pos += 1;
                } else {
                    break;
                }
            }
            let span = Span::new(self.file_id, start, self.pos, start_line, start_col);
            return Some(SpannedToken::new(Token::Int, span));
        }
        
        if self.source[self.pos..].starts_with("0t") || self.source[self.pos..].starts_with("0T") {
            self.pos += 2;
            while self.pos < self.source.len() {
                let c = self.source[self.pos..].chars().next().unwrap();
                if ('0'..='2').contains(&c) || c == '_' {
                    self.pos += 1;
                } else {
                    break;
                }
            }
            let span = Span::new(self.file_id, start, self.pos, start_line, start_col);
            return Some(SpannedToken::new(Token::Int, span));
        }
        
        if self.source[self.pos..].starts_with("0e") || self.source[self.pos..].starts_with("0E") {
            self.pos += 2;
            while self.pos < self.source.len() {
                let c = self.source[self.pos..].chars().next().unwrap();
                if ('0'..='2').contains(&c) || c == '_' {
                    self.pos += 1;
                } else {
                    break;
                }
            }
            let span = Span::new(self.file_id, start, self.pos, start_line, start_col);
            return Some(SpannedToken::new(Token::Int, span));
        }
        
        if self.source[self.pos..].starts_with("0d") || self.source[self.pos..].starts_with("0D") {
            self.pos += 2;
            while self.pos < self.source.len() {
                let c = self.source[self.pos..].chars().next().unwrap();
                if ('0'..='9').contains(&c) || ('a'..='b').contains(&c) || ('A'..='B').contains(&c) || c == '_' {
                    self.pos += 1;
                } else {
                    break;
                }
            }
            let span = Span::new(self.file_id, start, self.pos, start_line, start_col);
            return Some(SpannedToken::new(Token::Int, span));
        }
        
        if self.source[self.pos..].starts_with("0h") || self.source[self.pos..].starts_with("0H") {
            self.pos += 2;
            while self.pos < self.source.len() {
                let c = self.source[self.pos..].chars().next().unwrap();
                if ('0'..='9').contains(&c) || ('a'..='n').contains(&c) || ('A'..='N').contains(&c) || c == '_' {
                    self.pos += 1;
                } else {
                    break;
                }
            }
            let span = Span::new(self.file_id, start, self.pos, start_line, start_col);
            return Some(SpannedToken::new(Token::Int, span));
        }
        
        if self.source[self.pos..].starts_with("0s") || self.source[self.pos..].starts_with("0S") {
            self.pos += 2;
            while self.pos < self.source.len() {
                let c = self.source[self.pos..].chars().next().unwrap();
                if ('0'..='9').contains(&c) || ('a'..='z').contains(&c) || ('A'..='Z').contains(&c) || c == '_' {
                    self.pos += 1;
                } else {
                    break;
                }
            }
            let span = Span::new(self.file_id, start, self.pos, start_line, start_col);
            return Some(SpannedToken::new(Token::Int, span));
        }

        let mut has_dot = false;
        let mut has_exponent = false;
        
        while self.pos < self.source.len() {
            let c = self.source[self.pos..].chars().next().unwrap();
            if c.is_ascii_digit() {
                self.pos += 1;
            } else if c == '.' && !has_dot {
                has_dot = true;
                self.pos += 1;
            } else if (c == 'e' || c == 'E') && !has_exponent {
                has_exponent = true;
                self.pos += 1;
                if self.pos < self.source.len() {
                    let next_c = self.source[self.pos..].chars().next().unwrap();
                    if next_c == '+' || next_c == '-' {
                        self.pos += 1;
                    }
                }
            } else if c == '_' {
                self.pos += 1;
            } else {
                break;
            }
        }
        
        if self.pos == original_pos {
            self.pos += 1;
            let span = Span::new(self.file_id, start, self.pos, start_line, start_col);
            return Some(SpannedToken::new(Token::Unknown, span));
        }

        let span = Span::new(self.file_id, start, self.pos, start_line, start_col);
        Some(SpannedToken::new(if has_dot || has_exponent { Token::Float } else { Token::Int }, span))
    }
}

pub struct TokenStream {
    tokens: Vec<SpannedToken>,
    current: usize,
}

impl TokenStream {
    pub fn new(tokens: Vec<SpannedToken>) -> Self {
        TokenStream {
            tokens,
            current: 0,
        }
    }

    pub fn peek(&self) -> Option<&SpannedToken> {
        self.tokens.get(self.current)
    }

    pub fn peek_n(&self, n: usize) -> Option<&SpannedToken> {
        self.tokens.get(self.current + n)
    }

    pub fn next(&mut self) -> Option<&SpannedToken> {
        let token = self.tokens.get(self.current);
        self.current += 1;
        token
    }

    pub fn expect(&mut self, expected: Token) -> Result<&SpannedToken, ()> {
        if let Some(token) = self.peek() {
            if token.token == expected {
                return Ok(self.next().unwrap());
            }
        }
        Err(())
    }

    pub fn at_end(&self) -> bool {
        self.current >= self.tokens.len()
    }

    pub fn remaining(&self) -> &[SpannedToken] {
        &self.tokens[self.current..]
    }

    pub fn consume(&mut self, token: Token) -> bool {
        if let Some(t) = self.peek() {
            if t.token == token {
                self.next();
                return true;
            }
        }
        false
    }

    pub fn position(&self) -> usize {
        self.current
    }

    pub fn set_position(&mut self, pos: usize) {
        self.current = pos;
    }

    pub fn len(&self) -> usize {
        self.tokens.len()
    }
}

pub struct Rodeo {
    strings: Vec<String>,
}

impl Rodeo {
    pub fn new() -> Self {
        Rodeo {
            strings: Vec::new(),
        }
    }

    pub fn get_or_intern(&mut self, text: &str) -> Spur {
        if let Some(idx) = self.strings.iter().position(|s| s == text) {
            Spur(idx)
        } else {
            self.strings.push(text.to_string());
            Spur(self.strings.len() - 1)
        }
    }

    pub fn resolve(&self, spur: &Spur) -> &str {
        &self.strings[spur.0]
    }

    pub fn len(&self) -> usize {
        self.strings.len()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Spur(usize);

pub fn tokenize(source: &str, file_id: FileId) -> (Vec<SpannedToken>, Rodeo, Vec<LexerError>) {
    let mut interner = Rodeo::new();
    let mut lexer = Lexer::new(source, file_id);
    let tokens = lexer.tokenize();
    let errors = lexer.take_errors();
    (tokens, interner, errors)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_keywords() {
        let source = "let x = 42;";
        let file_id = FileId(0);
        let (tokens, _) = tokenize(source, file_id);
        assert!(tokens.iter().any(|t| t.token == Token::Let));
        assert!(tokens.iter().any(|t| t.token == Token::Semicolon));
    }

    #[test]
    fn test_tokenize_chinese_keywords() {
        let source = "令 x = 42;";
        let file_id = FileId(0);
        let (tokens, _) = tokenize(source, file_id);
        assert!(tokens.iter().any(|t| t.token == Token::Let));
    }

    #[test]
    fn test_tokenize_number() {
        let source = "42";
        let file_id = FileId(0);
        let (tokens, _) = tokenize(source, file_id);
        assert!(tokens.iter().any(|t| t.token == Token::Int));
    }

    #[test]
    fn test_tokenize_float() {
        let source = "3.14";
        let file_id = FileId(0);
        let (tokens, _) = tokenize(source, file_id);
        assert!(tokens.iter().any(|t| t.token == Token::Float));
    }

    #[test]
    fn test_tokenize_hex() {
        let source = "0xFF";
        let file_id = FileId(0);
        let (tokens, _) = tokenize(source, file_id);
        assert!(tokens.iter().any(|t| t.token == Token::Int));
    }

    #[test]
    fn test_tokenize_binary() {
        let source = "0b1010";
        let file_id = FileId(0);
        let (tokens, _) = tokenize(source, file_id);
        assert!(tokens.iter().any(|t| t.token == Token::Int));
    }

    #[test]
    fn test_tokenize_octal() {
        let source = "0o755";
        let file_id = FileId(0);
        let (tokens, _) = tokenize(source, file_id);
        assert!(tokens.iter().any(|t| t.token == Token::Int));
    }

    #[test]
    fn test_tokenize_scientific() {
        let source = "1.5e10";
        let file_id = FileId(0);
        let (tokens, _) = tokenize(source, file_id);
        assert!(tokens.iter().any(|t| t.token == Token::Float));
    }

    #[test]
    fn test_tokenize_string() {
        let source = "\"hello\"";
        let file_id = FileId(0);
        let (tokens, _) = tokenize(source, file_id);
        assert!(tokens.iter().any(|t| t.token == Token::String));
    }

    #[test]
    fn test_tokenize_raw_string() {
        let source = "r#\"hello\"#";
        let file_id = FileId(0);
        let (tokens, _) = tokenize(source, file_id);
        assert!(tokens.iter().any(|t| t.token == Token::RawString));
    }

    #[test]
    fn test_tokenize_byte_string() {
        let source = "b\"hello\"";
        let file_id = FileId(0);
        let (tokens, _) = tokenize(source, file_id);
        assert!(tokens.iter().any(|t| t.token == Token::ByteString));
    }

    #[test]
    fn test_tokenize_char() {
        let source = "'a'";
        let file_id = FileId(0);
        let (tokens, _) = tokenize(source, file_id);
        assert!(tokens.iter().any(|t| t.token == Token::Char));
    }

    #[test]
    fn test_tokenize_byte() {
        let source = "b'a'";
        let file_id = FileId(0);
        let (tokens, _) = tokenize(source, file_id);
        assert!(tokens.iter().any(|t| t.token == Token::Byte));
    }

    #[test]
    fn test_tokenize_operators() {
        let source = "+ - * / % == != < > <= >= && || ! & | ^ << >>";
        let file_id = FileId(0);
        let (tokens, _) = tokenize(source, file_id);
        assert!(tokens.iter().any(|t| t.token == Token::Plus));
        assert!(tokens.iter().any(|t| t.token == Token::EqEq));
        assert!(tokens.iter().any(|t| t.token == Token::LShift));
    }

    #[test]
    fn test_tokenize_compound_assignment() {
        let source = "+= -= *= /= %= &= |= ^= <<= >>=";
        let file_id = FileId(0);
        let (tokens, _) = tokenize(source, file_id);
        assert!(tokens.iter().any(|t| t.token == Token::PlusEq));
        assert!(tokens.iter().any(|t| t.token == Token::LShiftEq));
    }

    #[test]
    fn test_tokenize_comments() {
        let source = "// line comment\nlet x = 42; /* block comment */";
        let file_id = FileId(0);
        let (tokens, _) = tokenize(source, file_id);
        assert!(tokens.iter().any(|t| t.token == Token::Let));
        assert!(!tokens.iter().any(|t| t.token == Token::Comment));
    }

    #[test]
    fn test_tokenize_underscore() {
        let source = "_";
        let file_id = FileId(0);
        let (tokens, _) = tokenize(source, file_id);
        assert!(tokens.iter().any(|t| t.token == Token::Underscore));
    }

    #[test]
    fn test_tokenize_double_colon() {
        let source = "std::collections::HashMap";
        let file_id = FileId(0);
        let (tokens, _) = tokenize(source, file_id);
        assert!(tokens.iter().filter(|t| t.token == Token::DoubleColon).count() == 2);
    }

    #[test]
    fn test_tokenize_question() {
        let source = "x ? y : z";
        let file_id = FileId(0);
        let (tokens, _) = tokenize(source, file_id);
        assert!(tokens.iter().any(|t| t.token == Token::Question));
    }

    #[test]
    fn test_tokenize_double_question() {
        let source = "x ?? y";
        let file_id = FileId(0);
        let (tokens, _) = tokenize(source, file_id);
        assert!(tokens.iter().any(|t| t.token == Token::DoubleQuestion));
    }

    #[test]
    fn test_tokenize_arrow() {
        let source = "->";
        let file_id = FileId(0);
        let (tokens, _) = tokenize(source, file_id);
        assert!(tokens.iter().any(|t| t.token == Token::ThinArrow));
    }

    #[test]
    fn test_tokenize_thick_arrow() {
        let source = "=>";
        let file_id = FileId(0);
        let (tokens, _) = tokenize(source, file_id);
        assert!(tokens.iter().any(|t| t.token == Token::Arrow));
    }

    #[test]
    fn test_tokenize_at_sign() {
        let source = "@attribute";
        let file_id = FileId(0);
        let (tokens, _) = tokenize(source, file_id);
        assert!(tokens.iter().any(|t| t.token == Token::At));
    }

    #[test]
    fn test_tokenize_dollar() {
        let source = "$lifetime";
        let file_id = FileId(0);
        let (tokens, _) = tokenize(source, file_id);
        assert!(tokens.iter().any(|t| t.token == Token::Dollar));
    }

    #[test]
    fn test_intern() {
        let mut interner = Rodeo::new();
        let id1 = interner.get_or_intern("hello");
        let id2 = interner.get_or_intern("world");
        let id3 = interner.get_or_intern("hello");
        
        assert_eq!(id1, id3);
        assert_ne!(id1, id2);
        assert_eq!(interner.resolve(&id1), "hello");
        assert_eq!(interner.resolve(&id2), "world");
    }

    #[test]
    fn test_tokenize_extended_memory_order() {
        let source = "consume happensbefore volatile memorybarrier wait notify notifyall datadependency";
        let file_id = FileId(0);
        let (tokens, _) = tokenize(source, file_id);
        assert!(tokens.iter().any(|t| t.token == Token::Consume));
        assert!(tokens.iter().any(|t| t.token == Token::HappensBefore));
        assert!(tokens.iter().any(|t| t.token == Token::Volatile));
        assert!(tokens.iter().any(|t| t.token == Token::MemoryBarrier));
        assert!(tokens.iter().any(|t| t.token == Token::Wait));
        assert!(tokens.iter().any(|t| t.token == Token::Notify));
        assert!(tokens.iter().any(|t| t.token == Token::NotifyAll));
        assert!(tokens.iter().any(|t| t.token == Token::DataDependency));
    }

    #[test]
    fn test_tokenize_chinese_extended_memory_order() {
        let source = "消费 发生前 易变 内存屏障 等待 通知 通知全部 数据依赖";
        let file_id = FileId(0);
        let (tokens, _) = tokenize(source, file_id);
        assert!(tokens.iter().any(|t| t.token == Token::Consume));
        assert!(tokens.iter().any(|t| t.token == Token::HappensBefore));
        assert!(tokens.iter().any(|t| t.token == Token::Volatile));
        assert!(tokens.iter().any(|t| t.token == Token::MemoryBarrier));
        assert!(tokens.iter().any(|t| t.token == Token::Wait));
        assert!(tokens.iter().any(|t| t.token == Token::Notify));
        assert!(tokens.iter().any(|t| t.token == Token::NotifyAll));
        assert!(tokens.iter().any(|t| t.token == Token::DataDependency));
    }

    #[test]
    fn test_tokenize_language_features() {
        let source = "pattern range guard generic forall default sync sized intoiterator";
        let file_id = FileId(0);
        let (tokens, _) = tokenize(source, file_id);
        assert!(tokens.iter().any(|t| t.token == Token::Pattern));
        assert!(tokens.iter().any(|t| t.token == Token::Range));
        assert!(tokens.iter().any(|t| t.token == Token::Guard));
        assert!(tokens.iter().any(|t| t.token == Token::Generic));
        assert!(tokens.iter().any(|t| t.token == Token::ForAll));
        assert!(tokens.iter().any(|t| t.token == Token::Default));
        assert!(tokens.iter().any(|t| t.token == Token::Sync));
        assert!(tokens.iter().any(|t| t.token == Token::Sized));
        assert!(tokens.iter().any(|t| t.token == Token::IntoIterator));
    }

    #[test]
    fn test_tokenize_chinese_language_features() {
        let source = "模式 范围 守卫 泛型 全称 默认 同步 大小 迭代器";
        let file_id = FileId(0);
        let (tokens, _) = tokenize(source, file_id);
        assert!(tokens.iter().any(|t| t.token == Token::Pattern));
        assert!(tokens.iter().any(|t| t.token == Token::Range));
        assert!(tokens.iter().any(|t| t.token == Token::Guard));
        assert!(tokens.iter().any(|t| t.token == Token::Generic));
        assert!(tokens.iter().any(|t| t.token == Token::ForAll));
        assert!(tokens.iter().any(|t| t.token == Token::Default));
        assert!(tokens.iter().any(|t| t.token == Token::Sync));
        assert!(tokens.iter().any(|t| t.token == Token::Sized));
        assert!(tokens.iter().any(|t| t.token == Token::IntoIterator));
    }

    #[test]
    fn test_tokenize_macro_closure() {
        let source = "macro macrorules procedural functional attribute derive closure capture captureref capturevalue";
        let file_id = FileId(0);
        let (tokens, _) = tokenize(source, file_id);
        assert!(tokens.iter().any(|t| t.token == Token::Macro));
        assert!(tokens.iter().any(|t| t.token == Token::MacroRules));
        assert!(tokens.iter().any(|t| t.token == Token::Procedural));
        assert!(tokens.iter().any(|t| t.token == Token::Functional));
        assert!(tokens.iter().any(|t| t.token == Token::Attribute));
        assert!(tokens.iter().any(|t| t.token == Token::Derive));
        assert!(tokens.iter().any(|t| t.token == Token::Closure));
        assert!(tokens.iter().any(|t| t.token == Token::Capture));
        assert!(tokens.iter().any(|t| t.token == Token::CaptureRef));
        assert!(tokens.iter().any(|t| t.token == Token::CaptureValue));
    }

    #[test]
    fn test_tokenize_chinese_macro_closure() {
        let source = "宏 宏规则 过程式 函数式 属性 派生 闭包 捕获 捕获引用 捕获值";
        let file_id = FileId(0);
        let (tokens, _) = tokenize(source, file_id);
        assert!(tokens.iter().any(|t| t.token == Token::Macro));
        assert!(tokens.iter().any(|t| t.token == Token::MacroRules));
        assert!(tokens.iter().any(|t| t.token == Token::Procedural));
        assert!(tokens.iter().any(|t| t.token == Token::Functional));
        assert!(tokens.iter().any(|t| t.token == Token::Attribute));
        assert!(tokens.iter().any(|t| t.token == Token::Derive));
        assert!(tokens.iter().any(|t| t.token == Token::Closure));
        assert!(tokens.iter().any(|t| t.token == Token::Capture));
        assert!(tokens.iter().any(|t| t.token == Token::CaptureRef));
        assert!(tokens.iter().any(|t| t.token == Token::CaptureValue));
    }

    #[test]
    fn test_tokenize_iterator_error_handling() {
        let source = "iterator next item collect chain filter fold map result ok err try catch error context throw";
        let file_id = FileId(0);
        let (tokens, _) = tokenize(source, file_id);
        assert!(tokens.iter().any(|t| t.token == Token::Iterator));
        assert!(tokens.iter().any(|t| t.token == Token::Next));
        assert!(tokens.iter().any(|t| t.token == Token::Item));
        assert!(tokens.iter().any(|t| t.token == Token::Collect));
        assert!(tokens.iter().any(|t| t.token == Token::Chain));
        assert!(tokens.iter().any(|t| t.token == Token::Filter));
        assert!(tokens.iter().any(|t| t.token == Token::Fold));
        assert!(tokens.iter().any(|t| t.token == Token::Map));
        assert!(tokens.iter().any(|t| t.token == Token::Result));
        assert!(tokens.iter().any(|t| t.token == Token::Ok));
        assert!(tokens.iter().any(|t| t.token == Token::Err));
        assert!(tokens.iter().any(|t| t.token == Token::Try));
        assert!(tokens.iter().any(|t| t.token == Token::Catch));
        assert!(tokens.iter().any(|t| t.token == Token::Error));
        assert!(tokens.iter().any(|t| t.token == Token::Context));
        assert!(tokens.iter().any(|t| t.token == Token::Throw));
    }

    #[test]
    fn test_tokenize_chinese_iterator_error_handling() {
        let source = "迭代 下一个 项 收集 链 过滤 折叠 映射 结果 成功 错误 尝试 捕获异常 异常类型 上下文 抛出";
        let file_id = FileId(0);
        let (tokens, _) = tokenize(source, file_id);
        assert!(tokens.iter().any(|t| t.token == Token::Iterator));
        assert!(tokens.iter().any(|t| t.token == Token::Next));
        assert!(tokens.iter().any(|t| t.token == Token::Item));
        assert!(tokens.iter().any(|t| t.token == Token::Collect));
        assert!(tokens.iter().any(|t| t.token == Token::Chain));
        assert!(tokens.iter().any(|t| t.token == Token::Filter));
        assert!(tokens.iter().any(|t| t.token == Token::Fold));
        assert!(tokens.iter().any(|t| t.token == Token::Map));
        assert!(tokens.iter().any(|t| t.token == Token::Result));
        assert!(tokens.iter().any(|t| t.token == Token::Ok));
        assert!(tokens.iter().any(|t| t.token == Token::Err));
        assert!(tokens.iter().any(|t| t.token == Token::Try));
        assert!(tokens.iter().any(|t| t.token == Token::Catch));
        assert!(tokens.iter().any(|t| t.token == Token::Error));
        assert!(tokens.iter().any(|t| t.token == Token::Context));
        assert!(tokens.iter().any(|t| t.token == Token::Throw));
    }

    #[test]
    fn test_tokenize_async_programming() {
        let source = "future yield stream";
        let file_id = FileId(0);
        let (tokens, _) = tokenize(source, file_id);
        assert!(tokens.iter().any(|t| t.token == Token::Future));
        assert!(tokens.iter().any(|t| t.token == Token::Yield));
        assert!(tokens.iter().any(|t| t.token == Token::Stream));
    }

    #[test]
    fn test_tokenize_chinese_async_programming() {
        let source = "未来 产出 流";
        let file_id = FileId(0);
        let (tokens, _) = tokenize(source, file_id);
        assert!(tokens.iter().any(|t| t.token == Token::Future));
        assert!(tokens.iter().any(|t| t.token == Token::Yield));
        assert!(tokens.iter().any(|t| t.token == Token::Stream));
    }
}
