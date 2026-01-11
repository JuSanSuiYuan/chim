use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SwiftType {
    Void,
    Bool,
    Int,
    Int8,
    Int16,
    Int32,
    Int64,
    UInt,
    UInt8,
    UInt16,
    UInt32,
    UInt64,
    Float,
    Double,
    String,
    Character,
    Array(Box<SwiftType>),
    Dictionary(Box<SwiftType>, Box<SwiftType>),
    Optional(Box<SwiftType>),
    Result(Box<SwiftType>, Box<SwiftType>),
    Error,
    Any,
    AnyObject,
    UnsafeMutableRawPointer,
    UnsafeRawPointer,
    UnsafeMutablePointer(Box<SwiftType>),
    UnsafePointer(Box<SwiftType>),
    Struct(String),
    Class(String),
    Protocol(String),
    Enum(String),
    Actor(String),
}

#[derive(Debug, Clone)]
pub struct SwiftMethod {
    pub name: String,
    pub return_type: SwiftType,
    pub params: Vec<SwiftParameter>,
    pub is_static: bool,
    pub is_final: bool,
    pub is_mutating: bool,
    pub is_async: bool,
    pub is_throwing: bool,
    pub access: SwiftAccess,
    pub body: Option<String>,
    pub attributes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SwiftAccess {
    Public,
    Internal,
    Private,
    FilePrivate,
}

#[derive(Debug, Clone)]
pub struct SwiftParameter {
    pub name: String,
    pub param_type: SwiftType,
    pub is_inout: bool,
    pub default_value: Option<String>,
    pub label: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SwiftStruct {
    pub name: String,
    pub fields: Vec<SwiftField>,
    pub methods: Vec<SwiftMethod>,
    pub access: SwiftAccess,
    pub conforms_to: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct SwiftField {
    pub name: String,
    pub field_type: SwiftType,
    pub access: SwiftAccess,
    pub is_mutable: bool,
}

#[derive(Debug, Clone)]
pub struct SwiftProtocol {
    pub name: String,
    pub methods: Vec<SwiftMethod>,
    pub access: SwiftAccess,
    pub inherits: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct SwiftClass {
    pub name: String,
    pub fields: Vec<SwiftField>,
    pub methods: Vec<SwiftMethod>,
    pub access: SwiftAccess,
    pub is_final: bool,
    pub inherits: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct SwiftActor {
    pub name: String,
    pub fields: Vec<SwiftField>,
    pub methods: Vec<SwiftMethod>,
    pub access: SwiftAccess,
}

#[derive(Debug, Clone)]
pub struct SwiftFFIGenerator {
    type_mapping: HashMap<String, SwiftType>,
    imports: Vec<String>,
}

impl SwiftFFIGenerator {
    pub fn new() -> Self {
        let mut generator = Self {
            type_mapping: HashMap::new(),
            imports: vec!["Foundation".to_string()],
        };
        generator.init_type_mapping();
        generator
    }

    fn init_type_mapping(&mut self) {
        self.type_mapping
            .insert("c_void".to_string(), SwiftType::Void);
        self.type_mapping
            .insert("c_bool".to_string(), SwiftType::Bool);
        self.type_mapping
            .insert("c_char".to_string(), SwiftType::Int8);
        self.type_mapping
            .insert("c_int".to_string(), SwiftType::Int32);
        self.type_mapping
            .insert("c_long".to_string(), SwiftType::Int64);
        self.type_mapping
            .insert("c_float".to_string(), SwiftType::Float);
        self.type_mapping
            .insert("c_double".to_string(), SwiftType::Double);
        self.type_mapping
            .insert("c_string".to_string(), SwiftType::String);
        self.type_mapping
            .insert("c_voidptr".to_string(), SwiftType::UnsafeMutableRawPointer);
    }

    pub fn generate_bridging_header(&self, functions: &[SwiftFunction]) -> String {
        let mut output = String::new();

        output.push_str("#ifndef CHIM_SWIFT_BRIDGE_H\n");
        output.push_str("#define CHIM_SWIFT_BRIDGE_H\n\n");
        output.push_str("#include <stdint.h>\n");
        output.push_str("#include <stdbool.h>\n");
        output.push_str("#include <stddef.h>\n\n");

        for func in functions {
            self.generate_c_function_declaration(func, &mut output);
        }

        output.push_str("\n#endif /* CHIM_SWIFT_BRIDGE_H */\n");
        output
    }

    fn generate_c_function_declaration(&self, func: &SwiftFunction, output: &mut String) {
        output.push_str(&format!(
            "{} {}(",
            self.map_c_type_to_string(&func.return_type),
            func.name
        ));

        let params: Vec<String> = func
            .params
            .iter()
            .map(|p| format!("{} {}", self.map_c_type_to_string(&p.param_type), p.name))
            .collect();
        output.push_str(&params.join(", "));
        output.push_str(");\n");
    }

    fn map_c_type_to_string(&self, ty: &SwiftCType) -> String {
        match ty {
            SwiftCType::CVoid => "void".to_string(),
            SwiftCType::CBool => "bool".to_string(),
            SwiftCType::CChar => "char".to_string(),
            SwiftCType::CShort => "short".to_string(),
            SwiftCType::CInt => "int".to_string(),
            SwiftCType::CLong => "long".to_string(),
            SwiftCType::CLongLong => "long long".to_string(),
            SwiftCType::CUChar => "unsigned char".to_string(),
            SwiftCType::CUShort => "unsigned short".to_string(),
            SwiftCType::CUInt => "unsigned int".to_string(),
            SwiftCType::CULong => "unsigned long".to_string(),
            SwiftCType::CULongLong => "unsigned long long".to_string(),
            SwiftCType::CFloat => "float".to_string(),
            SwiftCType::CDouble => "double".to_string(),
            SwiftCType::CString => "const char *".to_string(),
            SwiftCType::CVoidPtr => "void *".to_string(),
        }
    }

    pub fn generate_swift_module(&self, decls: &[SwiftDeclaration]) -> String {
        let mut output = String::new();

        output.push_str("// Swift FFI Module\n");
        output.push_str("// Generated by Chim Compiler\n\n");

        for import in &self.imports {
            output.push_str(&format!("import {}\n", import));
        }
        output.push_str("\n");

        for decl in decls {
            match decl {
                SwiftDeclaration::Struct(s) => self.generate_swift_struct(s, &mut output),
                SwiftDeclaration::Class(c) => self.generate_swift_class(c, &mut output),
                SwiftDeclaration::Protocol(p) => self.generate_swift_protocol(p, &mut output),
                SwiftDeclaration::Actor(a) => self.generate_swift_actor(a, &mut output),
            }
        }

        output
    }

    fn generate_swift_struct(&self, struct_: &SwiftStruct, output: &mut String) {
        let access = self.access_to_string(struct_.access);

        output.push_str(&format!("{}struct {}", access, struct_.name));

        if !struct_.conforms_to.is_empty() {
            output.push_str(&format!(": {}", struct_.conforms_to.join(", ")));
        }

        output.push_str(" {\n");

        for field in &struct_.fields {
            self.generate_swift_field(field, output);
        }

        output.push_str("}\n\n");
    }

    fn generate_swift_class(&self, class: &SwiftClass, output: &mut String) {
        let access = self.access_to_string(class.access);
        let final_str = if class.is_final { "final " } else { "" };

        output.push_str(&format!("{}{}class {}", final_str, access, class.name));

        if !class.inherits.is_empty() {
            output.push_str(&format!(": {}", class.inherits.join(", ")));
        }

        output.push_str(" {\n");

        for field in &class.fields {
            self.generate_swift_field(field, output);
        }

        output.push_str("}\n\n");
    }

    fn generate_swift_protocol(&self, protocol_: &SwiftProtocol, output: &mut String) {
        let access = self.access_to_string(protocol_.access);

        output.push_str(&format!("{}protocol {}", access, protocol_.name));

        if !protocol_.inherits.is_empty() {
            output.push_str(&format!(": {}", protocol_.inherits.join(", ")));
        }

        output.push_str(" {\n");

        for method in &protocol_.methods {
            self.generate_swift_protocol_method(method, output);
        }

        output.push_str("}\n\n");
    }

    fn generate_swift_actor(&self, actor: &SwiftActor, output: &mut String) {
        let access = self.access_to_string(actor.access);

        output.push_str(&format!("{}actor {}", access, actor.name));

        output.push_str(" {\n");

        for field in &actor.fields {
            self.generate_swift_field(field, output);
        }

        output.push_str("}\n\n");
    }

    fn generate_swift_field(&self, field: &SwiftField, output: &mut String) {
        let access = self.access_to_string(field.access);
        let mutable = if field.is_mutable { "var" } else { "let" };

        output.push_str(&format!("    {} {}: {}", access, mutable, field.name));
        output.push_str(&format!(": {}", self.type_to_string(&field.field_type)));
        output.push_str("\n");
    }

    fn generate_swift_protocol_method(&self, method: &SwiftMethod, output: &mut String) {
        output.push_str(&format!("    func {}", method.name));
        output.push_str("(");
        let params: Vec<String> = method
            .params
            .iter()
            .map(|p| self.format_parameter(p))
            .collect();
        output.push_str(&params.join(", "));
        output.push_str(")");

        if method.is_async {
            output.push_str(" async");
        }

        if method.is_throwing {
            output.push_str(" throws");
        }

        output.push_str(" -> ");
        output.push_str(&self.type_to_string(&method.return_type));
        output.push_str("\n");
    }

    fn format_parameter(&self, param: &SwiftParameter) -> String {
        let label = param.label.clone().unwrap_or_else(|| param.name.clone());
        let inout = if param.is_inout { "inout " } else { "" };
        format!(
            "{}{}: {}",
            inout,
            label,
            self.type_to_string(&param.param_type)
        )
    }

    fn access_to_string(&self, access: SwiftAccess) -> String {
        match access {
            SwiftAccess::Public => "public".to_string(),
            SwiftAccess::Internal => "internal".to_string(),
            SwiftAccess::Private => "private".to_string(),
            SwiftAccess::FilePrivate => "fileprivate".to_string(),
        }
    }

    fn type_to_string(&self, ty: &SwiftType) -> String {
        match ty {
            SwiftType::Void => "Void".to_string(),
            SwiftType::Bool => "Bool".to_string(),
            SwiftType::Int => "Int".to_string(),
            SwiftType::Int8 => "Int8".to_string(),
            SwiftType::Int16 => "Int16".to_string(),
            SwiftType::Int32 => "Int32".to_string(),
            SwiftType::Int64 => "Int64".to_string(),
            SwiftType::UInt => "UInt".to_string(),
            SwiftType::UInt8 => "UInt8".to_string(),
            SwiftType::UInt16 => "UInt16".to_string(),
            SwiftType::UInt32 => "UInt32".to_string(),
            SwiftType::UInt64 => "UInt64".to_string(),
            SwiftType::Float => "Float".to_string(),
            SwiftType::Double => "Double".to_string(),
            SwiftType::String => "String".to_string(),
            SwiftType::Character => "Character".to_string(),
            SwiftType::Array(inner) => format!("[{}]", self.type_to_string(inner)),
            SwiftType::Dictionary(key, value) => {
                format!(
                    "[{}: {}]",
                    self.type_to_string(key),
                    self.type_to_string(value)
                )
            }
            SwiftType::Optional(inner) => format!("{}?", self.type_to_string(inner)),
            SwiftType::Error => "Error".to_string(),
            SwiftType::Any => "Any".to_string(),
            SwiftType::AnyObject => "AnyObject".to_string(),
            SwiftType::UnsafeMutableRawPointer => "UnsafeMutableRawPointer".to_string(),
            SwiftType::UnsafeRawPointer => "UnsafeRawPointer".to_string(),
            SwiftType::UnsafeMutablePointer(inner) => {
                format!("UnsafeMutablePointer<{}>", self.type_to_string(inner))
            }
            SwiftType::UnsafePointer(inner) => {
                format!("UnsafePointer<{}>", self.type_to_string(inner))
            }
            SwiftType::Struct(name) => name.clone(),
            SwiftType::Class(name) => name.clone(),
            SwiftType::Protocol(name) => format!("{}Protocol", name),
            SwiftType::Enum(name) => name.clone(),
            SwiftType::Actor(name) => name.clone(),
            _ => "Any".to_string(),
        }
    }

    pub fn add_import(&mut self, import: String) {
        if !self.imports.contains(&import) {
            self.imports.push(import);
        }
    }
}

#[derive(Debug, Clone)]
pub struct SwiftFunction {
    pub name: String,
    pub return_type: SwiftCType,
    pub params: Vec<SwiftCParameter>,
    pub access: SwiftAccess,
    pub is_async: bool,
    pub is_throwing: bool,
}

#[derive(Debug, Clone)]
pub struct SwiftCParameter {
    pub name: String,
    pub param_type: SwiftCType,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SwiftCType {
    CVoid,
    CBool,
    CChar,
    CShort,
    CInt,
    CLong,
    CLongLong,
    CUChar,
    CUShort,
    CUInt,
    CULong,
    CULongLong,
    CFloat,
    CDouble,
    CString,
    CVoidPtr,
}

#[derive(Debug, Clone)]
pub enum SwiftDeclaration {
    Struct(SwiftStruct),
    Class(SwiftClass),
    Protocol(SwiftProtocol),
    Actor(SwiftActor),
}

pub fn map_chim_type_to_swift(chim_type: &crate::Type) -> SwiftType {
    match chim_type {
        crate::Type::CVoid => SwiftType::Void,
        crate::Type::CBool => SwiftType::Bool,
        crate::Type::CChar => SwiftType::Int8,
        crate::Type::CShort => SwiftType::Int16,
        crate::Type::CInt => SwiftType::Int32,
        crate::Type::CLong => SwiftType::Int64,
        crate::Type::CLongLong => SwiftType::Int64,
        crate::Type::CUChar => SwiftType::UInt8,
        crate::Type::CUShort => SwiftType::UInt16,
        crate::Type::CUInt => SwiftType::UInt32,
        crate::Type::CULong => SwiftType::UInt64,
        crate::Type::CULongLong => SwiftType::UInt64,
        crate::Type::CFloat => SwiftType::Float,
        crate::Type::CDouble => SwiftType::Double,
        crate::Type::CStr => SwiftType::String,
        crate::Type::CVoidPtr => SwiftType::UnsafeMutableRawPointer,
        crate::Type::ISize => SwiftType::Int,
        crate::Type::USize => SwiftType::UInt,
        crate::Type::Pointer { target, .. } => {
            SwiftType::UnsafeMutablePointer(Box::new(map_chim_type_to_swift(target)))
        }
        crate::Type::Array { element, length: _ } => {
            SwiftType::Array(Box::new(map_chim_type_to_swift(element)))
        }
        crate::Type::Function { params, ret, .. } => {
            let param_types: Vec<SwiftType> =
                params.iter().map(|p| map_chim_type_to_swift(p)).collect();
            let ret_type = map_chim_type_to_swift(ret);
            SwiftType::Struct(format!(
                "@convention(c) ({}) -> {}",
                param_types
                    .iter()
                    .map(|t| self.type_to_string(t))
                    .collect::<Vec<_>>()
                    .join(", "),
                self.type_to_string(&ret_type)
            ))
        }
        crate::Type::Struct { name, .. } => SwiftType::Struct(name.to_string()),
        _ => SwiftType::Any,
    }
}
