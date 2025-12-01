use logos::Logos;
use std::fmt;

/// Chim语言的词法标记类型
#[derive(Logos, Debug, PartialEq, Clone)]
#[logos(skip r"[ \t\r\n]+")]
#[logos(skip r"//.*")]
#[logos(skip r"#.*")]
pub enum Token {
    // 关键字 - 中文/英文双轨支持
    #[token("函数")] #[token("fn")] Function,
    #[token("令")] #[token("let")] Let,
    #[token("设")] #[token("set")] Set,
    #[token("可变")] #[token("mut")] Mut,
    #[token("输出")] #[token("print")] Print,
    #[token("返回")] #[token("return")] Return,
    #[token("导入")] #[token("import")] Import,
    #[token("导出")] #[token("export")] Export,
    #[token("公共")] #[token("pub")] Pub,
    #[token("结构体")] #[token("struct")] Struct,
    #[token("枚举")] #[token("enum")] Enum,
    #[token("类")] #[token("class")] Class,
    #[token("构造函数")] Constructor,
    #[token("方法")] Method,
    #[token("实现")] Impl,
    #[token("特性")] #[token("trait")] Trait,
    #[token("匹配")] #[token("match")] Match,
    #[token("案例")] #[token("case")] Case,
    #[token("默认")] Default,
    #[token("如果")] #[token("if")] If,
    #[token("若")] Elif2,  // 若作为if的简化版
    #[token("否则如果")] #[token("elif")] Elif,
    #[token("否则")] #[token("else")] Else,
    #[token("对于")] #[token("for")] For,
    #[token("在")] #[token("in")] In,
    #[token("循环")] #[token("loop")] Loop,
    #[token("当")] #[token("while")] While,
    #[token("跳出")] #[token("break")] Break,
    #[token("继续")] #[token("continue")] Continue,
    #[token("空")] #[token("null")] Null,
    #[token("真")] #[token("true")] True,
    #[token("假")] #[token("false")] False,
    #[token("内联汇编")] InlineAsm,
    #[token("不安全")] #[token("unsafe")] Unsafe,
    #[token("常量")] #[token("const")] Const,
    #[token("静态")] #[token("static")] Static,
    #[token("类型别名")] #[token("type")] TypeAlias,
    #[token("尝试")] #[token("try")] Try,
    #[token("捕获")] #[token("catch")] Catch,
    #[token("通道")] #[token("chan")] Chan,
    #[token("协程")] #[token("async")] Async,
    #[token("等待")] #[token("await")] Await,
    #[token("兼容")] #[token("compat")] Compat,
    #[token("安全")] #[token("safe")] Safe,
    #[token("系统")] #[token("system")] System,
    #[token("动态")] #[token("dynamic")] Dynamic,
    #[token("DSL")] DSL,
    #[token("toml")] TOML,
    
    // 组生命周期相关
    #[token("组")] #[token("group")] Group,
    #[token("快照")] #[token("snapshot")] Snapshot,
    #[token("句柄")] #[token("handle")] Handle,
    #[token("arena")] Arena,
    
    // 运算符
    #[token("=")] Assign,
    #[token(":=")] ColonAssign,
    #[token("+")] Plus,
    #[token("-")] Minus,
    #[token("*")] Multiply,
    #[token("/")] Divide,
    #[token("%")] Modulo,
    #[token("==")] Equal,
    #[token("!=")] NotEqual,
    #[token(">=")] GreaterOrEqual,
    #[token("<=")] LessOrEqual,
    #[token(">")] Greater,
    #[token("<")] Less,
    #[token("&&")] And,
    #[token("||")] Or,
    #[token("!")] Not,
    #[token("&" , priority = 2)] Ampersand,
    #[token("|", priority = 2)] Pipe,
    #[token("^" , priority = 2)] Caret,
    #[token("<<")] LeftShift,
    #[token(">>")] RightShift,
    #[token("+=")] AddAssign,
    #[token("-=")] SubtractAssign,
    #[token("*=")] MultiplyAssign,
    #[token("/=")] DivideAssign,
    #[token("%=")] RemainderAssign,
    #[token("&=")] AndAssign,
    #[token("|=")] OrAssign,
    #[token("^=")] XorAssign,
    #[token("<<=")] LeftShiftAssign,
    #[token(">>=")] RightShiftAssign,
    #[token("->")] Arrow,
    #[token("<-")] DirectionArrow,
    #[token("...")] Ellipsis,
    #[token(".")] Dot,
    #[token(",")] Comma,
    #[token(":")] Colon,
    #[token("::")] DoubleColon,
    #[token(";")] Semicolon,
    #[token("(")] LeftParen,
    #[token(")")] RightParen,
    #[token("[")] LeftBracket,
    #[token("]")] RightBracket,
    #[token("{")] LeftBrace,
    #[token("}")] RightBrace,
    #[token("|", priority = 3)] MatchPipe,
    
    // 字面量
    #[regex(r"[0-9]+")] Integer,
    #[regex(r"[0-9]+\.[0-9]+")] Float,
    #[regex(r#"\"(\\.|[^\"\\])*\""#)] String,
    #[regex(r"'(\\.|[^'\\])'")]Char,
    
    // 标识符
    #[regex(r"[a-zA-Z_\u{4e00}-\u{9fa5}][a-zA-Z0-9_\u{4e00}-\u{9fa5}]*")] Identifier,
    
    // 特殊标记（内部使用，用于缩进处理）
    INDENT,
    DEDENT,
    NEWLINE,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Token::Function => write!(f, "function"),
            Token::Let => write!(f, "let"),
            Token::Set => write!(f, "set"),
            Token::Mut => write!(f, "mut"),
            Token::Print => write!(f, "print"),
            Token::Return => write!(f, "return"),
            Token::Import => write!(f, "import"),
            Token::Export => write!(f, "export"),
            Token::Pub => write!(f, "pub"),
            Token::Struct => write!(f, "struct"),
            Token::Enum => write!(f, "enum"),
            Token::Class => write!(f, "class"),
            Token::Constructor => write!(f, "constructor"),
            Token::Method => write!(f, "method"),
            Token::Impl => write!(f, "impl"),
            Token::Trait => write!(f, "trait"),
            Token::Match => write!(f, "match"),
            Token::Case => write!(f, "case"),
            Token::Default => write!(f, "default"),
            Token::If => write!(f, "if"),
            Token::Elif2 => write!(f, "if"),  // 若也显示为if
            Token::Elif => write!(f, "elif"),
            Token::Else => write!(f, "else"),
            Token::For => write!(f, "for"),
            Token::In => write!(f, "in"),
            Token::Loop => write!(f, "loop"),
            Token::While => write!(f, "while"),
            Token::Break => write!(f, "break"),
            Token::Continue => write!(f, "continue"),
            Token::Null => write!(f, "null"),
            Token::True => write!(f, "true"),
            Token::False => write!(f, "false"),
            Token::InlineAsm => write!(f, "inline_asm"),
            Token::Unsafe => write!(f, "unsafe"),
            Token::Const => write!(f, "const"),
            Token::Static => write!(f, "static"),
            Token::TypeAlias => write!(f, "type"),
            Token::Try => write!(f, "try"),
            Token::Catch => write!(f, "catch"),
            Token::Chan => write!(f, "chan"),
            Token::Async => write!(f, "async"),
            Token::Await => write!(f, "await"),
            Token::Compat => write!(f, "compat"),
            Token::Safe => write!(f, "safe"),
            Token::System => write!(f, "system"),
            Token::Dynamic => write!(f, "dynamic"),
            Token::DSL => write!(f, "dsl"),
            Token::TOML => write!(f, "toml"),
            
            // 组生命周期
            Token::Group => write!(f, "group"),
            Token::Snapshot => write!(f, "snapshot"),
            Token::Handle => write!(f, "handle"),
            Token::Arena => write!(f, "arena"),
            
            // 运算符
            Token::Assign => write!(f, "="),
            Token::ColonAssign => write!(f, ":="),
            Token::Plus => write!(f, "+"),
            Token::Minus => write!(f, "-"),
            Token::Multiply => write!(f, "*"),
            Token::Divide => write!(f, "/"),
            Token::Modulo => write!(f, "%"),

            Token::Equal => write!(f, "=="),
            Token::NotEqual => write!(f, "!="),
            Token::GreaterOrEqual => write!(f, ">="),
            Token::LessOrEqual => write!(f, "<="),
            Token::Greater => write!(f, ">"),
            Token::Less => write!(f, "<"),
            Token::And => write!(f, "&&"),
            Token::Or => write!(f, "||"),
            Token::Not => write!(f, "!"),
            Token::Ampersand => write!(f, "&"),
            Token::Pipe => write!(f, "|"),
            Token::Caret => write!(f, "^"),
            Token::LeftShift => write!(f, "<<"),
            Token::RightShift => write!(f, ">>"),
            Token::AddAssign => write!(f, "+="),
            Token::SubtractAssign => write!(f, "-="),
            Token::MultiplyAssign => write!(f, "*="),
            Token::DivideAssign => write!(f, "/="),
            Token::RemainderAssign => write!(f, "%="),
            Token::AndAssign => write!(f, "&="),
            Token::OrAssign => write!(f, "|="),
            Token::XorAssign => write!(f, "^="),
            Token::LeftShiftAssign => write!(f, "<<="),
            Token::RightShiftAssign => write!(f, ">>="),
            Token::Arrow => write!(f, "->"),
            Token::DirectionArrow => write!(f, "<-") ,
            Token::Ellipsis => write!(f, "..."),
            Token::Dot => write!(f, "."),
            Token::Comma => write!(f, ","),
            Token::Colon => write!(f, ":"),
            Token::DoubleColon => write!(f, "::"),
            Token::Semicolon => write!(f, ";"),
            Token::LeftParen => write!(f, "("),
            Token::RightParen => write!(f, ")"),
            Token::LeftBracket => write!(f, "["),
            Token::RightBracket => write!(f, "]"),
            Token::LeftBrace => write!(f, "{{"),
            Token::RightBrace => write!(f, "}}") ,
            Token::MatchPipe => write!(f, "|") ,
            
            // 字面量
            Token::Integer => write!(f, "integer"),
            Token::Float => write!(f, "float"),
            Token::String => write!(f, "string"),
            Token::Char => write!(f, "char"),
            
            // 标识符
            Token::Identifier => write!(f, "identifier"),
            
            // 特殊标记
            Token::INDENT => write!(f, "INDENT"),
            Token::DEDENT => write!(f, "DEDENT"),
            Token::NEWLINE => write!(f, "NEWLINE"),
        }
    }
}

/// 带有位置信息的词法标记
#[derive(Debug, Clone)]
pub struct SpannedToken {
    pub token: Token,
    pub span: logos::Span,
    pub text: String,
    pub line: usize,
    pub column: usize,
}

/// 词法分析器
pub struct Lexer {
    source: String,
    tokens: Vec<SpannedToken>,
    current: usize,
    line: usize,
    column: usize,
    indent_level: usize,
    indent_stack: Vec<usize>,
}

impl Lexer {
    pub fn new(source: &str) -> Self {
        Lexer {
            source: source.to_string(),
            tokens: Vec::new(),
            current: 0,
            line: 1,
            column: 1,
            indent_level: 0,
            indent_stack: vec![0],
        }
    }
    
    /// 执行词法分析并返回标记流
    pub fn lex_all(&mut self) -> Vec<SpannedToken> {
        match self.lex() {
            Ok(tokens) => tokens,
            Err(_) => Vec::new(),
        }
    }
    
    /// 执行词法分析并返回标记流
    pub fn lex(&mut self) -> Result<Vec<SpannedToken>, String> {
        let source_clone = self.source.clone();
        let mut logos_lexer = Token::lexer(&source_clone);
        self.line = 1;
        self.column = 1;
        
        let mut last_end = 0;
        
        while let Some(tok_result) = logos_lexer.next() {
            let span = logos_lexer.span();
            let text_slice = logos_lexer.slice();
            
            let token = match tok_result {
                Ok(t) => t,
                Err(_) => continue,
            };
            
            // 处理跳过的空白（换行和缩进）
            if span.start > last_end {
                let skipped = &source_clone[last_end..span.start];
                self.process_whitespace(skipped);
            }
            
            last_end = span.end;
            
            // 添加token
            self.tokens.push(SpannedToken {
                token,
                span,
                text: text_slice.to_string(),
                line: self.line,
                column: self.column,
            });
            
            self.column += text_slice.len();
        }
        
        // 处理文件末尾
        if last_end < source_clone.len() {
            let skipped = &source_clone[last_end..];
            self.process_whitespace(skipped);
        }
        
        // 关闭所有缩进
        while self.indent_stack.len() > 1 {
            self.indent_stack.pop();
            self.indent_level = *self.indent_stack.last().unwrap();
            self.tokens.push(SpannedToken {
                token: Token::DEDENT,
                span: logos::Span::default(),
                text: "".to_string(),
                line: self.line,
                column: 1,
            });
        }
        
        Ok(self.tokens.clone())
    }
    
    /// 处理空白字符（换行和缩进）
    fn process_whitespace(&mut self, text: &str) {
        for ch in text.chars() {
            match ch {
                '\n' => {
                    self.tokens.push(SpannedToken {
                        token: Token::NEWLINE,
                        span: logos::Span::default(),
                        text: "\n".to_string(),
                        line: self.line,
                        column: self.column,
                    });
                    self.line += 1;
                    self.column = 1;
                },
                ' ' | '\t' | '\r' => {
                    self.column += 1;
                },
                _ => {}
            }
        }
    }
    
    /// 处理换行和缩进
    fn process_newlines_and_indents(&mut self, text: &str, raw_tokens: &mut Vec<(Token, logos::Span, String)>) {
        let mut lines = text.split('\n');
        
        // 处理第一行的剩余部分（可能包含空格）
        if let Some(first_line) = lines.next() {
            if !first_line.is_empty() {
                // 只处理空格，忽略其他字符（应该被logos处理）
                if first_line.chars().all(|c| c == ' ') {
                    // 不产生标记，只更新列位置
                    self.column += first_line.len();
                }
            }
        }
        
        // 处理换行和缩进
        for (i, line) in lines.enumerate() {
            if i > 0 || !text.starts_with('\n') {
                // 添加换行标记
                self.tokens.push(SpannedToken {
                    token: Token::NEWLINE,
                    span: logos::Span::default(),
                    text: "\n".to_string(),
                    line: self.line,
                    column: self.column,
                });
                
                self.line += 1;
                self.column = 1;
            }
            
            // 计算缩进级别（假设使用空格）
            let indent = line.chars().take_while(|&c| c == ' ').count();
            self.column += indent;
            
            // 处理缩进变化
            if indent > self.indent_level {
                // 增加缩进
                self.indent_stack.push(indent);
                self.indent_level = indent;
                self.tokens.push(SpannedToken {
                    token: Token::INDENT,
                    span: logos::Span::default(),
                    text: " ".repeat(indent),
                    line: self.line,
                    column: 1,
                });
            } else if indent < self.indent_level {
                // 减少缩进
                while let Some(last_indent) = self.indent_stack.last() {
                    if *last_indent > indent {
                        self.indent_stack.pop();
                        self.indent_level = *self.indent_stack.last().unwrap();
                        self.tokens.push(SpannedToken {
                            token: Token::DEDENT,
                            span: logos::Span::default(),
                            text: "".to_string(),
                            line: self.line,
                            column: 1,
                        });
                    } else {
                        break;
                    }
                }
            }
        }
    }
}

/// 测试词法分析器
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_basic_tokens() {
        let source = "fn main():\n    let x = 1\n    return x";
        let mut lexer = Lexer::new(source);
        let tokens = lexer.lex().unwrap();
        
        assert!(!tokens.is_empty());
        assert!(tokens.iter().any(|t| t.token == Token::Function));
        assert!(tokens.iter().any(|t| t.token == Token::Identifier && t.text == "main"));
    }
}