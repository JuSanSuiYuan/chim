use std::collections::HashMap;
use std::io::{self, Read, Write};

pub type TomlResult<T> = Result<T, TomlError>;

#[derive(Debug, Clone)]
pub enum TomlError {
    ParseError(String),
    WriteError(String),
    InvalidFormat(String),
    MissingKey(String),
    InvalidValue(String),
}

impl std::fmt::Display for TomlError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TomlError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            TomlError::WriteError(msg) => write!(f, "Write error: {}", msg),
            TomlError::InvalidFormat(msg) => write!(f, "Invalid format: {}", msg),
            TomlError::MissingKey(key) => write!(f, "Missing key: {}", key),
            TomlError::InvalidValue(msg) => write!(f, "Invalid value: {}", msg),
        }
    }
}

impl std::error::Error for TomlError {}

#[derive(Debug, Clone, PartialEq)]
pub enum TomlValue {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Array(Vec<TomlValue>),
    Table(HashMap<String, TomlValue>),
    InlineTable(HashMap<String, TomlValue>),
    DateTime(String),
    None,
}

impl TomlValue {
    pub fn as_str(&self) -> Option<&str> {
        match self {
            TomlValue::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_i64(&self) -> Option<i64> {
        match self {
            TomlValue::Integer(i) => Some(*i),
            _ => None,
        }
    }

    pub fn as_f64(&self) -> Option<f64> {
        match self {
            TomlValue::Float(f) => Some(*f),
            TomlValue::Integer(i) => Some(*i as f64),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            TomlValue::Boolean(b) => Some(*b),
            _ => None,
        }
    }

    pub fn as_array(&self) -> Option<&Vec<TomlValue>> {
        match self {
            TomlValue::Array(arr) => Some(arr),
            _ => None,
        }
    }

    pub fn as_table(&self) -> Option<&HashMap<String, TomlValue>> {
        match self {
            TomlValue::Table(table) => Some(table),
            TomlValue::InlineTable(table) => Some(table),
            _ => None,
        }
    }

    pub fn is_none(&self) -> bool {
        matches!(self, TomlValue::None)
    }

    pub fn get(&self, key: &str) -> Option<&TomlValue> {
        match self {
            TomlValue::Table(table) | TomlValue::InlineTable(table) => table.get(key),
            _ => None,
        }
    }
}

impl From<String> for TomlValue {
    fn from(s: String) -> Self {
        TomlValue::String(s)
    }
}

impl From<i64> for TomlValue {
    fn from(i: i64) -> Self {
        TomlValue::Integer(i)
    }
}

impl From<f64> for TomlValue {
    fn from(f: f64) -> Self {
        TomlValue::Float(f)
    }
}

impl From<bool> for TomlValue {
    fn from(b: bool) -> Self {
        TomlValue::Boolean(b)
    }
}

impl From<Vec<TomlValue>> for TomlValue {
    fn from(arr: Vec<TomlValue>) -> Self {
        TomlValue::Array(arr)
    }
}

impl From<HashMap<String, TomlValue>> for TomlValue {
    fn from(table: HashMap<String, TomlValue>) -> Self {
        TomlValue::Table(table)
    }
}

pub struct TomlDocument {
    pub values: HashMap<String, TomlValue>,
}

impl TomlDocument {
    pub fn new() -> Self {
        TomlDocument {
            values: HashMap::new(),
        }
    }

    pub fn set(&mut self, key: String, value: TomlValue) {
        self.values.insert(key, value);
    }

    pub fn get(&self, key: &str) -> Option<&TomlValue> {
        self.values.get(key)
    }

    pub fn get_str(&self, key: &str) -> TomlResult<&str> {
        self.get(key)
            .and_then(|v| v.as_str())
            .ok_or_else(|| TomlError::MissingKey(key.to_string()))
    }

    pub fn get_i64(&self, key: &str) -> TomlResult<i64> {
        self.get(key)
            .and_then(|v| v.as_i64())
            .ok_or_else(|| TomlError::MissingKey(key.to_string()))
    }

    pub fn get_f64(&self, key: &str) -> TomlResult<f64> {
        self.get(key)
            .and_then(|v| v.as_f64())
            .ok_or_else(|| TomlError::MissingKey(key.to_string()))
    }

    pub fn get_bool(&self, key: &str) -> TomlResult<bool> {
        self.get(key)
            .and_then(|v| v.as_bool())
            .ok_or_else(|| TomlError::MissingKey(key.to_string()))
    }

    pub fn get_array(&self, key: &str) -> TomlResult<&Vec<TomlValue>> {
        self.get(key)
            .and_then(|v| v.as_array())
            .ok_or_else(|| TomlError::MissingKey(key.to_string()))
    }

    pub fn get_table(&self, key: &str) -> TomlResult<&HashMap<String, TomlValue>> {
        self.get(key)
            .and_then(|v| v.as_table())
            .ok_or_else(|| TomlError::MissingKey(key.to_string()))
    }

    pub fn to_string(&self) -> TomlResult<String> {
        let mut output = String::new();
        self.write(&mut output)?;
        Ok(output)
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> TomlResult<()> {
        for (key, value) in &self.values {
            self.write_value(writer, key, value, 0)?;
        }
        Ok(())
    }

    fn write_value<W: Write>(
        &self,
        writer: &mut W,
        key: &str,
        value: &TomlValue,
        indent: usize,
    ) -> TomlResult<()> {
        let indent_str = " ".repeat(indent);

        match value {
            TomlValue::String(s) => {
                writeln!(writer, "{}{} = \"{}\"", indent_str, key, s)?;
            }
            TomlValue::Integer(i) => {
                writeln!(writer, "{}{} = {}", indent_str, key, i)?;
            }
            TomlValue::Float(f) => {
                writeln!(writer, "{}{} = {}", indent_str, key, f)?;
            }
            TomlValue::Boolean(b) => {
                writeln!(writer, "{}{} = {}", indent_str, key, b)?;
            }
            TomlValue::Array(arr) => {
                writeln!(writer, "{}{} = [", indent_str, key)?;
                for item in arr {
                    self.write_value(writer, "", item, indent + 2)?;
                }
                writeln!(writer, "{}]", indent_str)?;
            }
            TomlValue::Table(table) => {
                writeln!(writer, "{}[{}]", indent_str, key)?;
                for (sub_key, sub_value) in table {
                    self.write_value(writer, sub_key, sub_value, indent + 2)?;
                }
            }
            TomlValue::InlineTable(table) => {
                write!(writer, "{}{} = {{", indent_str, key)?;
                let mut first = true;
                for (sub_key, sub_value) in table {
                    if !first {
                        write!(writer, ", ")?;
                    }
                    first = false;
                    write!(writer, "{} = ", sub_key)?;
                    self.write_inline_value(writer, sub_value)?;
                }
                writeln!(writer, "}}")?;
            }
            TomlValue::DateTime(dt) => {
                writeln!(writer, "{}{} = {}", indent_str, key, dt)?;
            }
            TomlValue::None => {}
        }

        Ok(())
    }

    fn write_inline_value<W: Write>(&self, writer: &mut W, value: &TomlValue) -> TomlResult<()> {
        match value {
            TomlValue::String(s) => {
                write!(writer, "\"{}\"", s)?;
            }
            TomlValue::Integer(i) => {
                write!(writer, "{}", i)?;
            }
            TomlValue::Float(f) => {
                write!(writer, "{}", f)?;
            }
            TomlValue::Boolean(b) => {
                write!(writer, "{}", b)?;
            }
            TomlValue::Array(arr) => {
                write!(writer, "[")?;
                for (i, item) in arr.iter().enumerate() {
                    if i > 0 {
                        write!(writer, ", ")?;
                    }
                    self.write_inline_value(writer, item)?;
                }
                write!(writer, "]")?;
            }
            TomlValue::Table(table) | TomlValue::InlineTable(table) => {
                write!(writer, "{{")?;
                let mut first = true;
                for (sub_key, sub_value) in table {
                    if !first {
                        write!(writer, ", ")?;
                    }
                    first = false;
                    write!(writer, "{} = ", sub_key)?;
                    self.write_inline_value(writer, sub_value)?;
                }
                write!(writer, "}}")?;
            }
            TomlValue::DateTime(dt) => {
                write!(writer, "{}", dt)?;
            }
            TomlValue::None => {}
        }

        Ok(())
    }
}

impl Default for TomlDocument {
    fn default() -> Self {
        Self::new()
    }
}

pub struct TomlParser;

impl TomlParser {
    pub fn new() -> Self {
        TomlParser
    }

    pub fn parse<R: Read>(&self, reader: &mut R) -> TomlResult<TomlDocument> {
        let mut content = String::new();
        reader.read_to_string(&mut content)
            .map_err(|e| TomlError::ParseError(e.to_string()))?;

        self.parse_str(&content)
    }

    pub fn parse_str(&self, content: &str) -> TomlResult<TomlDocument> {
        let mut document = TomlDocument::new();
        let mut current_table: Option<HashMap<String, TomlValue>> = None;
        let mut current_array: Option<Vec<TomlValue>> = None;

        for line in content.lines() {
            let line = line.trim();

            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if line.starts_with('[') {
                if let Some(table_name) = line.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
                    let table = if table_name.contains('.') {
                        let parts: Vec<&str> = table_name.split('.').collect();
                        let mut table = HashMap::new();
                        let mut current = &mut document.values;

                        for (i, part) in parts.iter().enumerate() {
                            if i == parts.len() - 1 {
                                let sub_table = HashMap::new();
                                current.insert(part.to_string(), TomlValue::Table(sub_table.clone()));
                                current_table = Some(sub_table);
                            } else {
                                if !current.contains_key(*part) {
                                    current.insert(part.to_string(), TomlValue::Table(HashMap::new()));
                                }
                                if let Some(TomlValue::Table(ref sub)) = current.get(*part) {
                                    current = sub;
                                }
                            }
                        }
                        table
                    } else {
                        let table = HashMap::new();
                        document.set(table_name.to_string(), TomlValue::Table(table.clone()));
                        table
                    };
                    current_table = Some(table);
                    current_array = None;
                }
                continue;
            }

            if line.starts_with("[[") {
                if let Some(array_name) = line.strip_prefix("[[").and_then(|s| s.strip_suffix("]]")) {
                    let array = Vec::new();
                    document.set(array_name.to_string(), TomlValue::Array(array.clone()));
                    current_array = Some(array);
                    current_table = None;
                }
                continue;
            }

            if let Some((key, value_str)) = line.split_once('=') {
                let key = key.trim();
                let value = self.parse_value(value_str.trim())?;

                if let Some(ref mut table) = current_table {
                    table.insert(key.to_string(), value);
                } else if let Some(ref mut array) = current_array {
                    let mut table = HashMap::new();
                    table.insert(key.to_string(), value);
                    array.push(TomlValue::Table(table));
                } else {
                    document.set(key.to_string(), value);
                }
            }
        }

        Ok(document)
    }

    fn parse_value(&self, s: &str) -> TomlResult<TomlValue> {
        if s.starts_with('"') && s.ends_with('"') {
            let content = &s[1..s.len() - 1];
            return Ok(TomlValue::String(content.to_string()));
        }

        if s.starts_with('\'') && s.ends_with('\'') {
            let content = &s[1..s.len() - 1];
            return Ok(TomlValue::String(content.to_string()));
        }

        if s == "true" {
            return Ok(TomlValue::Boolean(true));
        }

        if s == "false" {
            return Ok(TomlValue::Boolean(false));
        }

        if let Ok(i) = s.parse::<i64>() {
            return Ok(TomlValue::Integer(i));
        }

        if let Ok(f) = s.parse::<f64>() {
            return Ok(TomlValue::Float(f));
        }

        if s.starts_with('[') && s.ends_with(']') {
            let content = &s[1..s.len() - 1];
            let mut array = Vec::new();

            for item in content.split(',') {
                let item = item.trim();
                if !item.is_empty() {
                    array.push(self.parse_value(item)?);
                }
            }

            return Ok(TomlValue::Array(array));
        }

        if s.starts_with('{') && s.ends_with('}') {
            let content = &s[1..s.len() - 1];
            let mut table = HashMap::new();

            for item in content.split(',') {
                let item = item.trim();
                if let Some((key, value)) = item.split_once('=') {
                    let key = key.trim();
                    let value = self.parse_value(value.trim())?;
                    table.insert(key.to_string(), value);
                }
            }

            return Ok(TomlValue::InlineTable(table));
        }

        Err(TomlError::InvalidValue(format!("Cannot parse value: {}", s)))
    }
}

impl Default for TomlParser {
    fn default() -> Self {
        Self::new()
    }
}

pub fn parse<R: Read>(reader: &mut R) -> TomlResult<TomlDocument> {
    TomlParser::new().parse(reader)
}

pub fn parse_str(content: &str) -> TomlResult<TomlDocument> {
    TomlParser::new().parse_str(content)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_toml_value() {
        let value = TomlValue::String("hello".to_string());
        assert_eq!(value.as_str(), Some("hello"));
        assert_eq!(value.as_i64(), None);
    }

    #[test]
    fn test_toml_document() {
        let mut doc = TomlDocument::new();
        doc.set("name".to_string(), TomlValue::String("test".to_string()));
        doc.set("version".to_string(), TomlValue::Float(1.0));

        assert_eq!(doc.get_str("name").unwrap(), "test");
        assert_eq!(doc.get_f64("version").unwrap(), 1.0);
    }

    #[test]
    fn test_toml_parser() {
        let toml = r#"
name = "test"
version = 1.0
enabled = true

[section]
key = "value"
"#;

        let doc = parse_str(toml).unwrap();
        assert_eq!(doc.get_str("name").unwrap(), "test");
        assert_eq!(doc.get_f64("version").unwrap(), 1.0);
        assert_eq!(doc.get_bool("enabled").unwrap(), true);
        assert!(doc.get_table("section").is_some());
    }

    #[test]
    fn test_toml_array() {
        let toml = r#"
items = ["a", "b", "c"]
"#;

        let doc = parse_str(toml).unwrap();
        let arr = doc.get_array("items").unwrap();
        assert_eq!(arr.len(), 3);
        assert_eq!(arr[0].as_str(), Some("a"));
    }

    #[test]
    fn test_toml_inline_table() {
        let toml = r#"
[section]
name = { first = "John", last = "Doe" }
"#;

        let doc = parse_str(toml).unwrap();
        let table = doc.get_table("section").unwrap();
        let inline = table.get("name").and_then(|v| v.as_table()).unwrap();
        assert_eq!(inline.get_str("first").unwrap(), "John");
    }

    #[test]
    fn test_toml_write() {
        let mut doc = TomlDocument::new();
        doc.set("name".to_string(), TomlValue::String("test".to_string()));
        doc.set("version".to_string(), TomlValue::Float(1.0));

        let toml = doc.to_string().unwrap();
        assert!(toml.contains("name = \"test\""));
        assert!(toml.contains("version = 1"));
    }
}
