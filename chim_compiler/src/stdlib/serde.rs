// ==================== 序列化/反序列化模块 ====================
// 支持 JSON、MessagePack 等格式的序列化与反序列化

pub mod serde {
    use crate::stdlib::prelude::{Option, Result, Vec, String, HashMap};
    use crate::stdlib::string::StringBuilder;

    // ==================== Value 类型 ====================

    #[derive(Debug, Clone, PartialEq)]
    pub enum Value {
        Null,
        Bool(bool),
        Int(int),
        Float(float),
        String(string),
        Array(Vec<Value>),
        Object(HashMap<string, Value>),
    }

    impl Value {
        pub fn null() -> Value {
            Value::Null
        }

        pub fn bool(b: bool) -> Value {
            Value::Bool(b)
        }

        pub fn int(n: int) -> Value {
            Value::Int(n)
        }

        pub fn float(f: float) -> Value {
            Value::Float(f)
        }

        pub fn string(s: string) -> Value {
            Value::String(s)
        }

        pub fn array() -> Vec<Value> {
            Vec::new()
        }

        pub fn object() -> HashMap<string, Value> {
            HashMap::new()
        }

        pub fn is_null(&self) -> bool {
            self == &Value::Null
        }

        pub fn as_bool(&self) -> Option<bool> {
            match self {
                Value::Bool(b) => Some(*b),
                _ => None,
            }
        }

        pub fn as_int(&self) -> Option<int> {
            match self {
                Value::Int(n) => Some(*n),
                Value::Float(f) => Some(*f as int),
                _ => None,
            }
        }

        pub fn as_float(&self) -> Option<float> {
            match self {
                Value::Float(f) => Some(*f),
                Value::Int(n) => Some(*n as float),
                _ => None,
            }
        }

        pub fn as_string(&self) -> Option<string> {
            match self {
                Value::String(s) => Some(s.clone()),
                _ => None,
            }
        }

        pub fn as_array(&self) -> Option<&Vec<Value>> {
            match self {
                Value::Array(a) => Some(a),
                _ => None,
            }
        }

        pub fn as_object(&self) -> Option<&HashMap<string, Value>> {
            match self {
                Value::Object(o) => Some(o),
                _ => None,
            }
        }

        pub fn get(&self, key: &string) -> Option<&Value> {
            match self {
                Value::Object(o) => o.get(key),
                _ => None,
            }
        }

        pub fn index(&self, idx: int) -> Option<&Value> {
            match self {
                Value::Array(a) => a.get(idx),
                _ => None,
            }
        }

        pub fn len(&self) -> int {
            match self {
                Value::Array(a) => a.len(),
                Value::Object(o) => o.len(),
                Value::String(s) => s.len(),
                _ => 0,
            }
        }
    }

    // ==================== JSON 序列化 ====================

    pub mod json {
        use super::{Value, Result};
        use crate::stdlib::prelude::{String, StringBuilder};

        pub fn to_string(value: &Value) -> string {
            let mut sb = StringBuilder::new();
            serialize_value(value, &mut sb);
            sb.to_string()
        }

        fn serialize_value(value: &Value, sb: &mut StringBuilder) {
            match value {
                Value::Null => sb.push_str("null"),
                Value::Bool(b) => {
                    if *b {
                        sb.push_str("true");
                    } else {
                        sb.push_str("false");
                    }
                }
                Value::Int(n) => sb.push_str(&n.to_string()),
                Value::Float(f) => sb.push_str(&f.to_string()),
                Value::String(s) => serialize_string(s, sb),
                Value::Array(arr) => {
                    sb.push_char('[');
                    let mut first = true;
                    for v in arr {
                        if !first {
                            sb.push_char(',');
                        }
                        serialize_value(v, sb);
                        first = false;
                    }
                    sb.push_char(']');
                }
                Value::Object(obj) => {
                    sb.push_char('{');
                    let mut first = true;
                    for (k, v) in obj {
                        if !first {
                            sb.push_char(',');
                        }
                        serialize_string(k, sb);
                        sb.push_char(':');
                        serialize_value(v, sb);
                        first = false;
                    }
                    sb.push_char('}');
                }
            }
        }

        fn serialize_string(s: &string, sb: &mut StringBuilder) {
            sb.push_char('"');
            let bytes = s.as_bytes();
            for &c in bytes {
                match c {
                    b'"' => sb.push_str("\\\""),
                    b'\\' => sb.push_str("\\\\"),
                    b'\n' => sb.push_str("\\n"),
                    b'\r' => sb.push_str("\\r"),
                    b'\t' => sb.push_str("\\t"),
                    _ => sb.push_char(c as char),
                }
            }
            sb.push_char('"');
        }

        pub fn from_string(s: &string) -> Result<Value> {
            let mut parser = JsonParser {
                input: s.clone(),
                pos: 0,
            };
            parser.parse()
        }

        struct JsonParser {
            input: string,
            pos: int,
        }

        impl JsonParser {
            fn parse(&mut self) -> Result<Value> {
                self.skip_whitespace();
                self.parse_value()
            }

            fn skip_whitespace(&mut self) {
                while self.pos < self.input.len() {
                    let c = self.input.as_bytes()[self.pos as usize] as char;
                    if c == ' ' || c == '\n' || c == '\r' || c == '\t' {
                        self.pos = self.pos + 1;
                    } else {
                        break;
                    }
                }
            }

            fn parse_value(&mut self) -> Result<Value> {
                if self.pos >= self.input.len() {
                    return Err(Error::new("unexpected end of input".to_string()));
                }
                
                let c = self.input.as_bytes()[self.pos as usize] as char;
                match c {
                    '"' => self.parse_string(),
                    't' => self.parse_true(),
                    'f' => self.parse_false(),
                    'n' => self.parse_null(),
                    '[' => self.parse_array(),
                    '{' => self.parse_object(),
                    _ => self.parse_number(),
                }
            }

            fn parse_string(&mut self) -> Result<Value> {
                self.pos = self.pos + 1;
                let mut sb = StringBuilder::new();
                
                while self.pos < self.input.len() {
                    let c = self.input.as_bytes()[self.pos as usize] as char;
                    if c == '"' {
                        self.pos = self.pos + 1;
                        return Ok(Value::String(sb.to_string()));
                    }
                    if c == '\\' && self.pos + 1 < self.input.len() {
                        self.pos = self.pos + 1;
                        let escaped = self.input.as_bytes()[self.pos as usize] as char;
                        match escaped {
                            '"' => sb.push_char('"'),
                            '\\' => sb.push_char('\\'),
                            'n' => sb.push_char('\n'),
                            'r' => sb.push_char('\r'),
                            't' => sb.push_char('\t'),
                            'u' => {
                                self.pos = self.pos + 1;
                                let hex = &self.input[self.pos as usize..self.pos as usize + 4];
                                let code = hex_to_int(hex);
                                sb.push_char(code as u8 as char);
                                self.pos = self.pos + 3;
                            }
                            _ => sb.push_char(escaped),
                        }
                    } else {
                        sb.push_char(c);
                        self.pos = self.pos + 1;
                    }
                }
                
                Err(Error::new("unclosed string".to_string()))
            }

            fn parse_true(&mut self) -> Result<Value> {
                if self.input[self.pos as usize..self.pos as usize + 4] == "true".to_string() {
                    self.pos = self.pos + 4;
                    Ok(Value::Bool(true))
                } else {
                    Err(Error::new("invalid token".to_string()))
                }
            }

            fn parse_false(&mut self) -> Result<Value> {
                if self.input[self.pos as usize..self.pos as usize + 5] == "false".to_string() {
                    self.pos = self.pos + 5;
                    Ok(Value::Bool(false))
                } else {
                    Err(Error::new("invalid token".to_string()))
                }
            }

            fn parse_null(&mut self) -> Result<Value> {
                if self.input[self.pos as usize..self.pos as usize + 4] == "null".to_string() {
                    self.pos = self.pos + 4;
                    Ok(Value::Null)
                } else {
                    Err(Error::new("invalid token".to_string()))
                }
            }

            fn parse_array(&mut self) -> Result<Value> {
                self.pos = self.pos + 1;
                self.skip_whitespace();
                let mut arr = Vec::new();
                
                if self.pos < self.input.len() {
                    let c = self.input.as_bytes()[self.pos as usize] as char;
                    if c != ']' {
                        loop {
                            self.skip_whitespace();
                            arr.push(self.parse_value()?);
                            self.skip_whitespace();
                            
                            if self.pos >= self.input.len() {
                                break;
                            }
                            let c = self.input.as_bytes()[self.pos as usize] as char;
                            if c == ',' {
                                self.pos = self.pos + 1;
                            } else if c == ']' {
                                self.pos = self.pos + 1;
                                break;
                            } else {
                                return Err(Error::new("expected , or ]".to_string()));
                            }
                        }
                    } else {
                        self.pos = self.pos + 1;
                    }
                }
                
                Ok(Value::Array(arr))
            }

            fn parse_object(&mut self) -> Result<Value> {
                self.pos = self.pos + 1;
                self.skip_whitespace();
                let mut obj = HashMap::new();
                
                if self.pos < self.input.len() {
                    let c = self.input.as_bytes()[self.pos as usize] as char;
                    if c != '}' {
                        loop {
                            self.skip_whitespace();
                            let key = match self.parse_value()? {
                                Value::String(s) => s,
                                _ => return Err(Error::new("expected string key".to_string())),
                            };
                            self.skip_whitespace();
                            
                            if self.pos >= self.input.len() || self.input.as_bytes()[self.pos as usize] as char != ':' {
                                return Err(Error::new("expected :".to_string()));
                            }
                            self.pos = self.pos + 1;
                            
                            self.skip_whitespace();
                            let value = self.parse_value()?;
                            obj.insert(key, value);
                            
                            self.skip_whitespace();
                            if self.pos >= self.input.len() {
                                break;
                            }
                            let c = self.input.as_bytes()[self.pos as usize] as char;
                            if c == ',' {
                                self.pos = self.pos + 1;
                            } else if c == '}' {
                                self.pos = self.pos + 1;
                                break;
                            } else {
                                return Err(Error::new("expected , or }".to_string()));
                            }
                        }
                    } else {
                        self.pos = self.pos + 1;
                    }
                }
                
                Ok(Value::Object(obj))
            }

            fn parse_number(&mut self) -> Result<Value> {
                let start = self.pos;
                let mut has_dot = false;
                let mut has_exp = false;
                
                if self.pos < self.input.len() {
                    let c = self.input.as_bytes()[self.pos as usize] as char;
                    if c == '-' {
                        self.pos = self.pos + 1;
                    }
                }
                
                while self.pos < self.input.len() {
                    let c = self.input.as_bytes()[self.pos as usize] as char;
                    if c >= '0' && c <= '9' {
                        self.pos = self.pos + 1;
                    } else if c == '.' && !has_dot {
                        has_dot = true;
                        self.pos = self.pos + 1;
                    } else if (c == 'e' || c == 'E') && !has_exp {
                        has_exp = true;
                        self.pos = self.pos + 1;
                        if self.pos < self.input.len() {
                            let c = self.input.as_bytes()[self.pos as usize] as char;
                            if c == '+' || c == '-' {
                                self.pos = self.pos + 1;
                            }
                        }
                    } else {
                        break;
                    }
                }
                
                let num_str = self.input[start as usize..self.pos as usize].to_string();
                
                if has_dot || has_exp {
                    match num_str.parse() {
                        Ok(f) => Ok(Value::Float(f)),
                        Err(_) => Err(Error::new("invalid number".to_string())),
                    }
                } else {
                    match num_str.parse() {
                        Ok(n) => Ok(Value::Int(n)),
                        Err(_) => Err(Error::new("invalid number".to_string())),
                    }
                }
            }
        }

        fn hex_to_int(hex: &string) -> int {
            let mut result = 0;
            let bytes = hex.as_bytes();
            for &c in bytes {
                let digit = if c >= b'0' && c <= b'9' {
                    (c - b'0') as int
                } else if c >= b'a' && c <= b'f' {
                    (c - b'a' + 10) as int
                } else if c >= b'A' && c <= b'F' {
                    (c - b'A' + 10) as int
                } else {
                    0
                };
                result = result * 16 + digit;
            }
            result
        }

        #[derive(Debug, Clone)]
        pub struct Error {
            message: string,
        }

        impl Error {
            pub fn new(msg: string) -> Error {
                Error { message: msg }
            }
        }

        pub type Result<T> = std::result::Result<T, Error>;
    }

    // ==================== MessagePack 序列化 ====================

    pub mod msgpack {
        use super::Value;
        use crate::stdlib::prelude::{Vec, String};

        pub fn to_vec(value: &Value) -> Vec<u8> {
            let mut bytes = Vec::new();
            serialize_value(value, &mut bytes);
            bytes
        }

        fn serialize_value(value: &Value, bytes: &mut Vec<u8>) {
            match value {
                Value::Null => bytes.push(0xC0),
                Value::Bool(b) => {
                    if *b {
                        bytes.push(0xC3);
                    } else {
                        bytes.push(0xC2);
                    }
                }
                Value::Int(n) => {
                    if *n >= 0 {
                        if *n <= 127 {
                            bytes.push(*n as u8);
                        } else if *n <= 255 {
                            bytes.push(0xCC);
                            bytes.push(*n as u8);
                        } else if *n <= 65535 {
                            bytes.push(0xCD);
                            bytes.push((*n >> 8) as u8);
                            bytes.push(*n as u8);
                        } else {
                            bytes.push(0xCE);
                            bytes.push((*n >> 24) as u8);
                            bytes.push((*n >> 16) as u8);
                            bytes.push((*n >> 8) as u8);
                            bytes.push(*n as u8);
                        }
                    } else {
                        bytes.push(0xD0);
                        bytes.push((*n & 0xFF) as u8);
                    }
                }
                Value::Float(f) => {
                    bytes.push(0xCB);
                    let bits = float_to_bits(*f);
                    bytes.push((bits >> 56) as u8);
                    bytes.push((bits >> 48) as u8);
                    bytes.push((bits >> 40) as u8);
                    bytes.push((bits >> 32) as u8);
                    bytes.push((bits >> 24) as u8);
                    bytes.push((bits >> 16) as u8);
                    bytes.push((bits >> 8) as u8);
                    bytes.push(bits as u8);
                }
                Value::String(s) => {
                    let len = s.len();
                    if len < 32 {
                        bytes.push((0xA0 | len as u8) as u8);
                    } else if len < 256 {
                        bytes.push(0xD9);
                        bytes.push(len as u8);
                    } else {
                        bytes.push(0xDA);
                        bytes.push((len >> 8) as u8);
                        bytes.push(len as u8);
                    }
                    for c in s.as_bytes() {
                        bytes.push(*c);
                    }
                }
                Value::Array(arr) => {
                    let len = arr.len();
                    if len < 16 {
                        bytes.push((0x90 | len as u8) as u8);
                    } else {
                        bytes.push(0xDC);
                        bytes.push((len >> 8) as u8);
                        bytes.push(len as u8);
                    }
                    for v in arr {
                        serialize_value(v, bytes);
                    }
                }
                Value::Object(obj) => {
                    let len = obj.len();
                    bytes.push(0xDF);
                    bytes.push((len >> 8) as u8);
                    bytes.push(len as u8);
                    for (k, v) in obj {
                        serialize_value(&Value::String(k.clone()), bytes);
                        serialize_value(v, bytes);
                    }
                }
            }
        }

        fn float_to_bits(f: float) -> u64 {
            let ptr = &f as *const float as *const u64;
            unsafe { *ptr }
        }

        pub fn from_vec(bytes: &[u8]) -> Result<Value> {
            let mut parser = MsgPackParser {
                input: bytes.to_vec(),
                pos: 0,
            };
            parser.parse()
        }

        struct MsgPackParser {
            input: Vec<u8>,
            pos: int,
        }

        impl MsgPackParser {
            fn parse(&mut self) -> Result<Value> {
                if self.pos >= self.input.len() {
                    return Err(Error::new("unexpected end".to_string()));
                }
                self.parse_value()
            }

            fn parse_value(&mut self) -> Result<Value> {
                if self.pos >= self.input.len() {
                    return Err(Error::new("unexpected end".to_string()));
                }
                
                let b = self.input[self.pos as usize];
                self.pos = self.pos + 1;
                
                match b {
                    0xC0 => Ok(Value::Null),
                    0xC2 => Ok(Value::Bool(false)),
                    0xC3 => Ok(Value::Bool(true)),
                    0xA0..=0xBF => {
                        let len = (b - 0xA0) as int;
                        self.parse_string(len)
                    }
                    0xD9 => {
                        let len = self.input[self.pos as usize] as int;
                        self.pos = self.pos + 1;
                        self.parse_string(len)
                    }
                    0xDA => {
                        let len = ((self.input[self.pos as usize] as int) << 8) | self.input[self.pos as usize + 1] as int;
                        self.pos = self.pos + 2;
                        self.parse_string(len)
                    }
                    0x90..=0x9F => {
                        let len = (b - 0x90) as int;
                        self.parse_array(len)
                    }
                    0xDC => {
                        let len = ((self.input[self.pos as usize] as int) << 8) | self.input[self.pos as usize + 1] as int;
                        self.pos = self.pos + 2;
                        self.parse_array(len)
                    }
                    0xDF => {
                        let len = ((self.input[self.pos as usize] as int) << 8) | self.input[self.pos as usize + 1] as int;
                        self.pos = self.pos + 2;
                        self.parse_object(len)
                    }
                    0xCB => self.parse_float(),
                    0xD0 => self.parse_int8(),
                    _ if b <= 0x7F => Ok(Value::Int(b as int)),
                    _ => Ok(Value::Null),
                }
            }

            fn parse_string(&mut self, len: int) -> Result<Value> {
                let end = self.pos + len;
                if end > self.input.len() as int {
                    return Err(Error::new("string too long".to_string()));
                }
                let s = String::from_utf8_lossy(&self.input[self.pos as usize..end as usize]).to_string();
                self.pos = end;
                Ok(Value::String(s))
            }

            fn parse_array(&mut self, len: int) -> Result<Value> {
                let mut arr = Vec::new();
                for _ in 0..len {
                    arr.push(self.parse_value()?);
                }
                Ok(Value::Array(arr))
            }

            fn parse_object(&mut self, len: int) -> Result<Value> {
                let mut obj = HashMap::new();
                for _ in 0..len {
                    let key = match self.parse_value()? {
                        Value::String(s) => s,
                        _ => return Err(Error::new("object key must be string".to_string())),
                    };
                    let value = self.parse_value()?;
                    obj.insert(key, value);
                }
                Ok(Value::Object(obj))
            }

            fn parse_float(&mut self) -> Result<Value> {
                if self.pos + 8 > self.input.len() as int {
                    return Err(Error::new("float too short".to_string()));
                }
                let mut bits: u64 = 0;
                for i in 0..8 {
                    bits = (bits << 8) | (self.input[self.pos as usize + i] as u64);
                }
                self.pos = self.pos + 8;
                let f = bits_to_float(bits);
                Ok(Value::Float(f))
            }

            fn parse_int8(&mut self) -> Result<Value> {
                if self.pos >= self.input.len() as int {
                    return Err(Error::new("int8 too short".to_string()));
                }
                let n = self.input[self.pos as usize] as i8;
                self.pos = self.pos + 1;
                Ok(Value::Int(n as int))
            }
        }

        fn bits_to_float(bits: u64) -> float {
            let ptr = &bits as *const u64 as *const float;
            unsafe { *ptr }
        }

        #[derive(Debug, Clone)]
        pub struct Error {
            message: string,
        }

        impl Error {
            pub fn new(msg: string) -> Error {
                Error { message: msg }
            }
        }

        pub type Result<T> = std::result::Result<T, Error>;
    }

    // ==================== 错误类型 ====================

    #[derive(Debug, Clone)]
    pub struct Error {
        message: string,
    }

    impl Error {
        pub fn new(msg: string) -> Error {
            Error { message: msg }
        }
    }

    pub type Result<T> = std::result::Result<T, Error>;
}
