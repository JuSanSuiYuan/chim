use std::collections::HashMap;
use std::io::{self, Read, Write};

pub type YamlResult<T> = Result<T, YamlError>;

#[derive(Debug, Clone)]
pub enum YamlError {
    ParseError(String),
    WriteError(String),
    InvalidFormat(String),
    MissingKey(String),
    InvalidValue(String),
}

impl std::fmt::Display for YamlError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            YamlError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            YamlError::WriteError(msg) => write!(f, "Write error: {}", msg),
            YamlError::InvalidFormat(msg) => write!(f, "Invalid format: {}", msg),
            YamlError::MissingKey(key) => write!(f, "Missing key: {}", key),
            YamlError::InvalidValue(msg) => write!(f, "Invalid value: {}", msg),
        }
    }
}

impl std::error::Error for YamlError {}

#[derive(Debug, Clone, PartialEq)]
pub enum YamlValue {
    Null,
    Bool(bool),
    Integer(i64),
    Float(f64),
    String(String),
    Sequence(Vec<YamlValue>),
    Mapping(HashMap<String, YamlValue>),
    Alias(String),
}

impl YamlValue {
    pub fn null() -> Self {
        YamlValue::Null
    }

    pub fn bool(b: bool) -> Self {
        YamlValue::Bool(b)
    }

    pub fn int(n: i64) -> Self {
        YamlValue::Integer(n)
    }

    pub fn float(f: f64) -> Self {
        YamlValue::Float(f)
    }

    pub fn string(s: String) -> Self {
        YamlValue::String(s)
    }

    pub fn sequence(seq: Vec<YamlValue>) -> Self {
        YamlValue::Sequence(seq)
    }

    pub fn mapping(map: HashMap<String, YamlValue>) -> Self {
        YamlValue::Mapping(map)
    }

    pub fn alias(name: String) -> Self {
        YamlValue::Alias(name)
    }

    pub fn is_null(&self) -> bool {
        matches!(self, YamlValue::Null)
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            YamlValue::Bool(b) => Some(*b),
            _ => None,
        }
    }

    pub fn as_i64(&self) -> Option<i64> {
        match self {
            YamlValue::Integer(i) => Some(*i),
            YamlValue::Float(f) => Some(*f as i64),
            _ => None,
        }
    }

    pub fn as_f64(&self) -> Option<f64> {
        match self {
            YamlValue::Float(f) => Some(*f),
            YamlValue::Integer(i) => Some(*i as f64),
            _ => None,
        }
    }

    pub fn as_string(&self) -> Option<&String> {
        match self {
            YamlValue::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_sequence(&self) -> Option<&Vec<YamlValue>> {
        match self {
            YamlValue::Sequence(seq) => Some(seq),
            _ => None,
        }
    }

    pub fn as_mapping(&self) -> Option<&HashMap<String, YamlValue>> {
        match self {
            YamlValue::Mapping(map) => Some(map),
            _ => None,
        }
    }

    pub fn get(&self, key: &str) -> Option<&YamlValue> {
        match self {
            YamlValue::Mapping(map) => map.get(key),
            _ => None,
        }
    }

    pub fn get_mut(&mut self, key: &str) -> Option<&mut YamlValue> {
        match self {
            YamlValue::Mapping(map) => map.get_mut(key),
            _ => None,
        }
    }

    pub fn index(&self, idx: usize) -> Option<&YamlValue> {
        match self {
            YamlValue::Sequence(seq) => seq.get(idx),
            _ => None,
        }
    }

    pub fn len(&self) -> usize {
        match self {
            YamlValue::Sequence(seq) => seq.len(),
            YamlValue::Mapping(map) => map.len(),
            YamlValue::String(s) => s.len(),
            _ => 0,
        }
    }
}

impl From<String> for YamlValue {
    fn from(s: String) -> Self {
        YamlValue::String(s)
    }
}

impl From<&str> for YamlValue {
    fn from(s: &str) -> Self {
        YamlValue::String(s.to_string())
    }
}

impl From<i64> for YamlValue {
    fn from(i: i64) -> Self {
        YamlValue::Integer(i)
    }
}

impl From<f64> for YamlValue {
    fn from(f: f64) -> Self {
        YamlValue::Float(f)
    }
}

impl From<bool> for YamlValue {
    fn from(b: bool) -> Self {
        YamlValue::Bool(b)
    }
}

impl From<Vec<YamlValue>> for YamlValue {
    fn from(seq: Vec<YamlValue>) -> Self {
        YamlValue::Sequence(seq)
    }
}

impl From<HashMap<String, YamlValue>> for YamlValue {
    fn from(map: HashMap<String, YamlValue>) -> Self {
        YamlValue::Mapping(map)
    }
}

pub struct YamlDocument {
    pub root: YamlValue,
    pub anchors: HashMap<String, YamlValue>,
}

impl YamlDocument {
    pub fn new() -> Self {
        YamlDocument {
            root: YamlValue::Null,
            anchors: HashMap::new(),
        }
    }

    pub fn set_root(&mut self, root: YamlValue) {
        self.root = root;
    }

    pub fn get_root(&self) -> &YamlValue {
        &self.root
    }

    pub fn get_root_mut(&mut self) -> &mut YamlValue {
        &mut self.root
    }

    pub fn add_anchor(&mut self, name: String, value: YamlValue) {
        self.anchors.insert(name, value);
    }

    pub fn get_anchor(&self, name: &str) -> Option<&YamlValue> {
        self.anchors.get(name)
    }

    pub fn to_string(&self) -> YamlResult<String> {
        let mut output = String::new();
        self.write(&mut output)?;
        Ok(output)
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> YamlResult<()> {
        self.write_value(writer, &self.root, 0)
    }

    fn write_value<W: Write>(
        &self,
        writer: &mut W,
        value: &YamlValue,
        depth: usize,
    ) -> YamlResult<()> {
        let indent = " ".repeat(depth * 2);

        match value {
            YamlValue::Null => writeln!(writer, "{}~", indent)?,
            YamlValue::Bool(b) => writeln!(writer, "{}{}", indent, if *b { "true" } else { "false" })?,
            YamlValue::Integer(i) => writeln!(writer, "{}{}", indent, i)?,
            YamlValue::Float(f) => writeln!(writer, "{}{}", indent, f)?,
            YamlValue::String(s) => {
                if s.contains('\n') || s.contains(':') || s.contains('#') {
                    writeln!(writer, "{}|", indent)?;
                    for line in s.lines() {
                        writeln!(writer, "{}  {}", indent, line)?;
                    }
                } else {
                    writeln!(writer, "{}{}", indent, s)?;
                }
            }
            YamlValue::Sequence(seq) => {
                if seq.is_empty() {
                    writeln!(writer, "{}[]", indent)?;
                } else {
                    for v in seq {
                        self.write_value(writer, v, depth + 1)?;
                    }
                }
            }
            YamlValue::Mapping(map) => {
                if map.is_empty() {
                    writeln!(writer, "{{}}", indent)?;
                } else {
                    for (k, v) in map {
                        write!(writer, "{}{}:", indent, k)?;
                        match v {
                            YamlValue::Sequence(_) | YamlValue::Mapping(_) => {
                                writeln!(writer)?;
                                self.write_value(writer, v, depth + 1)?;
                            }
                            _ => {
                                write!(writer, " ")?;
                                self.write_value(writer, v, 0)?;
                            }
                        }
                    }
                }
            }
            YamlValue::Alias(name) => writeln!(writer, "{}*{}", indent, name)?,
        }

        Ok(())
    }
}

impl Default for YamlDocument {
    fn default() -> Self {
        Self::new()
    }
}

pub struct YamlParser;

impl YamlParser {
    pub fn new() -> Self {
        YamlParser
    }

    pub fn parse<R: Read>(&self, reader: &mut R) -> YamlResult<YamlDocument> {
        let mut content = String::new();
        reader.read_to_string(&mut content)
            .map_err(|e| YamlError::ParseError(e.to_string()))?;

        self.parse_str(&content)
    }

    pub fn parse_str(&self, content: &str) -> YamlResult<YamlDocument> {
        let mut parser = Parser {
            input: content,
            pos: 0,
            anchors: HashMap::new(),
        };
        let root = parser.parse_value()?;
        Ok(YamlDocument {
            root,
            anchors: parser.anchors,
        })
    }
}

impl Default for YamlParser {
    fn default() -> Self {
        Self::new()
    }
}

struct Parser<'a> {
    input: &'a str,
    pos: usize,
    anchors: HashMap<String, YamlValue>,
}

impl<'a> Parser<'a> {
    fn parse_value(&mut self) -> YamlResult<YamlValue> {
        self.skip_whitespace();

        if self.pos >= self.input.len() {
            return Ok(YamlValue::Null);
        }

        let c = self.input.as_bytes()[self.pos];
        match c {
            b'~' => self.parse_null(),
            b't' => self.parse_true(),
            b'f' => self.parse_false(),
            b'-' | b'0'..=b'9' => self.parse_number(),
            b'"' => self.parse_string(),
            b'\'' => self.parse_single_quoted_string(),
            b'[' => self.parse_sequence(),
            b'{' => self.parse_mapping(),
            b'|' | b'>' => self.parse_block_scalar(),
            b'&' => self.parse_anchor(),
            b'*' => self.parse_alias(),
            b'%' => self.parse_directive(),
            _ => self.parse_plain_scalar(),
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

    fn parse_null(&mut self) -> YamlResult<YamlValue> {
        self.pos += 1;
        Ok(YamlValue::Null)
    }

    fn parse_true(&mut self) -> YamlResult<YamlValue> {
        if self.input[self.pos..].starts_with("true") {
            self.pos += 4;
            Ok(YamlValue::Bool(true))
        } else {
            Err(YamlError::ParseError("Expected 'true'".to_string()))
        }
    }

    fn parse_false(&mut self) -> YamlResult<YamlValue> {
        if self.input[self.pos..].starts_with("false") {
            self.pos += 5;
            Ok(YamlValue::Bool(false))
        } else {
            Err(YamlError::ParseError("Expected 'false'".to_string()))
        }
    }

    fn parse_number(&mut self) -> YamlResult<YamlValue> {
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

        if num_str.contains('.') || num_str.contains('e') || num_str.contains('E') {
            num_str.parse::<f64>()
                .map(YamlValue::float)
                .map_err(|_| YamlError::ParseError("Invalid float number".to_string()))
        } else {
            num_str.parse::<i64>()
                .map(YamlValue::int)
                .map_err(|_| YamlError::ParseError("Invalid integer number".to_string()))
        }
    }

    fn parse_string(&mut self) -> YamlResult<YamlValue> {
        self.pos += 1;
        let mut result = String::new();

        while self.pos < self.input.len() {
            let c = self.input.as_bytes()[self.pos];
            self.pos += 1;

            if c == b'"' {
                return Ok(YamlValue::String(result));
            }

            if c == b'\\' {
                if self.pos >= self.input.len() {
                    return Err(YamlError::ParseError("Unclosed escape sequence".to_string()));
                }
                let escaped = self.input.as_bytes()[self.pos];
                self.pos += 1;
                let decoded = match escaped {
                    b'"' => '"',
                    b'\\' => '\\',
                    b'n' => '\n',
                    b'r' => '\r',
                    b't' => '\t',
                    b'u' => {
                        if self.pos + 4 > self.input.len() {
                            return Err(YamlError::ParseError("Invalid Unicode escape".to_string()));
                        }
                        let hex = &self.input[self.pos..self.pos + 4];
                        self.pos += 4;
                        u32::from_str_radix(hex, 16)
                            .map_err(|_| YamlError::ParseError("Invalid Unicode escape".to_string()))?
                            as char
                    }
                    _ => return Err(YamlError::ParseError("Invalid escape sequence".to_string())),
                };
                result.push(decoded);
            } else {
                result.push(c as char);
            }
        }

        Err(YamlError::ParseError("Unclosed string".to_string()))
    }

    fn parse_single_quoted_string(&mut self) -> YamlResult<YamlValue> {
        self.pos += 1;
        let mut result = String::new();

        while self.pos < self.input.len() {
            let c = self.input.as_bytes()[self.pos];
            self.pos += 1;

            if c == b'\'' {
                if self.pos < self.input.len() && self.input.as_bytes()[self.pos] == b'\'' {
                    self.pos += 1;
                    result.push('\'');
                } else {
                    return Ok(YamlValue::String(result));
                }
            } else {
                result.push(c as char);
            }
        }

        Err(YamlError::ParseError("Unclosed string".to_string()))
    }

    fn parse_sequence(&mut self) -> YamlResult<YamlValue> {
        self.pos += 1;
        self.skip_whitespace();

        let mut seq = Vec::new();

        if self.pos < self.input.len() && self.input.as_bytes()[self.pos] != b']' {
            loop {
                self.skip_whitespace();
                seq.push(self.parse_value()?);
                self.skip_whitespace();

                if self.pos >= self.input.len() {
                    return Err(YamlError::ParseError("Unclosed sequence".to_string()));
                }

                let c = self.input.as_bytes()[self.pos];
                self.pos += 1;

                if c == b']' {
                    break;
                } else if c != b',' {
                    return Err(YamlError::ParseError("Expected ',' or ']'".to_string()));
                }
            }
        } else {
            self.pos += 1;
        }

        Ok(YamlValue::Sequence(seq))
    }

    fn parse_mapping(&mut self) -> YamlResult<YamlValue> {
        self.pos += 1;
        self.skip_whitespace();

        let mut map = HashMap::new();

        if self.pos < self.input.len() && self.input.as_bytes()[self.pos] != b'}' {
            loop {
                self.skip_whitespace();

                if self.pos >= self.input.len() {
                    return Err(YamlError::ParseError("Unclosed mapping".to_string()));
                }

                let key = match self.parse_value()? {
                    YamlValue::String(s) => s,
                    _ => return Err(YamlError::ParseError("Mapping key must be a string".to_string())),
                };

                self.skip_whitespace();

                if self.pos >= self.input.len() || self.input.as_bytes()[self.pos] != b':' {
                    return Err(YamlError::ParseError("Expected ':'".to_string()));
                }
                self.pos += 1;

                self.skip_whitespace();
                let value = self.parse_value()?;
                map.insert(key, value);

                self.skip_whitespace();

                if self.pos >= self.input.len() {
                    return Err(YamlError::ParseError("Unclosed mapping".to_string()));
                }

                let c = self.input.as_bytes()[self.pos];
                self.pos += 1;

                if c == b'}' {
                    break;
                } else if c != b',' {
                    return Err(YamlError::ParseError("Expected ',' or '}'".to_string()));
                }
            }
        } else {
            self.pos += 1;
        }

        Ok(YamlValue::Mapping(map))
    }

    fn parse_block_scalar(&mut self) -> YamlResult<YamlValue> {
        self.pos += 1;
        self.skip_whitespace();

        let mut result = String::new();

        while self.pos < self.input.len() {
            let c = self.input.as_bytes()[self.pos];

            if c == b'\n' || c == b'\r' {
                self.pos += 1;
                self.skip_whitespace();

                if self.pos >= self.input.len() {
                    break;
                }

                let next_c = self.input.as_bytes()[self.pos];
                if next_c.is_ascii_whitespace() || next_c == b'\n' || next_c == b'\r' {
                    break;
                }

                if !result.is_empty() {
                    result.push('\n');
                }
            } else {
                result.push(c as char);
                self.pos += 1;
            }
        }

        Ok(YamlValue::String(result))
    }

    fn parse_anchor(&mut self) -> YamlResult<YamlValue> {
        self.pos += 1;
        let start = self.pos;

        while self.pos < self.input.len() {
            let c = self.input.as_bytes()[self.pos];
            if c.is_ascii_whitespace() || c == b':' {
                break;
            }
            self.pos += 1;
        }

        let name = self.input[start..self.pos].to_string();
        let value = self.parse_value()?;
        self.anchors.insert(name.clone(), value.clone());
        Ok(value)
    }

    fn parse_alias(&mut self) -> YamlResult<YamlValue> {
        self.pos += 1;
        let start = self.pos;

        while self.pos < self.input.len() {
            let c = self.input.as_bytes()[self.pos];
            if c.is_ascii_whitespace() || c == b':' {
                break;
            }
            self.pos += 1;
        }

        let name = self.input[start..self.pos].to_string();
        Ok(YamlValue::Alias(name))
    }

    fn parse_directive(&mut self) -> YamlResult<YamlValue> {
        self.pos += 1;
        self.skip_to_next_line();
        self.parse_value()
    }

    fn parse_plain_scalar(&mut self) -> YamlResult<YamlValue> {
        let start = self.pos;

        while self.pos < self.input.len() {
            let c = self.input.as_bytes()[self.pos];
            if c.is_ascii_whitespace() || c == b':' || c == b'#' {
                break;
            }
            self.pos += 1;
        }

        let value = self.input[start..self.pos].to_string();

        if value.is_empty() {
            return Ok(YamlValue::Null);
        }

        if value == "null" || value == "~" {
            return Ok(YamlValue::Null);
        }

        if value == "true" {
            return Ok(YamlValue::Bool(true));
        }

        if value == "false" {
            return Ok(YamlValue::Bool(false));
        }

        if let Ok(i) = value.parse::<i64>() {
            return Ok(YamlValue::Integer(i));
        }

        if let Ok(f) = value.parse::<f64>() {
            return Ok(YamlValue::Float(f));
        }

        Ok(YamlValue::String(value))
    }
}

pub fn parse<R: Read>(reader: &mut R) -> YamlResult<YamlDocument> {
    YamlParser::new().parse(reader)
}

pub fn parse_str(content: &str) -> YamlResult<YamlDocument> {
    YamlParser::new().parse_str(content)
}

pub fn to_string(value: &YamlValue) -> YamlResult<String> {
    let doc = YamlDocument { root: value.clone(), anchors: HashMap::new() };
    doc.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_yaml_value() {
        let value = YamlValue::string("hello".to_string());
        assert_eq!(value.as_string(), Some(&"hello".to_string()));
    }

    #[test]
    fn test_yaml_document() {
        let mut doc = YamlDocument::new();
        let mut map = HashMap::new();
        map.insert("name".to_string(), YamlValue::string("test".to_string()));
        map.insert("value".to_string(), YamlValue::int(42));
        doc.set_root(YamlValue::mapping(map));

        let yaml = doc.to_string().unwrap();
        assert!(yaml.contains("name"));
        assert!(yaml.contains("test"));
    }

    #[test]
    fn test_yaml_parser() {
        let yaml = "name: test\nvalue: 42";
        let doc = parse_str(yaml).unwrap();
        let root = doc.get_root();
        assert_eq!(root.get("name").and_then(|v| v.as_string()), Some(&"test".to_string()));
        assert_eq!(root.get("value").and_then(|v| v.as_i64()), Some(42));
    }

    #[test]
    fn test_yaml_sequence() {
        let yaml = "- item1\n- item2\n- item3";
        let doc = parse_str(yaml).unwrap();
        let seq = doc.get_root().as_sequence().unwrap();
        assert_eq!(seq.len(), 3);
        assert_eq!(seq[0].as_string(), Some(&"item1".to_string()));
    }

    #[test]
    fn test_yaml_null() {
        let yaml = "value: ~";
        let doc = parse_str(yaml).unwrap();
        let root = doc.get_root();
        assert!(root.get("value").map(|v| v.is_null()).unwrap_or(false));
    }
}
