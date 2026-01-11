use std::collections::HashMap;
use std::io::{self, Read, Write};

pub type XmlResult<T> = Result<T, XmlError>;

#[derive(Debug, Clone)]
pub enum XmlError {
    ParseError(String),
    WriteError(String),
    InvalidFormat(String),
    MissingAttribute(String),
    InvalidValue(String),
}

impl std::fmt::Display for XmlError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            XmlError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            XmlError::WriteError(msg) => write!(f, "Write error: {}", msg),
            XmlError::InvalidFormat(msg) => write!(f, "Invalid format: {}", msg),
            XmlError::MissingAttribute(attr) => {
                write!(f, "Missing attribute: {}", attr)
            }
            XmlError::InvalidValue(msg) => write!(f, "Invalid value: {}", msg),
        }
    }
}

impl std::error::Error for XmlError {}

#[derive(Debug, Clone, PartialEq)]
pub enum XmlNodeType {
    Element,
    Text,
    Comment,
    CData,
    ProcessingInstruction,
    DocumentType,
}

#[derive(Debug, Clone)]
pub struct XmlNode {
    pub node_type: XmlNodeType,
    pub name: Option<String>,
    pub attributes: HashMap<String, String>,
    pub children: Vec<XmlNode>,
    pub text: Option<String>,
}

impl XmlNode {
    pub fn new(node_type: XmlNodeType) -> Self {
        XmlNode {
            node_type,
            name: None,
            attributes: HashMap::new(),
            children: Vec::new(),
            text: None,
        }
    }

    pub fn element(name: String) -> Self {
        let mut node = XmlNode::new(XmlNodeType::Element);
        node.name = Some(name);
        node
    }

    pub fn text(content: String) -> Self {
        let mut node = XmlNode::new(XmlNodeType::Text);
        node.text = Some(content);
        node
    }

    pub fn comment(content: String) -> Self {
        let mut node = XmlNode::new(XmlNodeType::Comment);
        node.text = Some(content);
        node
    }

    pub fn cdata(content: String) -> Self {
        let mut node = XmlNode::new(XmlNodeType::CData);
        node.text = Some(content);
        node
    }

    pub fn set_attribute(&mut self, name: String, value: String) {
        self.attributes.insert(name, value);
    }

    pub fn get_attribute(&self, name: &str) -> Option<&String> {
        self.attributes.get(name)
    }

    pub fn add_child(&mut self, child: XmlNode) {
        self.children.push(child);
    }

    pub fn get_children(&self) -> &[XmlNode] {
        &self.children
    }

    pub fn get_children_by_name(&self, name: &str) -> Vec<&XmlNode> {
        self.children
            .iter()
            .filter(|node| node.name.as_ref().map_or(false, |n| n == name))
            .collect()
    }

    pub fn get_first_child_by_name(&self, name: &str) -> Option<&XmlNode> {
        self.get_children_by_name(name).first().copied()
    }

    pub fn get_text(&self) -> Option<&String> {
        self.text.as_ref()
    }

    pub fn get_text_content(&self) -> String {
        self.children
            .iter()
            .filter_map(|node| node.get_text())
            .collect::<Vec<_>>()
            .join("")
    }
}

pub struct XmlDocument {
    pub root: Option<XmlNode>,
    pub version: String,
    pub encoding: String,
    pub standalone: bool,
}

impl XmlDocument {
    pub fn new() -> Self {
        XmlDocument {
            root: None,
            version: "1.0".to_string(),
            encoding: "UTF-8".to_string(),
            standalone: true,
        }
    }

    pub fn set_root(&mut self, root: XmlNode) {
        self.root = Some(root);
    }

    pub fn get_root(&self) -> Option<&XmlNode> {
        self.root.as_ref()
    }

    pub fn to_string(&self) -> XmlResult<String> {
        let mut output = String::new();
        self.write(&mut output)?;
        Ok(output)
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> XmlResult<()> {
        writeln!(
            writer,
            r#"<?xml version="{}" encoding="{}"{}?>"#,
            self.version,
            self.encoding,
            if self.standalone { "" } else { " standalone=\"yes\"" }
        )?;

        if let Some(ref root) = self.root {
            self.write_node(writer, root, 0)?;
        }

        Ok(())
    }

    fn write_node<W: Write>(&self, writer: &mut W, node: &XmlNode, indent: usize) -> XmlResult<()> {
        let indent_str = " ".repeat(indent);

        match node.node_type {
            XmlNodeType::Element => {
                if let Some(ref name) = node.name {
                    write!(writer, "{}<{}", indent_str, name)?;

                    for (attr_name, attr_value) in &node.attributes {
                        write!(writer, " {}=\"{}\"", attr_name, attr_value)?;
                    }

                    if node.children.is_empty() {
                        if let Some(ref text) = node.text {
                            write!(writer, ">{}</{}>\n", text, name)?;
                        } else {
                            writeln!(writer, " />")?;
                        }
                    } else {
                        writeln!(writer, ">")?;

                        for child in &node.children {
                            self.write_node(writer, child, indent + 2)?;
                        }

                        writeln!(writer, "{}</{}>\n", indent_str, name)?;
                    }
                }
            }
            XmlNodeType::Text => {
                if let Some(ref text) = node.text {
                    write!(writer, "{}", text)?;
                }
            }
            XmlNodeType::Comment => {
                if let Some(ref text) = node.text {
                    writeln!(writer, "{}<!-- {} -->", indent_str, text)?;
                }
            }
            XmlNodeType::CData => {
                if let Some(ref text) = node.text {
                    writeln!(writer, "{}<![CDATA[ {} ]]>", indent_str, text)?;
                }
            }
            XmlNodeType::ProcessingInstruction => {
                if let Some(ref text) = node.text {
                    writeln!(writer, "{}<?{} ?>", indent_str, text)?;
                }
            }
            XmlNodeType::DocumentType => {
                if let Some(ref text) = node.text {
                    writeln!(writer, "{}<!DOCTYPE {}>", indent_str, text)?;
                }
            }
        }

        Ok(())
    }
}

impl Default for XmlDocument {
    fn default() -> Self {
        Self::new()
    }
}

pub struct XmlParser;

impl XmlParser {
    pub fn new() -> Self {
        XmlParser
    }

    pub fn parse<R: Read>(&self, reader: &mut R) -> XmlResult<XmlDocument> {
        let mut content = String::new();
        reader.read_to_string(&mut content)
            .map_err(|e| XmlError::ParseError(e.to_string()))?;

        self.parse_str(&content)
    }

    pub fn parse_str(&self, content: &str) -> XmlResult<XmlDocument> {
        let mut document = XmlDocument::new();

        let lines: Vec<&str> = content.lines().collect();
        let mut current_node: Option<XmlNode> = None;
        let mut node_stack: Vec<XmlNode> = Vec::new();

        for line in lines {
            let line = line.trim();

            if line.starts_with("<?xml") {
                continue;
            }

            if line.starts_with("<!--") {
                let comment = line
                    .strip_prefix("<!--")
                    .and_then(|s| s.strip_suffix("-->"))
                    .unwrap_or("");
                let comment_node = XmlNode::comment(comment.to_string());
                if let Some(ref mut parent) = current_node {
                    parent.add_child(comment_node);
                }
                continue;
            }

            if line.starts_with("<![CDATA[") {
                let cdata = line
                    .strip_prefix("<![CDATA[")
                    .and_then(|s| s.strip_suffix("]]>"))
                    .unwrap_or("");
                let cdata_node = XmlNode::cdata(cdata.to_string());
                if let Some(ref mut parent) = current_node {
                    parent.add_child(cdata_node);
                }
                continue;
            }

            if line.starts_with("<!") {
                let doctype = line
                    .strip_prefix("<!")
                    .and_then(|s| s.strip_suffix(">"))
                    .unwrap_or("");
                let doctype_node = XmlNode::new(XmlNodeType::DocumentType);
                doctype_node.text = Some(doctype.to_string());
                if let Some(ref mut parent) = current_node {
                    parent.add_child(doctype_node);
                }
                continue;
            }

            if line.starts_with("</") {
                let closing = line
                    .strip_prefix("</")
                    .and_then(|s| s.strip_suffix(">"))
                    .unwrap_or("");
                if let Some(node) = node_stack.pop() {
                    current_node = Some(node);
                }
                continue;
            }

            if line.starts_with("<") && !line.starts_with("</") {
                let tag_content = line
                    .strip_prefix("<")
                    .and_then(|s| s.strip_suffix(">"))
                    .unwrap_or("");

                let mut parts = tag_content.split_whitespace();
                let name = parts.next().unwrap_or("");

                let mut node = XmlNode::element(name.to_string());

                for part in parts {
                    if let Some((attr_name, attr_value)) = part.split_once('=') {
                        let value = attr_value.trim_matches('"');
                        node.set_attribute(attr_name.to_string(), value.to_string());
                    }
                }

                if let Some(ref mut parent) = current_node {
                    parent.add_child(node.clone());
                    node_stack.push(parent.clone());
                }

                if document.root.is_none() {
                    document.set_root(node.clone());
                }

                current_node = Some(node);
                continue;
            }

            if !line.starts_with("<") {
                if let Some(ref mut parent) = current_node {
                    let text_node = XmlNode::text(line.to_string());
                    parent.add_child(text_node);
                }
            }
        }

        Ok(document)
    }
}

impl Default for XmlParser {
    fn default() -> Self {
        Self::new()
    }
}

pub fn parse<R: Read>(reader: &mut R) -> XmlResult<XmlDocument> {
    XmlParser::new().parse(reader)
}

pub fn parse_str(content: &str) -> XmlResult<XmlDocument> {
    XmlParser::new().parse_str(content)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xml_node() {
        let mut node = XmlNode::element("root");
        node.set_attribute("id".to_string(), "1".to_string());
        node.add_child(XmlNode::text("Hello".to_string()));

        assert_eq!(node.name, Some("root".to_string()));
        assert_eq!(node.get_attribute("id"), Some(&"1".to_string()));
        assert_eq!(node.children.len(), 1);
    }

    #[test]
    fn test_xml_document() {
        let mut doc = XmlDocument::new();
        let mut root = XmlNode::element("root");
        root.set_attribute("version".to_string(), "1.0".to_string());
        root.add_child(XmlNode::text("Hello, World!".to_string()));
        doc.set_root(root);

        let xml = doc.to_string().unwrap();
        assert!(xml.contains("<?xml"));
        assert!(xml.contains("<root"));
        assert!(xml.contains("Hello, World!"));
    }

    #[test]
    fn test_xml_parser() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<root id="1">
    <child>Text</child>
</root>"#;

        let doc = parse_str(xml).unwrap();
        assert!(doc.get_root().is_some());
        let root = doc.get_root().unwrap();
        assert_eq!(root.name, Some("root".to_string()));
        assert_eq!(root.get_attribute("id"), Some(&"1".to_string()));
    }

    #[test]
    fn test_xml_comment() {
        let mut doc = XmlDocument::new();
        let mut root = XmlNode::element("root");
        root.add_child(XmlNode::comment("This is a comment".to_string()));
        doc.set_root(root);

        let xml = doc.to_string().unwrap();
        assert!(xml.contains("<!--"));
    }

    #[test]
    fn test_xml_cdata() {
        let mut doc = XmlDocument::new();
        let mut root = XmlNode::element("root");
        root.add_child(XmlNode::cdata("Some data".to_string()));
        doc.set_root(root);

        let xml = doc.to_string().unwrap();
        assert!(xml.contains("<![CDATA["));
    }
}
