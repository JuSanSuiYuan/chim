use crate::ast::{self, ASTNode};
use std::collections::HashMap;
use std::sync::Arc;

pub type MacroResult<T> = Result<T, MacroError>;

#[derive(Debug, Clone)]
pub enum MacroError {
    ParseError(String),
    ExpansionError(String),
    TypeCheckError(String),
    NameConflict(String),
    InvalidInput(String),
}

impl std::fmt::Display for MacroError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MacroError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            MacroError::ExpansionError(msg) => write!(f, "Expansion error: {}", msg),
            MacroError::TypeCheckError(msg) => write!(f, "Type check error: {}", msg),
            MacroError::NameConflict(msg) => write!(f, "Name conflict: {}", msg),
            MacroError::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
        }
    }
}

impl std::error::Error for MacroError {}

pub trait ProceduralMacro {
    fn name(&self) -> &str;
    fn expand(&self, input: &ASTNode) -> MacroResult<ASTNode>;
    fn validate(&self, input: &ASTNode) -> MacroResult<()> {
        Ok(())
    }
}

pub struct MacroRegistry {
    macros: HashMap<String, Arc<dyn ProceduralMacro + Send + Sync>>,
}

impl MacroRegistry {
    pub fn new() -> Self {
        MacroRegistry {
            macros: HashMap::new(),
        }
    }

    pub fn register(&mut self, mac: Arc<dyn ProceduralMacro + Send + Sync>) {
        self.macros.insert(mac.name().to_string(), mac);
    }

    pub fn get(&self, name: &str) -> Option<Arc<dyn ProceduralMacro + Send + Sync>> {
        self.macros.get(name).cloned()
    }

    pub fn expand(&self, name: &str, input: &ASTNode) -> MacroResult<ASTNode> {
        let mac = self.get(name).ok_or_else(|| {
            MacroError::InvalidInput(format!("Macro '{}' not found", name))
        })?;
        mac.validate(input)?;
        mac.expand(input)
    }
}

pub struct DeriveMacro {
    name: String,
    target: DeriveTarget,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DeriveTarget {
    Struct,
    Enum,
    Union,
}

impl DeriveMacro {
    pub fn new(name: String, target: DeriveTarget) -> Self {
        DeriveMacro { name, target }
    }
}

impl ProceduralMacro for DeriveMacro {
    fn name(&self) -> &str {
        &self.name
    }

    fn expand(&self, input: &ASTNode) -> MacroResult<ASTNode> {
        match input {
            ASTNode::StructDef(struct_def) => {
                if self.target != DeriveTarget::Struct {
                    return Err(MacroError::InvalidInput(
                        "Derive macro expects a struct".to_string(),
                    ));
                }
                self.expand_struct(struct_def)
            }
            ASTNode::EnumDef(enum_def) => {
                if self.target != DeriveTarget::Enum {
                    return Err(MacroError::InvalidInput(
                        "Derive macro expects an enum".to_string(),
                    ));
                }
                self.expand_enum(enum_def)
            }
            _ => Err(MacroError::InvalidInput(
                "Derive macro expects a struct or enum".to_string(),
            )),
        }
    }
}

impl DeriveMacro {
    fn expand_struct(&self, struct_def: &ast::StructDef) -> MacroResult<ASTNode> {
        match self.name.as_str() {
            "Clone" => self.derive_clone_struct(struct_def),
            "Copy" => self.derive_copy_struct(struct_def),
            "Debug" => self.derive_debug_struct(struct_def),
            "PartialEq" => self.derive_partialeq_struct(struct_def),
            "Eq" => self.derive_eq_struct(struct_def),
            "Hash" => self.derive_hash_struct(struct_def),
            "Serialize" => self.derive_serialize_struct(struct_def),
            "Deserialize" => self.derive_deserialize_struct(struct_def),
            _ => Err(MacroError::InvalidInput(format!(
                "Unknown derive macro: {}",
                self.name
            ))),
        }
    }

    fn expand_enum(&self, enum_def: &ast::EnumDef) -> MacroResult<ASTNode> {
        match self.name.as_str() {
            "Clone" => self.derive_clone_enum(enum_def),
            "Copy" => self.derive_copy_enum(enum_def),
            "Debug" => self.derive_debug_enum(enum_def),
            "PartialEq" => self.derive_partialeq_enum(enum_def),
            "Eq" => self.derive_eq_enum(enum_def),
            "Hash" => self.derive_hash_enum(enum_def),
            "Serialize" => self.derive_serialize_enum(enum_def),
            "Deserialize" => self.derive_deserialize_enum(enum_def),
            _ => Err(MacroError::InvalidInput(format!(
                "Unknown derive macro: {}",
                self.name
            ))),
        }
    }

    fn derive_clone_struct(&self, struct_def: &ast::StructDef) -> MacroResult<ASTNode> {
        let impl_code = format!(
            r#"
impl Clone for {} {{
    fn clone(&self) -> Self {{
        Self {{
            {}
        }}
    }}
}}
"#,
            struct_def.name,
            struct_def
                .fields
                .iter()
                .map(|f| format!("{}: self.{}.clone()", f.name, f.name))
                .collect::<Vec<_>>()
                .join(",\n            ")
        );

        Ok(ASTNode::ImplBlock(ast::ImplBlock {
            trait_name: Some("Clone".to_string()),
            for_type: struct_def.name.clone(),
            methods: vec![],
            body: impl_code,
        }))
    }

    fn derive_copy_struct(&self, struct_def: &ast::StructDef) -> MacroResult<ASTNode> {
        let impl_code = format!(
            r#"
impl Copy for {} {{
}}
"#,
            struct_def.name
        );

        Ok(ASTNode::ImplBlock(ast::ImplBlock {
            trait_name: Some("Copy".to_string()),
            for_type: struct_def.name.clone(),
            methods: vec![],
            body: impl_code,
        }))
    }

    fn derive_debug_struct(&self, struct_def: &ast::StructDef) -> MacroResult<ASTNode> {
        let impl_code = format!(
            r#"
impl std::fmt::Debug for {} {{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {{
        f.debug_struct("{}")
            {}
            .finish()
    }}
}}
"#,
            struct_def.name,
            struct_def.name,
            struct_def
                .fields
                .iter()
                .map(|f| format!(".field(\"{}\", &self.{})", f.name, f.name))
                .collect::<Vec<_>>()
                .join("\n            ")
        );

        Ok(ASTNode::ImplBlock(ast::ImplBlock {
            trait_name: Some("std::fmt::Debug".to_string()),
            for_type: struct_def.name.clone(),
            methods: vec![],
            body: impl_code,
        }))
    }

    fn derive_partialeq_struct(&self, struct_def: &ast::StructDef) -> MacroResult<ASTNode> {
        let impl_code = format!(
            r#"
impl PartialEq for {} {{
    fn eq(&self, other: &Self) -> bool {{
        {}
    }}
}}
"#,
            struct_def.name,
            struct_def
                .fields
                .iter()
                .map(|f| format!("self.{} == other.{}", f.name, f.name))
                .collect::<Vec<_>>()
                .join(" &&\n        ")
        );

        Ok(ASTNode::ImplBlock(ast::ImplBlock {
            trait_name: Some("PartialEq".to_string()),
            for_type: struct_def.name.clone(),
            methods: vec![],
            body: impl_code,
        }))
    }

    fn derive_eq_struct(&self, struct_def: &ast::StructDef) -> MacroResult<ASTNode> {
        let impl_code = format!(
            r#"
impl Eq for {} {{
}}
"#,
            struct_def.name
        );

        Ok(ASTNode::ImplBlock(ast::ImplBlock {
            trait_name: Some("Eq".to_string()),
            for_type: struct_def.name.clone(),
            methods: vec![],
            body: impl_code,
        }))
    }

    fn derive_hash_struct(&self, struct_def: &ast::StructDef) -> MacroResult<ASTNode> {
        let impl_code = format!(
            r#"
impl std::hash::Hash for {} {{
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {{
        {}
    }}
}}
"#,
            struct_def.name,
            struct_def
                .fields
                .iter()
                .map(|f| format!("self.{}.hash(state);", f.name))
                .collect::<Vec<_>>()
                .join("\n        ")
        );

        Ok(ASTNode::ImplBlock(ast::ImplBlock {
            trait_name: Some("std::hash::Hash".to_string()),
            for_type: struct_def.name.clone(),
            methods: vec![],
            body: impl_code,
        }))
    }

    fn derive_serialize_struct(&self, struct_def: &ast::StructDef) -> MacroResult<ASTNode> {
        let impl_code = format!(
            r#"
impl serde::Serialize for {} {{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {{
        let mut state = serializer.serialize_struct("{}", {})?;
        {}
        state.end()
    }}
}}
"#,
            struct_def.name,
            struct_def.name,
            struct_def.fields.len(),
            struct_def
                .fields
                .iter()
                .enumerate()
                .map(|(i, f)| format!(
                    "state.serialize_field(\"{}\", &self.{})?;",
                    f.name, f.name
                ))
                .collect::<Vec<_>>()
                .join("\n        ")
        );

        Ok(ASTNode::ImplBlock(ast::ImplBlock {
            trait_name: Some("serde::Serialize".to_string()),
            for_type: struct_def.name.clone(),
            methods: vec![],
            body: impl_code,
        }))
    }

    fn derive_deserialize_struct(&self, struct_def: &ast::StructDef) -> MacroResult<ASTNode> {
        let field_names: Vec<&str> = struct_def.fields.iter().map(|f| f.name.as_str()).collect();
        let impl_code = format!(
            r#"
impl<'de> serde::Deserialize<'de> for {} {{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {{
        #[allow(non_camel_case_types)]
        enum Field {{
            {},
            __ignore,
        }}
        impl<'de> serde::Deserialize<'de> for Field {{
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {{
                struct FieldVisitor;
                impl<'de> serde::de::Visitor<'de> for FieldVisitor {{
                    type Value = Field;
                    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {{
                        formatter.write_str("field name")
                    }}
                    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
                    where
                        E: serde::de::Error,
                    {{
                        match value {{
                            {} => Ok(Field::{}),
                            _ => Ok(Field::__ignore),
                        }}
                    }}
                }}
                deserializer.deserialize_identifier(FieldVisitor)
            }}
        }}
        struct Visitor;
        impl<'de> serde::de::Visitor<'de> for Visitor {{
            type Value = {};
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {{
                formatter.write_str("struct {}")
            }}
            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {{
                {}
                Ok(Self {{ {} }})
            }}
        }}
        const FIELDS: &'static [&'static str] = &[{}];
        deserializer.deserialize_struct("{}", FIELDS, Visitor)
    }}
}}
"#,
            struct_def.name,
            field_names.join(", "),
            field_names
                .iter()
                .map(|f| format!("\"{}\"", f))
                .collect::<Vec<_>>()
                .join(", "),
            field_names
                .iter()
                .map(|f| format!("\"{}\" => Ok(Field::{})", f, to_camel_case(f)))
                .collect::<Vec<_>>()
                .join("\n                            "),
            struct_def.name,
            struct_def.name,
            field_names
                .iter()
                .map(|f| format!("let mut {} = None;", f))
                .collect::<Vec<_>>()
                .join("\n                "),
            field_names.join(", "),
            field_names
                .iter()
                .map(|f| format!(
                    "{}: {}.ok_or_else(|| serde::de::Error::missing_field(\"{}\"))?",
                    f, f, f
                ))
                .collect::<Vec<_>>()
                .join("\n                "),
            field_names
                .iter()
                .map(|f| format!("\"{}\"", f))
                .collect::<Vec<_>>()
                .join(", "),
            struct_def.name
        );

        Ok(ASTNode::ImplBlock(ast::ImplBlock {
            trait_name: Some("serde::Deserialize".to_string()),
            for_type: struct_def.name.clone(),
            methods: vec![],
            body: impl_code,
        }))
    }

    fn derive_clone_enum(&self, enum_def: &ast::EnumDef) -> MacroResult<ASTNode> {
        let impl_code = format!(
            r#"
impl Clone for {} {{
    fn clone(&self) -> Self {{
        match self {{
            {}
        }}
    }}
}}
"#,
            enum_def.name,
            enum_def
                .variants
                .iter()
                .map(|v| {
                    if v.fields.is_empty() {
                        format!("{}::{} => {}::{}", enum_def.name, v.name, enum_def.name, v.name)
                    } else {
                        let fields = v
                            .fields
                            .iter()
                            .map(|f| format!("{}.clone()", f.name))
                            .collect::<Vec<_>>()
                            .join(", ");
                        format!(
                            "{}::{} {{ {} }} => {}::{} {{ {} }}",
                            enum_def.name, v.name, v.fields.iter().map(|f| f.name.as_str()).collect::<Vec<_>>().join(", "),
                            enum_def.name, v.name, fields
                        )
                    }
                })
                .collect::<Vec<_>>()
                .join("\n            ")
        );

        Ok(ASTNode::ImplBlock(ast::ImplBlock {
            trait_name: Some("Clone".to_string()),
            for_type: enum_def.name.clone(),
            methods: vec![],
            body: impl_code,
        }))
    }

    fn derive_copy_enum(&self, enum_def: &ast::EnumDef) -> MacroResult<ASTNode> {
        let all_fields_copy = enum_def
            .variants
            .iter()
            .all(|v| v.fields.iter().all(|f| f.ty == "int" || f.ty == "float" || f.ty == "bool"));

        if !all_fields_copy {
            return Err(MacroError::InvalidInput(
                "Enum with non-Copy fields cannot derive Copy".to_string(),
            ));
        }

        let impl_code = format!(
            r#"
impl Copy for {} {{
}}
"#,
            enum_def.name
        );

        Ok(ASTNode::ImplBlock(ast::ImplBlock {
            trait_name: Some("Copy".to_string()),
            for_type: enum_def.name.clone(),
            methods: vec![],
            body: impl_code,
        }))
    }

    fn derive_debug_enum(&self, enum_def: &ast::EnumDef) -> MacroResult<ASTNode> {
        let impl_code = format!(
            r#"
impl std::fmt::Debug for {} {{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {{
        match self {{
            {}
        }}
    }}
}}
"#,
            enum_def.name,
            enum_def
                .variants
                .iter()
                .map(|v| {
                    if v.fields.is_empty() {
                        format!("{}::{} => write!(f, \"{}::{}\")", enum_def.name, v.name, enum_def.name, v.name)
                    } else {
                        let fields = v
                            .fields
                            .iter()
                            .map(|f| format!("&{}", f.name))
                            .collect::<Vec<_>>()
                            .join(", ");
                        format!(
                            "{}::{} {{ {} }} => write!(f, \"{}::{} {{ {:?} }}\", {})",
                            enum_def.name, v.name, v.fields.iter().map(|f| f.name.as_str()).collect::<Vec<_>>().join(", "),
                            enum_def.name, v.name, v.fields.iter().map(|f| f.name.as_str()).collect::<Vec<_>>().join(", "),
                            fields
                        )
                    }
                })
                .collect::<Vec<_>>()
                .join("\n            ")
        );

        Ok(ASTNode::ImplBlock(ast::ImplBlock {
            trait_name: Some("std::fmt::Debug".to_string()),
            for_type: enum_def.name.clone(),
            methods: vec![],
            body: impl_code,
        }))
    }

    fn derive_partialeq_enum(&self, enum_def: &ast::EnumDef) -> MacroResult<ASTNode> {
        let impl_code = format!(
            r#"
impl PartialEq for {} {{
    fn eq(&self, other: &Self) -> bool {{
        match (self, other) {{
            {}
            _ => false,
        }}
    }}
}}
"#,
            enum_def.name,
            enum_def
                .variants
                .iter()
                .map(|v| {
                    if v.fields.is_empty() {
                        format!(
                            "({}::{}, {}::{}) => true",
                            enum_def.name, v.name, enum_def.name, v.name
                        )
                    } else {
                        let pattern = v.fields.iter().map(|f| format!("ref {}", f.name)).collect::<Vec<_>>().join(", ");
                        let expr = v.fields.iter().map(|f| format!("{} == {}", f.name, f.name)).collect::<Vec<_>>().join(" && ");
                        format!(
                            "({}::{} {{ {} }}, {}::{} {{ {} }}) => {}",
                            enum_def.name, v.name, pattern, enum_def.name, v.name, pattern, expr
                        )
                    }
                })
                .collect::<Vec<_>>()
                .join("\n            ")
        );

        Ok(ASTNode::ImplBlock(ast::ImplBlock {
            trait_name: Some("PartialEq".to_string()),
            for_type: enum_def.name.clone(),
            methods: vec![],
            body: impl_code,
        }))
    }

    fn derive_eq_enum(&self, enum_def: &ast::EnumDef) -> MacroResult<ASTNode> {
        let impl_code = format!(
            r#"
impl Eq for {} {{
}}
"#,
            enum_def.name
        );

        Ok(ASTNode::ImplBlock(ast::ImplBlock {
            trait_name: Some("Eq".to_string()),
            for_type: enum_def.name.clone(),
            methods: vec![],
            body: impl_code,
        }))
    }

    fn derive_hash_enum(&self, enum_def: &ast::EnumDef) -> MacroResult<ASTNode> {
        let impl_code = format!(
            r#"
impl std::hash::Hash for {} {{
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {{
        std::mem::discriminant(self).hash(state);
        match self {{
            {}
        }}
    }}
}}
"#,
            enum_def.name,
            enum_def
                .variants
                .iter()
                .map(|v| {
                    if v.fields.is_empty() {
                        format!("{}::{} => {{}}", enum_def.name, v.name)
                    } else {
                        let fields = v
                            .fields
                            .iter()
                            .map(|f| format!("{}.hash(state);", f.name))
                            .collect::<Vec<_>>()
                            .join("\n                ");
                        format!(
                            "{}::{} {{ {} }} => {{ {} }}",
                            enum_def.name, v.name, v.fields.iter().map(|f| f.name.as_str()).collect::<Vec<_>>().join(", "), fields
                        )
                    }
                })
                .collect::<Vec<_>>()
                .join("\n            ")
        );

        Ok(ASTNode::ImplBlock(ast::ImplBlock {
            trait_name: Some("std::hash::Hash".to_string()),
            for_type: enum_def.name.clone(),
            methods: vec![],
            body: impl_code,
        }))
    }

    fn derive_serialize_enum(&self, enum_def: &ast::EnumDef) -> MacroResult<ASTNode> {
        let impl_code = format!(
            r#"
impl serde::Serialize for {} {{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {{
        match self {{
            {}
        }}
    }}
}}
"#,
            enum_def.name,
            enum_def
                .variants
                .iter()
                .map(|v| {
                    if v.fields.is_empty() {
                        format!(
                            "{}::{} => serializer.serialize_unit_variant(\"{}\", {}, \"{}\")",
                            enum_def.name, v.name, enum_def.name, v.index, v.name
                        )
                    } else {
                        let fields = v.fields.iter().map(|f| format!("&self.{}", f.name)).collect::<Vec<_>>().join(", ");
                        format!(
                            "{}::{} {{ {} }} => serializer.serialize_newtype_variant(\"{}\", {}, \"{}\", {})",
                            enum_def.name, v.name, v.fields.iter().map(|f| f.name.as_str()).collect::<Vec<_>>().join(", "),
                            enum_def.name, v.index, v.name, fields
                        )
                    }
                })
                .collect::<Vec<_>>()
                .join("\n            ")
        );

        Ok(ASTNode::ImplBlock(ast::ImplBlock {
            trait_name: Some("serde::Serialize".to_string()),
            for_type: enum_def.name.clone(),
            methods: vec![],
            body: impl_code,
        }))
    }

    fn derive_deserialize_enum(&self, enum_def: &ast::EnumDef) -> MacroResult<ASTNode> {
        let impl_code = format!(
            r#"
impl<'de> serde::Deserialize<'de> for {} {{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {{
        struct Visitor;
        impl<'de> serde::de::Visitor<'de> for Visitor {{
            type Value = {};
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {{
                formatter.write_str("enum {}")
            }}
            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {{
                match value {{
                    {} => Ok({}::{}),
                    _ => Err(serde::de::Error::unknown_variant(value, VARIANTS)),
                }}
            }}
        }}
        const VARIANTS: &'static [&'static str] = &[{}];
        deserializer.deserialize_enum("{}", VARIANTS, Visitor)
    }}
}}
"#,
            enum_def.name,
            enum_def.name,
            enum_def.name,
            enum_def
                .variants
                .iter()
                .map(|v| format!("\"{}\" => Ok({}::{})", v.name, enum_def.name, v.name))
                .collect::<Vec<_>>()
                .join("\n                    "),
            enum_def
                .variants
                .iter()
                .map(|v| format!("\"{}\"", v.name))
                .collect::<Vec<_>>()
                .join(", "),
            enum_def.name
        );

        Ok(ASTNode::ImplBlock(ast::ImplBlock {
            trait_name: Some("serde::Deserialize".to_string()),
            for_type: enum_def.name.clone(),
            methods: vec![],
            body: impl_code,
        }))
    }
}

fn to_camel_case(s: &str) -> String {
    s.split('_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => {
                    first.to_uppercase().collect::<String>() + chars.as_str()
                }
            }
        })
        .collect()
}

pub struct AttributeMacro {
    name: String,
}

impl AttributeMacro {
    pub fn new(name: String) -> Self {
        AttributeMacro { name }
    }
}

impl ProceduralMacro for AttributeMacro {
    fn name(&self) -> &str {
        &self.name
    }

    fn expand(&self, input: &ASTNode) -> MacroResult<ASTNode> {
        match self.name.as_str() {
            "test" => self.expand_test(input),
            "bench" => self.expand_bench(input),
            "ignore" => self.expand_ignore(input),
            "should_panic" => self.expand_should_panic(input),
            _ => Err(MacroError::InvalidInput(format!(
                "Unknown attribute macro: {}",
                self.name
            ))),
        }
    }
}

impl AttributeMacro {
    fn expand_test(&self, input: &ASTNode) -> MacroResult<ASTNode> {
        match input {
            ASTNode::Function(func) => {
                let test_code = format!(
                    r#"
#[test]
fn {}() {{
    {}
}}
"#,
                    func.name, func.body
                );

                Ok(ASTNode::Function(ast::Function {
                    name: func.name.clone(),
                    params: func.params.clone(),
                    return_type: func.return_type.clone(),
                    body: test_code,
                }))
            }
            _ => Err(MacroError::InvalidInput(
                "test attribute expects a function".to_string(),
            )),
        }
    }

    fn expand_bench(&self, input: &ASTNode) -> MacroResult<ASTNode> {
        match input {
            ASTNode::Function(func) => {
                let bench_code = format!(
                    r#"
#[bench]
fn {}(bencher: &mut Bencher) {{
    bencher.iter(|| {{
        {}
    }});
}}
"#,
                    func.name, func.body
                );

                Ok(ASTNode::Function(ast::Function {
                    name: func.name.clone(),
                    params: func.params.clone(),
                    return_type: func.return_type.clone(),
                    body: bench_code,
                }))
            }
            _ => Err(MacroError::InvalidInput(
                "bench attribute expects a function".to_string(),
            )),
        }
    }

    fn expand_ignore(&self, input: &ASTNode) -> MacroResult<ASTNode> {
        match input {
            ASTNode::Function(func) => {
                let ignore_code = format!(
                    r#"
#[test]
#[ignore]
fn {}() {{
    {}
}}
"#,
                    func.name, func.body
                );

                Ok(ASTNode::Function(ast::Function {
                    name: func.name.clone(),
                    params: func.params.clone(),
                    return_type: func.return_type.clone(),
                    body: ignore_code,
                }))
            }
            _ => Err(MacroError::InvalidInput(
                "ignore attribute expects a function".to_string(),
            )),
        }
    }

    fn expand_should_panic(&self, input: &ASTNode) -> MacroResult<ASTNode> {
        match input {
            ASTNode::Function(func) => {
                let panic_code = format!(
                    r#"
#[test]
#[should_panic]
fn {}() {{
    {}
}}
"#,
                    func.name, func.body
                );

                Ok(ASTNode::Function(ast::Function {
                    name: func.name.clone(),
                    params: func.params.clone(),
                    return_type: func.return_type.clone(),
                    body: panic_code,
                }))
            }
            _ => Err(MacroError::InvalidInput(
                "should_panic attribute expects a function".to_string(),
            )),
        }
    }
}

pub struct FunctionLikeMacro {
    name: String,
}

impl FunctionLikeMacro {
    pub fn new(name: String) -> Self {
        FunctionLikeMacro { name }
    }
}

impl ProceduralMacro for FunctionLikeMacro {
    fn name(&self) -> &str {
        &self.name
    }

    fn expand(&self, input: &ASTNode) -> MacroResult<ASTNode> {
        match self.name.as_str() {
            "vec" => self.expand_vec(input),
            "format" => self.expand_format(input),
            "assert" => self.expand_assert(input),
            "debug_assert" => self.expand_debug_assert(input),
            _ => Err(MacroError::InvalidInput(format!(
                "Unknown function-like macro: {}",
                self.name
            ))),
        }
    }
}

impl FunctionLikeMacro {
    fn expand_vec(&self, input: &ASTNode) -> MacroResult<ASTNode> {
        match input {
            ASTNode::ArrayLiteral(arr) => {
                let vec_code = format!(
                    r#"
{{
    let mut v = Vec::new();
    {}
    v
}}
"#,
                    arr.elements
                        .iter()
                        .map(|e| format!("v.push({});", e))
                        .collect::<Vec<_>>()
                        .join("\n    ")
                );

                Ok(ASTNode::Block(vec_code))
            }
            _ => Err(MacroError::InvalidInput(
                "vec! macro expects an array literal".to_string(),
            )),
        }
    }

    fn expand_format(&self, input: &ASTNode) -> MacroResult<ASTNode> {
        match input {
            ASTNode::Call(call) => {
                let format_code = format!(
                    r#"
{{
        let mut result = String::new();
        {}
        result
    }}
"#,
                    call.args
                        .iter()
                        .map(|a| format!("result.push_str(&format!(\"{{}}\", {}));", a))
                        .collect::<Vec<_>>()
                        .join("\n        ")
                );

                Ok(ASTNode::Block(format_code))
            }
            _ => Err(MacroError::InvalidInput(
                "format! macro expects a call expression".to_string(),
            )),
        }
    }

    fn expand_assert(&self, input: &ASTNode) -> MacroResult<ASTNode> {
        match input {
            ASTNode::Call(call) => {
                let assert_code = format!(
                    r#"
if !({}) {{
    panic!("assertion failed: {}", "{}");
}}
"#,
                    call.args[0], call.args[0], call.args[0]
                );

                Ok(ASTNode::Block(assert_code))
            }
            _ => Err(MacroError::InvalidInput(
                "assert! macro expects a call expression".to_string(),
            )),
        }
    }

    fn expand_debug_assert(&self, input: &ASTNode) -> MacroResult<ASTNode> {
        match input {
            ASTNode::Call(call) => {
                let assert_code = format!(
                    r#"
if cfg!(debug_assertions) {{
    if !({}) {{
        panic!("debug assertion failed: {}", "{}");
    }}
}}
"#,
                    call.args[0], call.args[0], call.args[0]
                );

                Ok(ASTNode::Block(assert_code))
            }
            _ => Err(MacroError::InvalidInput(
                "debug_assert! macro expects a call expression".to_string(),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_macro_registry() {
        let mut registry = MacroRegistry::new();
        let mac = Arc::new(DeriveMacro::new("Clone".to_string(), DeriveTarget::Struct));
        registry.register(mac);
        assert!(registry.get("Clone").is_some());
    }

    #[test]
    fn test_to_camel_case() {
        assert_eq!(to_camel_case("hello_world"), "HelloWorld");
        assert_eq!(to_camel_case("foo_bar_baz"), "FooBarBaz");
    }
}
