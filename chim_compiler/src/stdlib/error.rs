// ==================== 错误处理标准库 ====================
// 错误类型、Result、panic 处理等功能

// ==================== 基础错误 trait ====================
pub trait Error: Display + Debug {
    fn source(&self) -> Option<&Error> {
        Option::None
    }
    
    fn description(&self) -> string {
        self.to_string()
    }
    
    fn cause(&self) -> Option<&Error> {
        self.source()
    }
}

pub trait Display {
    fn to_string(&self) -> string;
}

pub trait Debug {
    fn fmt(&self, f: &mut Formatter) -> Result;
}

// ==================== 常见错误类型 ====================
pub enum ErrorKind {
    NotFound,
    PermissionDenied,
    ConnectionRefused,
    ConnectionReset,
    HostUnreachable,
    NetworkUnreachable,
    AddrInUse,
    AddrNotAvailable,
    BrokenPipe,
    WouldBlock,
    InvalidInput,
    InvalidData,
    TimedOut,
    WriteZero,
    UnexpectedEof,
    FileTooLarge,
    StorageFull,
    NotImplemented,
    Other,
}

pub struct IOError {
    kind: ErrorKind,
    message: string,
    detail: Option<string>,
}

impl IOError {
    pub fn new(kind: ErrorKind, message: string) -> IOError {
        IOError { kind, message, detail: Option::None }
    }
    
    pub fn with_detail(kind: ErrorKind, message: string, detail: string) -> IOError {
        IOError { kind, message, detail: Option::Some(detail) }
    }
    
    pub fn kind(&self) -> ErrorKind {
        self.kind
    }
    
    pub fn message(&self) -> string {
        self.message
    }
    
    pub fn detail(&self) -> Option<&string> {
        self.detail.as_ref()
    }
    
    pub fn is_not_found(&self) -> bool {
        self.kind == ErrorKind::NotFound
    }
    
    pub fn is_permission_denied(&self) -> bool {
        self.kind == ErrorKind::PermissionDenied
    }
    
    pub fn is_eof(&self) -> bool {
        self.kind == ErrorKind::UnexpectedEof
    }
    
    pub fn is_timeout(&self) -> bool {
        self.kind == ErrorKind::TimedOut
    }
}

impl Display for IOError {
    fn to_string(&self) -> string {
        match &self.detail {
            Some(d) => format!("{}: {} ({})", self.message, d, self.kind_name()),
            None => format!("{} ({})", self.message, self.kind_name()),
        }
    }
}

impl Error for IOError {
    fn description(&self) -> string {
        self.to_string()
    }
    
    fn source(&self) -> Option<&Error> {
        Option::None
    }
}

impl IOError {
    fn kind_name(&self) -> string {
        match self.kind {
            ErrorKind::NotFound => "NotFound",
            ErrorKind::PermissionDenied => "PermissionDenied",
            ErrorKind::ConnectionRefused => "ConnectionRefused",
            ErrorKind::ConnectionReset => "ConnectionReset",
            ErrorKind::HostUnreachable => "HostUnreachable",
            ErrorKind::NetworkUnreachable => "NetworkUnreachable",
            ErrorKind::AddrInUse => "AddrInUse",
            ErrorKind::AddrNotAvailable => "AddrNotAvailable",
            ErrorKind::BrokenPipe => "BrokenPipe",
            ErrorKind::WouldBlock => "WouldBlock",
            ErrorKind::InvalidInput => "InvalidInput",
            ErrorKind::InvalidData => "InvalidData",
            ErrorKind::TimedOut => "TimedOut",
            ErrorKind::WriteZero => "WriteZero",
            ErrorKind::UnexpectedEof => "UnexpectedEof",
            ErrorKind::FileTooLarge => "FileTooLarge",
            ErrorKind::StorageFull => "StorageFull",
            ErrorKind::NotImplemented => "NotImplemented",
            ErrorKind::Other => "Other",
        }.to_string()
    }
}

// ==================== 解析错误 ====================
pub struct ParseError {
    input: string,
    position: int,
    expected: string,
    reason: Option<string>,
}

impl ParseError {
    pub fn new(input: string, position: int, expected: string) -> ParseError {
        ParseError { input, position, expected, reason: Option::None }
    }
    
    pub fn with_reason(input: string, position: int, expected: string, reason: string) -> ParseError {
        ParseError { input, position, expected, reason: Option::Some(reason) }
    }
    
    pub fn input(&self) -> string { self.input }
    pub fn position(&self) -> int { self.position }
    pub fn expected(&self) -> string { self.expected }
    pub fn reason(&self) -> Option<&string> { self.reason.as_ref() }
    
    pub fn to_string(&self) -> string {
        let context = if self.position > 0 && self.position < string_len(self.input) {
            let start = if self.position > 10 { self.position - 10 } else { 0 };
            let end = min(self.position + 10, string_len(self.input));
            format!("...{}|{}|{}...", 
                string_substr(self.input, start, self.position - start),
                string_substr(self.input, self.position, 1),
                string_substr(self.input, self.position + 1, end - self.position - 1))
        } else {
            "".to_string()
        };
        
        match &self.reason {
            Some(r) => format!("Parse error at position {}: expected {}, {}", self.position, self.expected, r),
            None => format!("Parse error at position {}: expected {}", self.position, self.expected),
        }
    }
}

impl Error for ParseError {
    fn description(&self) -> string {
        self.to_string()
    }
}

// ==================== panic 处理 ====================
pub fn panic<T>(message: string) -> T {
    __panic(message);
}

pub fn panic_with<T>(message: string, file: string, line: int) -> T {
    __panic_with_location(message, file, line);
}

pub fn assert(condition: bool, message: string) {
    if !condition {
        panic("Assertion failed: " + message);
    }
}

pub fn assert_eq<T>(a: T, b: T, message: string) where T: Eq {
    if a != b {
        panic("Assertion failed: " + message + " (values not equal)");
    }
}

pub fn assert_ne<T>(a: T, b: T, message: string) where T: Eq {
    if a == b {
        panic("Assertion failed: " + message + " (values equal)");
    }
}

pub fn debug_assert(condition: bool, message: string) {
    if !condition {
        __debug_panic("Debug assertion failed: " + message);
    }
}

pub fn unreachable<T>(message: string) -> T {
    panic("Unreachable code reached: " + message);
}

pub fn unimplemented<T>(message: string) -> T {
    panic("Unimplemented: " + message);
}

// ==================== 回溯信息 ====================
pub struct Backtrace {
    frames: [BacktraceFrame],
}

pub struct BacktraceFrame {
    function: string,
    file: string,
    line: int,
    col: int,
}

impl Backtrace {
    pub fn capture() -> Backtrace {
        Backtrace { frames: __backtrace_capture() }
    }
    
    pub fn frames(&self) -> &[BacktraceFrame] {
        &self.frames
    }
    
    pub fn count(&self) -> int {
        array_len(&self.frames)
    }
}

impl BacktraceFrame {
    pub fn function(&self) -> string { self.function }
    pub fn file(&self) -> string { self.file }
    pub fn line(&self) -> int { self.line }
    pub fn col(&self) -> int { self.col }
}

// ==================== 错误链 ====================
pub struct ErrorChain {
    current: Option<Box<dyn Error>>,
}

impl ErrorChain {
    pub fn new(error: &dyn Error) -> ErrorChain {
        ErrorChain { current: Option::Some(Box::new(error.clone())) }
    }
    
    pub fn next(&mut self) -> Option<&dyn Error> {
        match &mut self.current {
            Some(e) => {
                let next = e.source();
                if next.is_none() {
                    self.current = Option::None;
                } else {
                    self.current = Some(Box::new(next.unwrap().clone()));
                }
                self.current.as_deref()
            }
            None => Option::None,
        }
    }
    
    pub fn collect(&self) -> Vec<string> {
        let mut errors = Vec::new();
        let mut current: Option<&dyn Error> = Option::Some(self.current.as_ref().unwrap());
        while let Some(e) = current {
            errors.push(e.to_string());
            current = e.source();
        }
        errors
    }
}

// ==================== 格式化器 ====================
pub struct Formatter {
    output: string,
}

impl Formatter {
    pub fn new() -> Formatter {
        Formatter { output: "".to_string() }
    }
    
    pub fn write_str(&mut self, s: string) {
        self.output = self.output + s;
    }
    
    pub fn write_int(&mut self, n: int) {
        self.output = self.output + n.to_string();
    }
    
    pub fn write_float(&mut self, f: float) {
        self.output = self.output + f.to_string();
    }
    
    pub fn write_bool(&mut self, b: bool) {
        self.output = self.output + if b { "true" } else { "false" };
    }
    
    pub fn write_char(&mut self, c: string) {
        self.output = self.output + c;
    }
    
    pub fn write_fmt(&mut self, fmt: string, args: &[any]) {
        self.output = self.output + __format(fmt, args);
    }
    
    pub fn result(&self) -> string {
        self.output.clone()
    }
}

// ==================== 错误上下文 ====================
pub struct Context<T> {
    value: T,
    context: string,
}

impl<T> Context<T> {
    pub fn new(value: T, context: string) -> Context<T> {
        Context { value, context }
    }
    
    pub fn with<U>(self, f: fn(T) -> U, context: string) -> Context<U> {
        Context::new(f(self.value), context + ": " + self.context)
    }
    
    pub fn value(self) -> T {
        self.value
    }
    
    pub fn context(&self) -> string {
        self.context.clone()
    }
}

impl<T> Context<Option<T>> {
    pub fn ok_or<E>(self, error: E) -> Result<T, E> where E: Error {
        match self.value {
            Some(v) => Result::Ok(v),
            None => Result::Err(error),
        }
    }
    
    pub fn ok_or_else<E>(self, f: fn() -> E) -> Result<T, E> where E: Error {
        match self.value {
            Some(v) => Result::Ok(v),
            None => Result::Err(f()),
        }
    }
}

impl<T, E> Context<Result<T, E>> {
    pub fn context(self, ctx: string) -> Result<T, E> {
        match self.value {
            Ok(v) => Ok(v),
            Err(e) => Err(e),
        }
    }
}
