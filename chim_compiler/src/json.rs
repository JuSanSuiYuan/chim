use std::collections::HashMap;
use std::io::{self, Read, Write};

pub type JsonResult<T> = Result<T, JsonError>;

#[derive(Debug, Clone)]
pub enum JsonError {
    ParseError(String),
    WriteError(String),
    InvalidFormat(String),
    MissingKey(String),
    InvalidValue(String),
}

impl std::fmt::Display for JsonError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JsonError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            JsonError::WriteError(msg) => write!(f, "Write error: {}", msg),
            JsonError::InvalidFormat(msg) => write!(f, "Invalid format: {}", msg),
            JsonError::MissingKey(key) => write!(f, "Missing key: {}", key),
            JsonError::InvalidValue(msg) => write!(f, "Invalid value: {}", msg),
        }
    }
}

impl std::error::Error for JsonError {}

#[derive(Debug, Clone, PartialEq)]
pub enum JsonValue {
    Null,
    Bool(bool),
    Number(JsonNumber),
    String(String),
    Array(Vec<JsonValue>),
    Object(HashMap<String, JsonValue>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum JsonNumber {
    Int(i64),
    Float(f64),
}

impl JsonValue {
    pub fn null() -> Self {
        JsonValue::Null
    }

    pub fn bool(b: bool) -> Self {
        JsonValue::Bool(b)
    }

    pub fn int(n: i64) -> Self {
        JsonValue::Number(JsonNumber::Int(n))
    }

    pub fn float(f: f64) -> Self {
        JsonValue::Number(JsonNumber::Float(f))
    }

    pub fn string(s: String) -> Self {
        JsonValue::String(s)
    }

    pub fn array(arr: Vec<JsonValue>) -> Self {
        JsonValue::Array(arr)
    }

    pub fn object(obj: HashMap<String, JsonValue>) -> Self {
        JsonValue::Object(obj)
    }

    pub fn is_null(&self) -> bool {
        matches!(self, JsonValue::Null)
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            JsonValue::Bool(b) => Some(*b),
            _ => None,
        }
    }

    pub fn as_i64(&self) -> Option<i64> {
        match self {
            JsonValue::Number(JsonNumber::Int(i)) => Some(*i),
            JsonValue::Number(JsonNumber::Float(f)) => Some(*f as i64),
            _ => None,
        }
    }

    pub fn as_f64(&self) -> Option<f64> {
        match self {
            JsonValue::Number(JsonNumber::Float(f)) => Some(*f),
            JsonValue::Number(JsonNumber::Int(i)) => Some(*i as f64),
            _ => None,
        }
    }

    pub fn as_string(&self) -> Option<&String> {
        match self {
            JsonValue::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_array(&self) -> Option<&Vec<JsonValue>> {
        match self {
            JsonValue::Array(arr) => Some(arr),
            _ => None,
        }
    }

    pub fn as_object(&self) -> Option<&HashMap<String, JsonValue>> {
        match self {
            JsonValue::Object(obj) => Some(obj),
            _ => None,
        }
    }

    pub fn get(&self, key: &str) -> Option<&JsonValue> {
        match self {
            JsonValue::Object(obj) => obj.get(key),
            _ => None,
        }
    }

    pub fn get_mut(&mut self, key: &str) -> Option<&mut JsonValue> {
        match self {
            JsonValue::Object(obj) => obj.get_mut(key),
            _ => None,
        }
    }

    pub fn index(&self, idx: usize) -> Option<&JsonValue> {
        match self {
            JsonValue::Array(arr) => arr.get(idx),
            _ => None,
        }
    }

    pub fn len(&self) -> usize {
        match self {
            JsonValue::Array(arr) => arr.len(),
            JsonValue::Object(obj) => obj.len(),
            JsonValue::String(s) => s.len(),
            _ => 0,
        }
    }
}

impl From<String> for JsonValue {
    fn from(s: String) -> Self {
        JsonValue::String(s)
    }
}

impl From<&str> for JsonValue {
    fn from(s: &str) -> Self {
        JsonValue::String(s.to_string())
    }
}

impl From<i64> for JsonValue {
    fn from(i: i64) -> Self {
        JsonValue::Number(JsonNumber::Int(i))
    }
}

impl From<f64> for JsonValue {
    fn from(f: f64) -> Self {
        JsonValue::Number(JsonNumber::Float(f))
    }
}

impl From<bool> for JsonValue {
    fn from(b: bool) -> Self {
        JsonValue::Bool(b)
    }
}

impl From<Vec<JsonValue>> for JsonValue {
    fn from(arr: Vec<JsonValue>) -> Self {
        JsonValue::Array(arr)
    }
}

impl From<HashMap<String, JsonValue>> for JsonValue {
    fn from(obj: HashMap<String, JsonValue>) -> Self {
        JsonValue::Object(obj)
    }
}

pub struct JsonDocument {
    pub root: JsonValue,
}

impl JsonDocument {
    pub fn new() -> Self {
        JsonDocument {
            root: JsonValue::Null,
        }
    }

    pub fn set_root(&mut self, root: JsonValue) {
        self.root = root;
    }

    pub fn get_root(&self) -> &JsonValue {
        &self.root
    }

    pub fn get_root_mut(&mut self) -> &mut JsonValue {
        &mut self.root
    }

    pub fn to_string(&self) -> JsonResult<String> {
        let mut output = String::new();
        self.write(&mut output)?;
        Ok(output)
    }

    pub fn to_string_pretty(&self, indent: usize) -> JsonResult<String> {
        let mut output = String::new();
        self.write_pretty(&mut output, indent)?;
        Ok(output)
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> JsonResult<()> {
        self.write_value(writer, &self.root, 0)
    }

    pub fn write_pretty<W: Write>(&self, writer: &mut W, indent: usize) -> JsonResult<()> {
        self.write_value_pretty(writer, &self.root, 0, indent)
    }

    fn write_value<W: Write>(&self, writer: &mut W, value: &JsonValue, _depth: usize) -> JsonResult<()> {
        match value {
            JsonValue::Null => write!(writer, "null")?,
            JsonValue::Bool(b) => write!(writer, "{}", if *b { "true" } else { "false" })?,
            JsonValue::Number(n) => match n {
                JsonNumber::Int(i) => write!(writer, "{}", i)?,
                JsonNumber::Float(f) => write!(writer, "{}", f)?,
            },
            JsonValue::String(s) => self.write_string(writer, s)?,
            JsonValue::Array(arr) => {
                write!(writer, "[")?;
                for (i, v) in arr.iter().enumerate() {
                    if i > 0 {
                        write!(writer, ",")?;
                    }
                    self.write_value(writer, v, 0)?;
                }
                write!(writer, "]")?;
            }
            JsonValue::Object(obj) => {
                write!(writer, "{{")?;
                for (i, (k, v)) in obj.iter().enumerate() {
                    if i > 0 {
                        write!(writer, ",")?;
                    }
                    self.write_string(writer, k)?;
                    write!(writer, ":")?;
                    self.write_value(writer, v, 0)?;
                }
                write!(writer, "}}")?;
            }
        }
        Ok(())
    }

    fn write_value_pretty<W: Write>(
        &self,
        writer: &mut W,
        value: &JsonValue,
        depth: usize,
        indent: usize,
    ) -> JsonResult<()> {
        let indent_str = " ".repeat(depth * indent);
        let inner_indent_str = " ".repeat((depth + 1) * indent);

        match value {
            JsonValue::Null => write!(writer, "null")?,
            JsonValue::Bool(b) => write!(writer, "{}", if *b { "true" } else { "false" })?,
            JsonValue::Number(n) => match n {
                JsonNumber::Int(i) => write!(writer, "{}", i)?,
                JsonNumber::Float(f) => write!(writer, "{}", f)?,
            },
            JsonValue::String(s) => self.write_string(writer, s)?,
            JsonValue::Array(arr) => {
                if arr.is_empty() {
                    write!(writer, "[]")?;
                } else {
                    writeln!(writer, "[")?;
                    for (i, v) in arr.iter().enumerate() {
                        write!(writer, "{}", inner_indent_str)?;
                        self.write_value_pretty(writer, v, depth + 1, indent)?;
                        if i < arr.len() - 1 {
                            write!(writer, ",")?;
                        }
                        writeln!(writer)?;
                    }
                    write!(writer, "{}]", indent_str)?;
                }
            }
            JsonValue::Object(obj) => {
                if obj.is_empty() {
                    write!(writer, "{{}}")?;
                } else {
                    writeln!(writer, "{{")?;
                    for (i, (k, v)) in obj.iter().enumerate() {
                        write!(writer, "{}", inner_indent_str)?;
                        self.write_string(writer, k)?;
                        write!(writer, ": ")?;
                        self.write_value_pretty(writer, v, depth + 1, indent)?;
                        if i < obj.len() - 1 {
                            write!(writer, ",")?;
                        }
                        writeln!(writer)?;
                    }
                    write!(writer, "{}}}", indent_str)?;
                }
            }
        }
        Ok(())
    }

    fn write_string<W: Write>(&self, writer: &mut W, s: &str) -> JsonResult<()> {
        write!(writer, "\"")?;
        for c in s.chars() {
            match c {
                '"' => write!(writer, "\\\"")?,
                '\\' => write!(writer, "\\\\")?,
                '\n' => write!(writer, "\\n")?,
                '\r' => write!(writer, "\\r")?,
                '\t' => write!(writer, "\\t")?,
                '\u{0008}' => write!(writer, "\\b")?,
                '\u{000C}' => write!(writer, "\\f")?,
                _ if c < ' ' => {
                    write!(writer, "\\u{:04x}", c as u32)?;
                }
                _ => write!(writer, "{}", c)?,
            }
        }
        write!(writer, "\"")?;
        Ok(())
    }
}

impl Default for JsonDocument {
    fn default() -> Self {
        Self::new()
    }
}

pub struct JsonParser;

impl JsonParser {
    pub fn new() -> Self {
        JsonParser
    }

    pub fn parse<R: Read>(&self, reader: &mut R) -> JsonResult<JsonDocument> {
        let mut content = String::new();
        reader.read_to_string(&mut content)
            .map_err(|e| JsonError::ParseError(e.to_string()))?;

        self.parse_str(&content)
    }

    pub fn parse_str(&self, content: &str) -> JsonResult<JsonDocument> {
        let mut parser = Parser {
            input: content,
            pos: 0,
        };
        let root = parser.parse_value()?;
        Ok(JsonDocument { root })
    }
}

impl Default for JsonParser {
    fn default() -> Self {
        Self::new()
    }
}

struct Parser<'a> {
    input: &'a str,
    pos: usize,
}

impl<'a> Parser<'a> {
    fn parse_value(&mut self) -> JsonResult<JsonValue> {
        self.skip_whitespace();

        if self.pos >= self.input.len() {
            return Err(JsonError::ParseError("Unexpected end of input".to_string()));
        }

        let c = self.input.as_bytes()[self.pos];
        match c {
            b'n' => self.parse_null(),
            b't' => self.parse_true(),
            b'f' => self.parse_false(),
            b'"' => self.parse_string(),
            b'[' => self.parse_array(),
            b'{' => self.parse_object(),
            b'-' | b'0'..=b'9' => self.parse_number(),
            _ => Err(JsonError::ParseError(format!("Unexpected character: {}", c as char))),
        }
    }

    fn skip_whitespace(&mut self) {
        while self.pos < self.input.len() {
            let c = self.input.as_bytes()[self.pos];
            if c.is_ascii_whitespace() {
                self.pos += 1;
            } else {
                break;
            }
        }
    }

    fn parse_null(&mut self) -> JsonResult<JsonValue> {
        if self.input[self.pos..].starts_with("null") {
            self.pos += 4;
            Ok(JsonValue::Null)
        } else {
            Err(JsonError::ParseError("Expected 'null'".to_string()))
        }
    }

    fn parse_true(&mut self) -> JsonResult<JsonValue> {
        if self.input[self.pos..].starts_with("true") {
            self.pos += 4;
            Ok(JsonValue::Bool(true))
        } else {
            Err(JsonError::ParseError("Expected 'true'".to_string()))
        }
    }

    fn parse_false(&mut self) -> JsonResult<JsonValue> {
        if self.input[self.pos..].starts_with("false") {
            self.pos += 5;
            Ok(JsonValue::Bool(false))
        } else {
            Err(JsonError::ParseError("Expected 'false'".to_string()))
        }
    }

    fn parse_string(&mut self) -> JsonResult<JsonValue> {
        self.pos += 1;
        let mut result = String::new();

        while self.pos < self.input.len() {
            let c = self.input.as_bytes()[self.pos];
            self.pos += 1;

            if c == b'"' {
                return Ok(JsonValue::String(result));
            }

            if c == b'\\' {
                if self.pos >= self.input.len() {
                    return Err(JsonError::ParseError("Unclosed escape sequence".to_string()));
                }
                let escaped = self.input.as_bytes()[self.pos];
                self.pos += 1;
                let decoded = match escaped {
                    b'"' => '"',
                    b'\\' => '\\',
                    b'/' => '/',
                    b'b' => '\u{0008}',
                    b'f' => '\u{000C}',
                    b'n' => '\n',
                    b'r' => '\r',
                    b't' => '\t',
                    b'u' => {
                        if self.pos + 4 > self.input.len() {
                            return Err(JsonError::ParseError("Invalid Unicode escape".to_string()));
                        }
                        let hex = &self.input[self.pos..self.pos + 4];
                        self.pos += 4;
                        u32::from_str_radix(hex, 16)
                            .map_err(|_| JsonError::ParseError("Invalid Unicode escape".to_string()))?
                            as char
                    }
                    _ => return Err(JsonError::ParseError("Invalid escape sequence".to_string())),
                };
                result.push(decoded);
            } else {
                result.push(c as char);
            }
        }

        Err(JsonError::ParseError("Unclosed string".to_string()))
    }

    fn parse_number(&mut self) -> JsonResult<JsonValue> {
        let start = self.pos;

        if self.pos < self.input.len() && self.input.as_bytes()[self.pos] == b'-' {
            self.pos += 1;
        }

        if self.pos < self.input.len() && self.input.as_bytes()[self.pos] == b'0' {
            self.pos += 1;
        } else if self.pos < self.input.len() && self.input.as_bytes()[self.pos].is_ascii_digit() {
            while self.pos < self.input.len() && self.input.as_bytes()[self.pos].is_ascii_digit() {
                self.pos += 1;
            }
        }

        if self.pos < self.input.len() && self.input.as_bytes()[self.pos] == b'.' {
            self.pos += 1;
            while self.pos < self.input.len() && self.input.as_bytes()[self.pos].is_ascii_digit() {
                self.pos += 1;
            }
        }

        if self.pos < self.input.len() && (self.input.as_bytes()[self.pos] == b'e' || self.input.as_bytes()[self.pos] == b'E') {
            self.pos += 1;
            if self.pos < self.input.len() && (self.input.as_bytes()[self.pos] == b'+' || self.input.as_bytes()[self.pos] == b'-') {
                self.pos += 1;
            }
            while self.pos < self.input.len() && self.input.as_bytes()[self.pos].is_ascii_digit() {
                self.pos += 1;
            }
        }

        let num_str = &self.input[start..self.pos];

        if num_str.contains('.') || num_str.contains('e') || num_str.contains('E') {
            num_str.parse::<f64>()
                .map(JsonValue::float)
                .map_err(|_| JsonError::ParseError("Invalid float number".to_string()))
        } else {
            num_str.parse::<i64>()
                .map(JsonValue::int)
                .map_err(|_| JsonError::ParseError("Invalid integer number".to_string()))
        }
    }

    fn parse_array(&mut self) -> JsonResult<JsonValue> {
        self.pos += 1;
        self.skip_whitespace();

        let mut arr = Vec::new();

        if self.pos < self.input.len() && self.input.as_bytes()[self.pos] != b']' {
            loop {
                self.skip_whitespace();
                arr.push(self.parse_value()?);
                self.skip_whitespace();

                if self.pos >= self.input.len() {
                    return Err(JsonError::ParseError("Unclosed array".to_string()));
                }

                let c = self.input.as_bytes()[self.pos];
                self.pos += 1;

                if c == b']' {
                    break;
                } else if c != b',' {
                    return Err(JsonError::ParseError("Expected ',' or ']'".to_string()));
                }
            }
        } else {
            self.pos += 1;
        }

        Ok(JsonValue::Array(arr))
    }

    fn parse_object(&mut self) -> JsonResult<JsonValue> {
        self.pos += 1;
        self.skip_whitespace();

        let mut obj = HashMap::new();

        if self.pos < self.input.len() && self.input.as_bytes()[self.pos] != b'}' {
            loop {
                self.skip_whitespace();

                if self.pos >= self.input.len() {
                    return Err(JsonError::ParseError("Unclosed object".to_string()));
                }

                let key = match self.parse_value()? {
                    JsonValue::String(s) => s,
                    _ => return Err(JsonError::ParseError("Object key must be a string".to_string())),
                };

                self.skip_whitespace();

                if self.pos >= self.input.len() || self.input.as_bytes()[self.pos] != b':' {
                    return Err(JsonError::ParseError("Expected ':'".to_string()));
                }
                self.pos += 1;

                self.skip_whitespace();
                let value = self.parse_value()?;
                obj.insert(key, value);

                self.skip_whitespace();

                if self.pos >= self.input.len() {
                    return Err(JsonError::ParseError("Unclosed object".to_string()));
                }

                let c = self.input.as_bytes()[self.pos];
                self.pos += 1;

                if c == b'}' {
                    break;
                } else if c != b',' {
                    return Err(JsonError::ParseError("Expected ',' or '}'".to_string()));
                }
            }
        } else {
            self.pos += 1;
        }

        Ok(JsonValue::Object(obj))
    }
}

pub fn parse<R: Read>(reader: &mut R) -> JsonResult<JsonDocument> {
    JsonParser::new().parse(reader)
}

pub fn parse_str(content: &str) -> JsonResult<JsonDocument> {
    JsonParser::new().parse_str(content)
}

pub fn to_string(value: &JsonValue) -> JsonResult<String> {
    let doc = JsonDocument { root: value.clone() };
    doc.to_string()
}

pub fn to_string_pretty(value: &JsonValue, indent: usize) -> JsonResult<String> {
    let doc = JsonDocument { root: value.clone() };
    doc.to_string_pretty(indent)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_value() {
        let value = JsonValue::string("hello".to_string());
        assert_eq!(value.as_string(), Some(&"hello".to_string()));
    }

    #[test]
    fn test_json_document() {
        let mut doc = JsonDocument::new();
        let mut obj = HashMap::new();
        obj.insert("name".to_string(), JsonValue::string("test".to_string()));
        obj.insert("value".to_string(), JsonValue::int(42));
        doc.set_root(JsonValue::object(obj));

        let json = doc.to_string().unwrap();
        assert!(json.contains("\"name\""));
        assert!(json.contains("\"test\""));
    }

    #[test]
    fn test_json_parser() {
        let json = r#"{"name":"test","value":42}"#;
        let doc = parse_str(json).unwrap();
        let root = doc.get_root();
        assert_eq!(root.get("name").and_then(|v| v.as_string()), Some(&"test".to_string()));
        assert_eq!(root.get("value").and_then(|v| v.as_i64()), Some(42));
    }

    #[test]
    fn test_json_array() {
        let json = r#"[1,2,3,4,5]"#;
        let doc = parse_str(json).unwrap();
        let arr = doc.get_root().as_array().unwrap();
        assert_eq!(arr.len(), 5);
        assert_eq!(arr[0].as_i64(), Some(1));
    }

    #[test]
    fn test_json_pretty() {
        let mut doc = JsonDocument::new();
        let mut obj = HashMap::new();
        obj.insert("name".to_string(), JsonValue::string("test".to_string()));
        doc.set_root(JsonValue::object(obj));

        let json = doc.to_string_pretty(2).unwrap();
        assert!(json.contains("\n"));
        assert!(json.contains("  "));
    }
}
