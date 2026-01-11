use std::collections::HashMap;
use std::io::{self, Read, Write};

pub type ToonResult<T> = Result<T, ToonError>;

#[derive(Debug, Clone)]
pub enum ToonError {
    ParseError(String),
    WriteError(String),
    InvalidFormat(String),
    MissingKey(String),
    InvalidValue(String),
}

impl std::fmt::Display for ToonError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ToonError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            ToonError::WriteError(msg) => write!(f, "Write error: {}", msg),
            ToonError::InvalidFormat(msg) => write!(f, "Invalid format: {}", msg),
            ToonError::MissingKey(key) => write!(f, "Missing key: {}", key),
            ToonError::InvalidValue(msg) => write!(f, "Invalid value: {}", msg),
        }
    }
}

impl std::error::Error for ToonError {}

#[derive(Debug, Clone, PartialEq)]
pub enum ToonValue {
    Null,
    Bool(bool),
    Integer(i64),
    Float(f64),
    String(String),
    Binary(Vec<u8>),
    List(Vec<ToonValue>),
    Map(HashMap<String, ToonValue>),
}

impl ToonValue {
    pub fn null() -> Self {
        ToonValue::Null
    }

    pub fn bool(b: bool) -> Self {
        ToonValue::Bool(b)
    }

    pub fn int(n: i64) -> Self {
        ToonValue::Integer(n)
    }

    pub fn float(f: f64) -> Self {
        ToonValue::Float(f)
    }

    pub fn string(s: String) -> Self {
        ToonValue::String(s)
    }

    pub fn binary(data: Vec<u8>) -> Self {
        ToonValue::Binary(data)
    }

    pub fn list(list: Vec<ToonValue>) -> Self {
        ToonValue::List(list)
    }

    pub fn map(map: HashMap<String, ToonValue>) -> Self {
        ToonValue::Map(map)
    }

    pub fn is_null(&self) -> bool {
        matches!(self, ToonValue::Null)
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            ToonValue::Bool(b) => Some(*b),
            _ => None,
        }
    }

    pub fn as_i64(&self) -> Option<i64> {
        match self {
            ToonValue::Integer(i) => Some(*i),
            ToonValue::Float(f) => Some(*f as i64),
            _ => None,
        }
    }

    pub fn as_f64(&self) -> Option<f64> {
        match self {
            ToonValue::Float(f) => Some(*f),
            ToonValue::Integer(i) => Some(*i as f64),
            _ => None,
        }
    }

    pub fn as_string(&self) -> Option<&String> {
        match self {
            ToonValue::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_binary(&self) -> Option<&Vec<u8>> {
        match self {
            ToonValue::Binary(data) => Some(data),
            _ => None,
        }
    }

    pub fn as_list(&self) -> Option<&Vec<ToonValue>> {
        match self {
            ToonValue::List(list) => Some(list),
            _ => None,
        }
    }

    pub fn as_map(&self) -> Option<&HashMap<String, ToonValue>> {
        match self {
            ToonValue::Map(map) => Some(map),
            _ => None,
        }
    }

    pub fn get(&self, key: &str) -> Option<&ToonValue> {
        match self {
            ToonValue::Map(map) => map.get(key),
            _ => None,
        }
    }

    pub fn get_mut(&mut self, key: &str) -> Option<&mut ToonValue> {
        match self {
            ToonValue::Map(map) => map.get_mut(key),
            _ => None,
        }
    }

    pub fn index(&self, idx: usize) -> Option<&ToonValue> {
        match self {
            ToonValue::List(list) => list.get(idx),
            _ => None,
        }
    }

    pub fn len(&self) -> usize {
        match self {
            ToonValue::List(list) => list.len(),
            ToonValue::Map(map) => map.len(),
            ToonValue::String(s) => s.len(),
            ToonValue::Binary(data) => data.len(),
            _ => 0,
        }
    }
}

impl From<String> for ToonValue {
    fn from(s: String) -> Self {
        ToonValue::String(s)
    }
}

impl From<&str> for ToonValue {
    fn from(s: &str) -> Self {
        ToonValue::String(s.to_string())
    }
}

impl From<i64> for ToonValue {
    fn from(i: i64) -> Self {
        ToonValue::Integer(i)
    }
}

impl From<f64> for ToonValue {
    fn from(f: f64) -> Self {
        ToonValue::Float(f)
    }
}

impl From<bool> for ToonValue {
    fn from(b: bool) -> Self {
        ToonValue::Bool(b)
    }
}

impl From<Vec<ToonValue>> for ToonValue {
    fn from(list: Vec<ToonValue>) -> Self {
        ToonValue::List(list)
    }
}

impl From<HashMap<String, ToonValue>> for ToonValue {
    fn from(map: HashMap<String, ToonValue>) -> Self {
        ToonValue::Map(map)
    }
}

impl From<Vec<u8>> for ToonValue {
    fn from(data: Vec<u8>) -> Self {
        ToonValue::Binary(data)
    }
}

pub struct ToonDocument {
    pub root: ToonValue,
    pub metadata: HashMap<String, String>,
}

impl ToonDocument {
    pub fn new() -> Self {
        ToonDocument {
            root: ToonValue::Null,
            metadata: HashMap::new(),
        }
    }

    pub fn set_root(&mut self, root: ToonValue) {
        self.root = root;
    }

    pub fn get_root(&self) -> &ToonValue {
        &self.root
    }

    pub fn get_root_mut(&mut self) -> &mut ToonValue {
        &mut self.root
    }

    pub fn add_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }

    pub fn get_metadata(&self, key: &str) -> Option<&String> {
        self.metadata.get(key)
    }

    pub fn to_string(&self) -> ToonResult<String> {
        let mut output = String::new();
        self.write(&mut output)?;
        Ok(output)
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> ToonResult<()> {
        writeln!(writer, "# Toon Format v1.0")?;

        for (key, value) in &self.metadata {
            writeln!(writer, "# {}: {}", key, value)?;
        }

        self.write_value(writer, &self.root, 0)?;
        Ok(())
    }

    fn write_value<W: Write>(
        &self,
        writer: &mut W,
        value: &ToonValue,
        depth: usize,
    ) -> ToonResult<()> {
        let indent = " ".repeat(depth * 2);

        match value {
            ToonValue::Null => writeln!(writer, "{}null", indent)?,
            ToonValue::Bool(b) => writeln!(writer, "{}{}", indent, if *b { "true" } else { "false" })?,
            ToonValue::Integer(i) => writeln!(writer, "{}i:{}", indent, i)?,
            ToonValue::Float(f) => writeln!(writer, "{}f:{}", indent, f)?,
            ToonValue::String(s) => {
                if s.contains('\n') || s.contains('"') {
                    writeln!(writer, "{}s:|", indent)?;
                    for line in s.lines() {
                        writeln!(writer, "{}  {}", indent, line)?;
                    }
                } else {
                    writeln!(writer, "{}s:\"{}\"", indent, s)?;
                }
            }
            ToonValue::Binary(data) => {
                writeln!(writer, "{}b:{}", indent, base64::encode(data))?;
            }
            ToonValue::List(list) => {
                writeln!(writer, "{}[", indent)?;
                for v in list {
                    self.write_value(writer, v, depth + 1)?;
                }
                writeln!(writer, "{}]", indent)?;
            }
            ToonValue::Map(map) => {
                writeln!(writer, "{}{{", indent)?;
                for (k, v) in map {
                    write!(writer, "{}  {}: ", indent, k)?;
                    self.write_value(writer, v, 0)?;
                }
                writeln!(writer, "{}}}", indent)?;
            }
        }

        Ok(())
    }
}

impl Default for ToonDocument {
    fn default() -> Self {
        Self::new()
    }
}

pub struct ToonParser;

impl ToonParser {
    pub fn new() -> Self {
        ToonParser
    }

    pub fn parse<R: Read>(&self, reader: &mut R) -> ToonResult<ToonDocument> {
        let mut content = String::new();
        reader.read_to_string(&mut content)
            .map_err(|e| ToonError::ParseError(e.to_string()))?;

        self.parse_str(&content)
    }

    pub fn parse_str(&self, content: &str) -> ToonResult<ToonDocument> {
        let mut parser = Parser {
            input: content,
            pos: 0,
            metadata: HashMap::new(),
        };
        let root = parser.parse_value()?;
        Ok(ToonDocument {
            root,
            metadata: parser.metadata,
        })
    }
}

impl Default for ToonParser {
    fn default() -> Self {
        Self::new()
    }
}

struct Parser<'a> {
    input: &'a str,
    pos: usize,
    metadata: HashMap<String, String>,
}

impl<'a> Parser<'a> {
    fn parse_value(&mut self) -> ToonResult<ToonValue> {
        self.skip_whitespace();

        if self.pos >= self.input.len() {
            return Ok(ToonValue::Null);
        }

        let c = self.input.as_bytes()[self.pos];
        match c {
            b'n' => self.parse_null(),
            b't' => self.parse_true(),
            b'f' => self.parse_false(),
            b'i' => self.parse_integer(),
            b'f' => self.parse_float(),
            b's' => self.parse_string(),
            b'b' => self.parse_binary(),
            b'[' => self.parse_list(),
            b'{' => self.parse_map(),
            b'#' => self.parse_comment(),
            _ => Err(ToonError::ParseError(format!("Unexpected character: {}", c as char))),
        }
    }

    fn skip_whitespace(&mut self) {
        while self.pos < self.input.len() {
            let c = self.input.as_bytes()[self.pos];
            if c.is_ascii_whitespace() || c == b'\n' || c == b'\r' {
                self.pos += 1;
            } else {
                break;
            }
        }
    }

    fn skip_to_next_line(&mut self) {
        while self.pos < self.input.len() {
            let c = self.input.as_bytes()[self.pos];
            if c == b'\n' || c == b'\r' {
                self.pos += 1;
                break;
            }
            self.pos += 1;
        }
    }

    fn parse_null(&mut self) -> ToonResult<ToonValue> {
        if self.input[self.pos..].starts_with("null") {
            self.pos += 4;
            Ok(ToonValue::Null)
        } else {
            Err(ToonError::ParseError("Expected 'null'".to_string()))
        }
    }

    fn parse_true(&mut self) -> ToonResult<ToonValue> {
        if self.input[self.pos..].starts_with("true") {
            self.pos += 4;
            Ok(ToonValue::Bool(true))
        } else {
            Err(ToonError::ParseError("Expected 'true'".to_string()))
        }
    }

    fn parse_false(&mut self) -> ToonResult<ToonValue> {
        if self.input[self.pos..].starts_with("false") {
            self.pos += 5;
            Ok(ToonValue::Bool(false))
        } else {
            Err(ToonError::ParseError("Expected 'false'".to_string()))
        }
    }

    fn parse_integer(&mut self) -> ToonResult<ToonValue> {
        self.pos += 1;

        if self.pos >= self.input.len() || self.input.as_bytes()[self.pos] != b':' {
            return Err(ToonError::ParseError("Expected ':' after 'i'".to_string()));
        }
        self.pos += 1;

        let start = self.pos;

        if self.pos < self.input.len() && self.input.as_bytes()[self.pos] == b'-' {
            self.pos += 1;
        }

        while self.pos < self.input.len() && self.input.as_bytes()[self.pos].is_ascii_digit() {
            self.pos += 1;
        }

        let num_str = &self.input[start..self.pos];
        num_str.parse::<i64>()
            .map(ToonValue::int)
            .map_err(|_| ToonError::ParseError("Invalid integer".to_string()))
    }

    fn parse_float(&mut self) -> ToonResult<ToonValue> {
        self.pos += 1;

        if self.pos >= self.input.len() || self.input.as_bytes()[self.pos] != b':' {
            return Err(ToonError::ParseError("Expected ':' after 'f'".to_string()));
        }
        self.pos += 1;

        let start = self.pos;

        if self.pos < self.input.len() && self.input.as_bytes()[self.pos] == b'-' {
            self.pos += 1;
        }

        while self.pos < self.input.len() && self.input.as_bytes()[self.pos].is_ascii_digit() {
            self.pos += 1;
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
        num_str.parse::<f64>()
            .map(ToonValue::float)
            .map_err(|_| ToonError::ParseError("Invalid float".to_string()))
    }

    fn parse_string(&mut self) -> ToonResult<ToonValue> {
        self.pos += 1;

        if self.pos >= self.input.len() {
            return Err(ToonError::ParseError("Unexpected end of input".to_string()));
        }

        let c = self.input.as_bytes()[self.pos];
        self.pos += 1;

        match c {
            b'"' => self.parse_quoted_string(),
            b'|' => self.parse_block_string(),
            _ => Err(ToonError::ParseError("Expected '\"' or '|' after 's'".to_string())),
        }
    }

    fn parse_quoted_string(&mut self) -> ToonResult<ToonValue> {
        let mut result = String::new();

        while self.pos < self.input.len() {
            let c = self.input.as_bytes()[self.pos];
            self.pos += 1;

            if c == b'"' {
                return Ok(ToonValue::String(result));
            }

            if c == b'\\' {
                if self.pos >= self.input.len() {
                    return Err(ToonError::ParseError("Unclosed escape sequence".to_string()));
                }
                let escaped = self.input.as_bytes()[self.pos];
                self.pos += 1;
                let decoded = match escaped {
                    b'"' => '"',
                    b'\\' => '\\',
                    b'n' => '\n',
                    b'r' => '\r',
                    b't' => '\t',
                    _ => return Err(ToonError::ParseError("Invalid escape sequence".to_string())),
                };
                result.push(decoded);
            } else {
                result.push(c as char);
            }
        }

        Err(ToonError::ParseError("Unclosed string".to_string()))
    }

    fn parse_block_string(&mut self) -> ToonResult<ToonValue> {
        let mut result = String::new();

        loop {
            if self.pos >= self.input.len() {
                return Err(ToonError::ParseError("Unclosed block string".to_string()));
            }

            let c = self.input.as_bytes()[self.pos];

            if c.is_ascii_whitespace() || c == b'\n' || c == b'\r' {
                self.pos += 1;
                continue;
            }

            let next_c = if self.pos + 1 < self.input.len() {
                self.input.as_bytes()[self.pos + 1]
            } else {
                0
            };

            if next_c.is_ascii_whitespace() || next_c == b'\n' || next_c == b'\r' {
                self.pos += 1;
                break;
            }

            result.push(c as char);
            self.pos += 1;
        }

        Ok(ToonValue::String(result))
    }

    fn parse_binary(&mut self) -> ToonResult<ToonValue> {
        self.pos += 1;

        if self.pos >= self.input.len() || self.input.as_bytes()[self.pos] != b':' {
            return Err(ToonError::ParseError("Expected ':' after 'b'".to_string()));
        }
        self.pos += 1;

        let start = self.pos;

        while self.pos < self.input.len() {
            let c = self.input.as_bytes()[self.pos];
            if c.is_ascii_whitespace() || c == b'\n' || c == b'\r' {
                break;
            }
            self.pos += 1;
        }

        let encoded = &self.input[start..self.pos];
        base64::decode(encoded)
            .map(ToonValue::binary)
            .map_err(|_| ToonError::ParseError("Invalid base64 encoding".to_string()))
    }

    fn parse_list(&mut self) -> ToonResult<ToonValue> {
        self.pos += 1;
        self.skip_whitespace();

        let mut list = Vec::new();

        loop {
            self.skip_whitespace();

            if self.pos >= self.input.len() {
                return Err(ToonError::ParseError("Unclosed list".to_string()));
            }

            if self.input.as_bytes()[self.pos] == b']' {
                self.pos += 1;
                break;
            }

            list.push(self.parse_value()?);
        }

        Ok(ToonValue::List(list))
    }

    fn parse_map(&mut self) -> ToonResult<ToonValue> {
        self.pos += 1;
        self.skip_whitespace();

        let mut map = HashMap::new();

        loop {
            self.skip_whitespace();

            if self.pos >= self.input.len() {
                return Err(ToonError::ParseError("Unclosed map".to_string()));
            }

            if self.input.as_bytes()[self.pos] == b'}' {
                self.pos += 1;
                break;
            }

            let key = match self.parse_value()? {
                ToonValue::String(s) => s,
                _ => return Err(ToonError::ParseError("Map key must be a string".to_string())),
            };

            self.skip_whitespace();

            if self.pos >= self.input.len() || self.input.as_bytes()[self.pos] != b':' {
                return Err(ToonError::ParseError("Expected ':' after key".to_string()));
            }
            self.pos += 1;

            self.skip_whitespace();
            let value = self.parse_value()?;
            map.insert(key, value);
        }

        Ok(ToonValue::Map(map))
    }

    fn parse_comment(&mut self) -> ToonResult<ToonValue> {
        self.pos += 1;
        self.skip_whitespace();

        if self.pos >= self.input.len() {
            return Ok(ToonValue::Null);
        }

        let c = self.input.as_bytes()[self.pos];
        self.pos += 1;

        if c == b' ' {
            let start = self.pos;

            while self.pos < self.input.len() {
                let c = self.input.as_bytes()[self.pos];
                if c == b':' {
                    self.pos += 1;
                    let key = self.input[start..self.pos - 1].to_string();
                    let value_start = self.pos;
                    self.skip_to_next_line();
                    let value = self.input[value_start..self.pos].trim().to_string();
                    self.metadata.insert(key, value);
                    break;
                }
                self.pos += 1;
            }
        } else {
            self.skip_to_next_line();
        }

        self.parse_value()
    }
}

pub fn parse<R: Read>(reader: &mut R) -> ToonResult<ToonDocument> {
    ToonParser::new().parse(reader)
}

pub fn parse_str(content: &str) -> ToonResult<ToonDocument> {
    ToonParser::new().parse_str(content)
}

pub fn to_string(value: &ToonValue) -> ToonResult<String> {
    let doc = ToonDocument { root: value.clone(), metadata: HashMap::new() };
    doc.to_string()
}

mod base64 {
    pub fn encode(data: &[u8]) -> String {
        const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

        let mut result = String::new();
        let mut i = 0;

        while i < data.len() {
            let chunk = &data[i..std::cmp::min(i + 3, data.len())];
            let mut acc = 0u32;
            let mut bits = 0;

            for &byte in chunk {
                acc = (acc << 8) | (byte as u32);
                bits += 8;
            }

            while bits > 0 {
                let idx = ((acc >> (bits - 6)) & 0x3F) as usize;
                result.push(TABLE[idx] as char);
                bits -= 6;
            }

            i += 3;
        }

        while result.len() % 4 != 0 {
            result.push('=');
        }

        result
    }

    pub fn decode(encoded: &str) -> Result<Vec<u8>, String> {
        const TABLE: &[i8; 128] = &[
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            62, -1, -1, -1, 63, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61,
            -1, -1, -1, -1, -1, -1, -1, 0, 1, 2, 3, 4, 5, 6, 7,
            8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22,
            23, 24, 25, -1, -1, -1, -1, -1, -1, -1, 26, 27, 28, 29, 30, 31,
            32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47,
            48, 49, 50, 51,
        ];

        let encoded = encoded.trim_end_matches('=');
        let mut result = Vec::new();
        let mut i = 0;

        while i < encoded.len() {
            let chunk = &encoded.as_bytes()[i..std::cmp::min(i + 4, encoded.len())];
            let mut acc = 0u32;
            let mut bits = 0;

            for &byte in chunk {
                if byte >= 128 || TABLE[byte as usize] == -1 {
                    return Err("Invalid base64 character".to_string());
                }
                acc = (acc << 6) | (TABLE[byte as usize] as u32);
                bits += 6;
            }

            while bits >= 8 {
                result.push(((acc >> (bits - 8)) & 0xFF) as u8);
                bits -= 8;
            }

            i += 4;
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_toon_value() {
        let value = ToonValue::string("hello".to_string());
        assert_eq!(value.as_string(), Some(&"hello".to_string()));
    }

    #[test]
    fn test_toon_document() {
        let mut doc = ToonDocument::new();
        let mut map = HashMap::new();
        map.insert("name".to_string(), ToonValue::string("test".to_string()));
        map.insert("value".to_string(), ToonValue::int(42));
        doc.set_root(ToonValue::map(map));

        let toon = doc.to_string().unwrap();
        assert!(toon.contains("name"));
        assert!(toon.contains("test"));
    }

    #[test]
    fn test_toon_parser() {
        let toon = "name: s:\"test\"\nvalue: i:42";
        let doc = parse_str(toon).unwrap();
        let root = doc.get_root();
        assert_eq!(root.get("name").and_then(|v| v.as_string()), Some(&"test".to_string()));
        assert_eq!(root.get("value").and_then(|v| v.as_i64()), Some(42));
    }

    #[test]
    fn test_toon_list() {
        let toon = "[\n  i:1\n  i:2\n  i:3\n]";
        let doc = parse_str(toon).unwrap();
        let list = doc.get_root().as_list().unwrap();
        assert_eq!(list.len(), 3);
        assert_eq!(list[0].as_i64(), Some(1));
    }

    #[test]
    fn test_toon_binary() {
        let data = vec![1u8, 2u8, 3u8, 4u8];
        let value = ToonValue::binary(data.clone());
        let doc = ToonDocument { root: value, metadata: HashMap::new() };
        let toon = doc.to_string().unwrap();
        assert!(toon.contains("b:"));
    }

    #[test]
    fn test_base64_encode() {
        let data = vec![1u8, 2u8, 3u8];
        let encoded = base64::encode(&data);
        assert!(!encoded.is_empty());
    }

    #[test]
    fn test_base64_decode() {
        let encoded = "AQID";
        let decoded = base64::decode(encoded).unwrap();
        assert_eq!(decoded, vec![1u8, 2u8, 3u8]);
    }
}
