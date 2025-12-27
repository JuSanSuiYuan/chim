use logos::Logos;
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Clone)]
pub enum LexerError {
    #[error("Unexpected token at position {pos}: {char}")]
    UnexpectedToken {
        pos: usize,
        char: char,
    },
    #[error("Unterminated string literal")]
    UnterminatedString,
    #[error("Invalid number format")]
    InvalidNumber,
}

#[derive(Logos, Debug, PartialEq, Clone)]
pub enum Token {
    // 关键字
    #[token("fn")] Fn,
    #[token("let")] Let,
    #[token("var")] Var,
    #[token("if")] If,
    #[token("elif")] Elif,
    #[token("else")] Else,
    #[token("match")] Match,
    #[token("while")] While,
    #[token("for")] For,
    #[token("in")] In,
    #[token("return")] Return,
    #[token("struct")] Struct,
    #[token("enum")] Enum,
    #[token("group")] Group,
    #[token("init")] Init,
    #[token("cleanup")] Cleanup,
    #[token("safe")] Safe,
    #[token("unsafe")] Unsafe,
    #[token("true")] True,
    #[token("false")] False,
    #[token("null")] Null,
    #[token("import")] Import,
    #[token("as")] As,
    #[token("mut")] Mut,
    #[token("ref")] Ref,
    #[token("Ok")] Ok,
    #[token("Error")] ErrorKeyword,
    #[token("Some")] Some,
    #[token("None")] None,
    #[token("Box")] Box,
    #[token("Rc")] Rc,
    #[token("Arc")] Arc,
    #[token("Vec")] Vec,
    #[token("HashMap")] HashMap,
    #[token("List")] List,
    #[token("Result")] Result,
    #[token("Option")] Option,
    #[token("Container")] Container,
    #[token("Unit")] Unit,
    #[token("@kernel")] KernelAttr,
    
    // 中文关键字
    #[token("令")] LetZh,
    #[token("设")] VarZh,
    #[token("如果")] IfZh,
    #[token("否则")] ElseZh,
    #[token("匹配")] MatchZh,
    #[token("当")] WhileZh,
    #[token("循环")] LoopZh,
    #[token("返回")] ReturnZh,
    #[token("结构体")] StructZh,
    #[token("枚举")] EnumZh,
    #[token("组")] GroupZh,
    #[token("真")] TrueZh,
    #[token("假")] FalseZh,
    #[token("引用")] RefZh,
    
    // 输出函数关键字
    #[token("print")] Print,
    #[token("println")] Println,
    #[token("打印")] PrintZh,
    #[token("打印行")] PrintlnZh,
    
    // 字面量
    // 物理单位字面量 - 必须放在数字字面量之前，因为它是更具体的模式
    #[regex(r"(\d+(?:\.\d+)?)([a-zA-Z]+)", |lex| {
        let slice = lex.slice();
        let (num_str, unit_str) = slice.split_at(slice.find(|c: char| c.is_alphabetic()).unwrap());
        Some(format!("({}, {})", num_str, unit_str))
    })] UnitLiteral(String),
    #[regex(r"\d+\.\d+", |lex| lex.slice().parse().ok())] Float(f64),
    #[regex(r"\d+", |lex| lex.slice().parse().ok())] Integer(i64),
    // 字符串字面量处理 - 支持任意字符和转义序列
    #[regex(r#"("(\\.|[^"\\])*")"#, |lex| {
        let slice = lex.slice();
        // 移除首尾引号
        let content = &slice[1..slice.len()-1];
        let mut result = String::new();
        let mut chars = content.chars().peekable();
        
        while let Some(c) = chars.next() {
            if c == '\\' {
                if let Some(next) = chars.next() {
                    match next {
                        '"' => result.push('"'),
                        '\\' => result.push('\\'),
                        'n' => result.push('\n'),
                        't' => result.push('\t'),
                        'r' => result.push('\r'),
                        _ => {
                            result.push('\\');
                            result.push(next);
                        }
                    }
                }
            } else {
                result.push(c);
            }
        }
        
        Some(result)
    })] String(String),
    
    // 标识符
    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*", |lex| Some(lex.slice().to_string()))] Identifier(String),
    
    // 操作符
    #[token("+")] Plus,
    #[token("-")] Minus,
    #[token("*")] Star,
    #[token("/")] Slash,
    #[token("%")] Percent,
    #[token("&")] Ampersand,
    #[token("&mut")] AndMut,
    #[token("'")] Lifetime,
    
    #[token("==")] Eq,
    #[token("!=")] Ne,
    #[token("<")] Lt,
    #[token("<=")] Le,
    #[token(">=")] Ge,
    #[token(">")] Gt,
    
    #[token("&&")] And,
    #[token("||")] Or,
    #[token("!")] Not,
    
    #[token("=")] Assign,
    #[token("+=")] AddAssign,
    #[token("-=")] SubAssign,
    #[token("*=")] MulAssign,
    #[token("/=")] DivAssign,
    #[token("%=")] ModAssign,
    
    #[token("->")] Arrow,
    #[token("=>")] FatArrow,
    
    #[token("..")] Range,
    #[token("..=")] RangeInclusive,
    
    // 标点符号
    #[token(".")] Dot,
    #[token(",")] Comma,
    #[token(":")] Colon,
    #[token(";")] Semicolon,
    #[token("(")] LParen,
    #[token(")")] RParen,
    #[token("[")] LBracket,
    #[token("]")] RBracket,
    #[token("{")] LBrace,
    #[token("}")] RBrace,
    
    // 注释和空格
    #[regex(r"//.*", logos::skip)]
    Comment,
    
    #[regex(r"/\*(?:[^*]|\*[^/])*\*/", logos::skip)]
    MultiLineComment,
    
    #[regex(r"[\s\r\n]+", logos::skip)]
    Whitespace,
    
    // 未知标记
    Error(LexerError),
}
