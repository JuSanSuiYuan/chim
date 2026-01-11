use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DotNet10Type {
    Void,
    Boolean,
    SByte,
    Byte,
    Int16,
    UInt16,
    Int32,
    UInt32,
    Int64,
    UInt64,
    Single,
    Double,
    Decimal,
    Char,
    String,
    Object,
    Dynamic,
    DateTime,
    TimeSpan,
    Guid,
    IntPtr,
    UIntPtr,
    Nullable(Box<DotNet10Type>),
    Array(Box<DotNet10Type>),
    List(Box<DotNet10Type>),
    Dictionary(Box<DotNet10Type>, Box<DotNet10Type>),
    HashSet(Box<DotNet10Type>),
    Queue(Box<DotNet10Type>),
    Stack(Box<DotNet10Type>),
    Task,
    TaskT(Box<DotNet10Type>),
    ValueTask,
    ValueTaskT(Box<DotNet10Type>),
    Func(Vec<DotNet10Type>, Box<DotNet10Type>),
    Action(Vec<DotNet10Type>),
    Predicate(Box<DotNet10Type>),
    IEnumerable(Box<DotNet10Type>),
    IEnumerator(Box<DotNet10Type>),
    ICollection(Box<DotNet10Type>),
    IList(Box<DotNet10Type>),
    IDictionary(Box<DotNet10Type>, Box<DotNet10Type>),
    IAsyncEnumerable(Box<DotNet10Type>),
    IAsyncEnumerator(Box<DotNet10Type>),
    IDisposable,
    IAsyncDisposable,
    Struct(String),
    Record(String),
    RecordStruct(String),
    Class(String),
    Interface(String),
    Enum(String),
    Delegate(String, Vec<DotNet10Type>, Box<DotNet10Type>),
    Tuple(Vec<DotNet10Type>),
    ValueTuple(Vec<DotNet10Type>),
    Span(Box<DotNet10Type>),
    ReadOnlySpan(Box<DotNet10Type>),
    Memory(Box<DotNet10Type>),
    ReadOnlyMemory(Box<DotNet10Type>),
    Index,
    Range,
    Utf8String,
}

#[derive(Debug, Clone)]
pub struct DotNet10Function {
    pub name: String,
    pub return_type: DotNet10Type,
    pub params: Vec<DotNet10Parameter>,
    pub access: DotNet10Access,
    pub is_static: bool,
    pub is_virtual: bool,
    pub is_override: bool,
    pub is_async: bool,
    pub is_partial: bool,
    pub is_extension: bool,
    pub body: Option<String>,
    pub attributes: Vec<DotNet10Attribute>,
    pub type_params: Vec<String>,
    pub where_constraints: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct DotNet10Parameter {
    pub name: String,
    pub param_type: DotNet10Type,
    pub is_params: bool,
    pub is_out: bool,
    pub is_ref: bool,
    pub in_param: bool,
    pub default_value: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DotNet10Access {
    Public,
    Internal,
    Protected,
    Private,
    ProtectedInternal,
    PrivateProtected,
}

#[derive(Debug, Clone)]
pub struct DotNet10Attribute {
    pub name: String,
    pub arguments: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct DotNet10Struct {
    pub name: String,
    pub fields: Vec<DotNet10Property>,
    pub methods: Vec<DotNet10Function>,
    pub access: DotNet10Access,
    pub is_readonly: bool,
    pub is_partial: bool,
    pub implements: Vec<String>,
    pub type_params: Vec<String>,
    pub where_constraints: Vec<String>,
    pub attributes: Vec<DotNet10Attribute>,
}

#[derive(Debug, Clone)]
pub struct DotNet10Property {
    pub name: String,
    pub property_type: DotNet10Type,
    pub access: DotNet10Access,
    pub getter: Option<String>,
    pub setter: Option<String>,
    pub init: Option<String>,
    pub is_init_only: bool,
    pub is_static: bool,
    pub default_value: Option<String>,
    pub attributes: Vec<DotNet10Attribute>,
}

#[derive(Debug, Clone)]
pub struct DotNet10Record {
    pub name: String,
    pub properties: Vec<DotNet10Property>,
    pub methods: Vec<DotNet10Function>,
    pub parameters: Vec<DotNet10Parameter>,
    pub access: DotNet10Access,
    pub is_class: bool,
    pub is_struct: bool,
    pub implements: Vec<String>,
    pub type_params: Vec<String>,
    pub where_constraints: Vec<String>,
    pub attributes: Vec<DotNet10Attribute>,
    pub base_type: Option<String>,
}

#[derive(Debug, Clone)]
pub struct DotNet10Class {
    pub name: String,
    pub properties: Vec<DotNet10Property>,
    pub methods: Vec<DotNet10Function>,
    pub fields: Vec<DotNet10Field>,
    pub access: DotNet10Access,
    pub is_static: bool,
    pub is_abstract: bool,
    pub is_sealed: bool,
    pub is_partial: bool,
    pub inherits: Vec<String>,
    pub implements: Vec<String>,
    pub type_params: Vec<String>,
    pub where_constraints: Vec<String>,
    pub attributes: Vec<DotNet10Attribute>,
}

#[derive(Debug, Clone)]
pub struct DotNet10Field {
    pub name: String,
    pub field_type: DotNet10Type,
    pub access: DotNet10Access,
    pub is_static: bool,
    pub is_readonly: bool,
    pub is_volatile: bool,
    pub default_value: Option<String>,
}

#[derive(Debug, Clone)]
pub struct DotNet10Interface {
    pub name: String,
    pub methods: Vec<DotNet10Function>,
    pub properties: Vec<DotNet10Property>,
    pub access: DotNet10Access,
    pub inherits: Vec<String>,
    pub type_params: Vec<String>,
    pub where_constraints: Vec<String>,
    pub attributes: Vec<DotNet10Attribute>,
}

#[derive(Debug, Clone)]
pub struct DotNet10Delegate {
    pub name: String,
    pub return_type: DotNet10Type,
    pub params: Vec<DotNet10Parameter>,
    pub access: DotNet10Access,
    pub type_params: Vec<String>,
    pub where_constraints: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct DotNet10Enum {
    pub name: String,
    pub underlying_type: DotNet10Type,
    pub values: Vec<(String, i64)>,
    pub access: DotNet10Access,
    pub attributes: Vec<DotNet10Attribute>,
}

#[derive(Debug, Clone)]
pub struct DotNet10Namespace {
    pub name: String,
    pub using_directives: Vec<String>,
    pub declarations: Vec<DotNet10Declaration>,
}

#[derive(Debug, Clone)]
pub struct DotNet10FFIGenerator {
    type_mapping: HashMap<String, DotNet10Type>,
    options: DotNet10FFIOptions,
}

#[derive(Debug, Clone)]
pub struct DotNet10FFIOptions {
    pub language_version: String,
    pub enable_nullable: bool,
    pub enable_init_only: bool,
    pub enable_records: bool,
    pub enable_source_generators: bool,
    pub enable_top_level_statements: bool,
    pub enable_system_text_json: bool,
    pub target_framework: String,
}

impl Default for DotNet10FFIOptions {
    fn default() -> Self {
        Self {
            language_version: "10.0".to_string(),
            enable_nullable: true,
            enable_init_only: true,
            enable_records: true,
            enable_source_generators: true,
            enable_top_level_statements: false,
            enable_system_text_json: true,
            target_framework: "net10.0".to_string(),
        }
    }
}

impl DotNet10FFIGenerator {
    pub fn new() -> Self {
        let mut generator = Self {
            type_mapping: HashMap::new(),
            options: DotNet10FFIOptions::default(),
        };
        generator.init_type_mapping();
        generator
    }

    fn init_type_mapping(&mut self) {
        self.type_mapping
            .insert("c_void".to_string(), DotNet10Type::Void);
        self.type_mapping
            .insert("c_bool".to_string(), DotNet10Type::Boolean);
        self.type_mapping
            .insert("c_char".to_string(), DotNet10Type::SByte);
        self.type_mapping
            .insert("c_int".to_string(), DotNet10Type::Int32);
        self.type_mapping
            .insert("c_long".to_string(), DotNet10Type::Int64);
        self.type_mapping
            .insert("c_float".to_string(), DotNet10Type::Single);
        self.type_mapping
            .insert("c_double".to_string(), DotNet10Type::Double);
        self.type_mapping
            .insert("c_string".to_string(), DotNet10Type::String);
        self.type_mapping
            .insert("c_voidptr".to_string(), DotNet10Type::IntPtr);
    }

    pub fn generate_dotnet10_project(&self, namespace: &DotNet10Namespace) -> String {
        let mut output = String::new();

        output.push_str("// .NET 10 FFI Module\n");
        output.push_str("// Generated by Chim Compiler\n\n");

        output.push_str("using System;\n");
        output.push_str("using System.Collections.Generic;\n");
        output.push_str("using System.Threading.Tasks;\n");
        output.push_str("using System.Text.Json.Serialization;\n\n");

        if self.options.enable_nullable {
            output.push_str("#nullable enable\n\n");
        }

        output.push_str(&format!("namespace {}\n{{\n", namespace.name));

        output.push_str(
            &namespace
                .using_directives
                .iter()
                .map(|u| format!("    using {};", u))
                .collect::<Vec<_>>()
                .join("\n"),
        );
        if !namespace.using_directives.is_empty() {
            output.push_str("\n");
        }

        output.push_str("\n");

        for decl in &namespace.declarations {
            match decl {
                DotNet10Declaration::Class(c) => self.generate_dotnet10_class(c, &mut output, 1),
                DotNet10Declaration::Struct(s) => self.generate_dotnet10_struct(s, &mut output, 1),
                DotNet10Declaration::Record(r) => self.generate_dotnet10_record(r, &mut output, 1),
                DotNet10Declaration::Interface(i) => {
                    self.generate_dotnet10_interface(i, &mut output, 1)
                }
                DotNet10Declaration::Enum(e) => self.generate_dotnet10_enum(e, &mut output, 1),
                DotNet10Declaration::Delegate(d) => {
                    self.generate_dotnet10_delegate(d, &mut output, 1)
                }
            }
        }

        output.push_str("}\n");

        output
    }

    fn generate_dotnet10_class(&self, class: &DotNet10Class, output: &mut String, indent: usize) {
        let indent_str = "    ".repeat(indent);
        let access = self.access_to_string(class.access);
        let static_str = if class.is_static { "static " } else { "" };
        let abstract_str = if class.is_abstract { "abstract " } else { "" };
        let sealed_str = if class.is_sealed { "sealed " } else { "" };
        let partial_str = if class.is_partial { "partial " } else { "" };

        for attr in &class.attributes {
            output.push_str(&format!(
                "{}@{}\n",
                indent_str,
                self.attribute_to_string(attr)
            ));
        }

        output.push_str(&format!(
            "{}{}{}{}{}class {}",
            indent_str, access, static_str, abstract_str, sealed_str, partial_str
        ));
        output.push_str(&class.name);

        if !class.type_params.is_empty() {
            output.push_str(&format!("<{}>", class.type_params.join(", ")));
        }

        if !class.inherits.is_empty() {
            output.push_str(&format!(" : {}", class.inherits.join(", ")));
        }

        if !class.implements.is_empty() {
            output.push_str(&format!(", {}", class.implements.join(", ")));
        }

        if !class.where_constraints.is_empty() {
            output.push_str(&format!(
                " where {}",
                class.where_constraints.join(" where ")
            ));
        }

        output.push_str("\n");
        output.push_str(&format!("{}{{\n", indent_str));

        for field in &class.fields {
            self.generate_dotnet10_field(field, output, indent + 1);
        }

        if !class.properties.is_empty() {
            output.push_str("\n");
            for prop in &class.properties {
                self.generate_dotnet10_property(prop, output, indent + 1);
            }
        }

        if !class.methods.is_empty() {
            output.push_str("\n");
            for method in &class.methods {
                self.generate_dotnet10_function(method, output, indent + 1);
            }
        }

        output.push_str(&format!("}}\n\n"));
    }

    fn generate_dotnet10_struct(
        &self,
        struct_: &DotNet10Struct,
        output: &mut String,
        indent: usize,
    ) {
        let indent_str = "    ".repeat(indent);
        let access = self.access_to_string(struct_.access);
        let readonly_str = if struct_.is_readonly { "readonly " } else { "" };
        let partial_str = if struct_.is_partial { "partial " } else { "" };

        output.push_str(&format!(
            "{}{}{}struct {}",
            indent_str, access, readonly_str, partial_str
        ));
        output.push_str(&struct_.name);

        if !struct_.type_params.is_empty() {
            output.push_str(&format!("<{}>", struct_.type_params.join(", ")));
        }

        if !struct_.implements.is_empty() {
            output.push_str(&format!(" : {}", struct_.implements.join(", ")));
        }

        if !struct_.where_constraints.is_empty() {
            output.push_str(&format!(
                " where {}",
                struct_.where_constraints.join(" where ")
            ));
        }

        output.push_str("\n");
        output.push_str(&format!("{}{{\n", indent_str));

        for prop in &struct_.properties {
            self.generate_dotnet10_property(prop, output, indent + 1);
        }

        if !struct_.methods.is_empty() {
            output.push_str("\n");
            for method in &struct_.methods {
                self.generate_dotnet10_function(method, output, indent + 1);
            }
        }

        output.push_str(&format!("}}\n\n"));
    }

    fn generate_dotnet10_record(
        &self,
        record: &DotNet10Record,
        output: &mut String,
        indent: usize,
    ) {
        let indent_str = "    ".repeat(indent);
        let access = self.access_to_string(record.access);

        let record_keyword = if record.is_struct {
            "record struct"
        } else {
            "record"
        };

        output.push_str(&format!("{}{} {}", indent_str, access, record_keyword));
        output.push_str(&record.name);

        if !record.type_params.is_empty() {
            output.push_str(&format!("<{}>", record.type_params.join(", ")));
        }

        if !record.parameters.is_empty() {
            let params: Vec<String> = record
                .parameters
                .iter()
                .map(|p| self.format_parameter(p))
                .collect();
            output.push_str(&format!("({})", params.join(", ")));
        }

        if !record.implements.is_empty() {
            output.push_str(&format!(" : {}", record.implements.join(", ")));
        }

        if !record.where_constraints.is_empty() {
            output.push_str(&format!(
                " where {}",
                record.where_constraints.join(" where ")
            ));
        }

        output.push_str("\n");
        output.push_str(&format!("{}{{\n", indent_str));

        for prop in &record.properties {
            self.generate_dotnet10_property(prop, output, indent + 1);
        }

        if !record.methods.is_empty() {
            output.push_str("\n");
            for method in &record.methods {
                self.generate_dotnet10_function(method, output, indent + 1);
            }
        }

        output.push_str(&format!("}}\n\n"));
    }

    fn generate_dotnet10_interface(
        &self,
        interface: &DotNet10Interface,
        output: &mut String,
        indent: usize,
    ) {
        let indent_str = "    ".repeat(indent);
        let access = self.access_to_string(interface.access);

        output.push_str(&format!("{}interface {}", indent_str, access));
        output.push_str(&interface.name);

        if !interface.type_params.is_empty() {
            output.push_str(&format!("<{}>", interface.type_params.join(", ")));
        }

        if !interface.inherits.is_empty() {
            output.push_str(&format!(" : {}", interface.inherits.join(", ")));
        }

        if !interface.where_constraints.is_empty() {
            output.push_str(&format!(
                " where {}",
                interface.where_constraints.join(" where ")
            ));
        }

        output.push_str("\n");
        output.push_str(&format!("{}{{\n", indent_str));

        for method in &interface.methods {
            self.generate_dotnet10_function(method, output, indent + 1);
        }

        for prop in &interface.properties {
            self.generate_dotnet10_property(prop, output, indent + 1);
        }

        output.push_str(&format!("}}\n\n"));
    }

    fn generate_dotnet10_enum(&self, enum_: &DotNet10Enum, output: &mut String, indent: usize) {
        let indent_str = "    ".repeat(indent);
        let access = self.access_to_string(enum_.access);

        output.push_str(&format!("{}enum {}", indent_str, access));
        output.push_str(&enum_.name);

        let underlying_str = self.type_to_string(&enum_.underlying_type);
        if underlying_str != "Int32" {
            output.push_str(&format!(" : {}", underlying_str));
        }

        output.push_str("\n");
        output.push_str(&format!("{}{{\n", indent_str));

        for (name, value) in &enum_.values {
            output.push_str(&format!("{}    {} = {},\n", indent_str, name, value));
        }

        output.push_str(&format!("}}\n\n"));
    }

    fn generate_dotnet10_delegate(
        &self,
        delegate: &DotNet10Delegate,
        output: &mut String,
        indent: usize,
    ) {
        let indent_str = "    ".repeat(indent);
        let access = self.access_to_string(delegate.access);

        output.push_str(&format!("{}delegate {}", indent_str, access));
        output.push_str(&self.type_to_string(&delegate.return_type));
        output.push_str(" ");
        output.push_str(&delegate.name);

        if !delegate.type_params.is_empty() {
            output.push_str(&format!("<{}>", delegate.type_params.join(", ")));
        }

        output.push_str("(");
        let params: Vec<String> = delegate
            .params
            .iter()
            .map(|p| self.format_parameter(p))
            .collect();
        output.push_str(&params.join(", "));
        output.push_str(")");

        if !delegate.where_constraints.is_empty() {
            output.push_str(&format!(
                " where {}",
                delegate.where_constraints.join(" where ")
            ));
        }

        output.push_str(";\n\n");
    }

    fn generate_dotnet10_property(
        &self,
        prop: &DotNet10Property,
        output: &mut String,
        indent: usize,
    ) {
        let indent_str = "    ".repeat(indent);
        let access = self.access_to_string(prop.access);
        let static_str = if prop.is_static { "static " } else { "" };
        let init_only = if prop.is_init_only { "init " } else { "" };

        for attr in &prop.attributes {
            output.push_str(&format!(
                "{}@{}\n",
                indent_str,
                self.attribute_to_string(attr)
            ));
        }

        output.push_str(&format!(
            "{}{}{} {} {}",
            indent_str, access, static_str, init_only, prop.name
        ));
        output.push_str(&format!(": {}", self.type_to_string(&prop.property_type)));

        output.push_str("\n");
        output.push_str(&format!("{}{{\n", indent_str));

        if let Some(getter) = &prop.getter {
            output.push_str(&format!("{}    get => {};\n", indent_str, getter));
        }

        if let Some(setter) = &prop.setter {
            output.push_str(&format!("{}    set => {};\n", indent_str, setter));
        } else if let Some(init) = &prop.init {
            output.push_str(&format!("{}    init => {};\n", indent_str, init));
        }

        output.push_str(&format!("}}\n\n"));
    }

    fn generate_dotnet10_field(&self, field: &DotNet10Field, output: &mut String, indent: usize) {
        let indent_str = "    ".repeat(indent);
        let access = self.access_to_string(field.access);
        let static_str = if field.is_static { "static " } else { "" };
        let readonly_str = if field.is_readonly { "readonly " } else { "" };
        let volatile_str = if field.is_volatile { "volatile " } else { "" };

        output.push_str(&format!(
            "{}{}{}{}{} {}: {}",
            indent_str,
            access,
            static_str,
            readonly_str,
            volatile_str,
            field.name,
            self.type_to_string(&field.field_type)
        ));

        if let Some(default) = &field.default_value {
            output.push_str(&format!(" = {}", default));
        }

        output.push_str(";\n");
    }

    fn generate_dotnet10_function(
        &self,
        func: &DotNet10Function,
        output: &mut String,
        indent: usize,
    ) {
        let indent_str = "    ".repeat(indent);
        let access = self.access_to_string(func.access);
        let static_str = if func.is_static { "static " } else { "" };
        let virtual_str = if func.is_virtual { "virtual " } else { "" };
        let override_str = if func.is_override { "override " } else { "" };
        let async_str = if func.is_async { "async " } else { "" };
        let partial_str = if func.is_partial { "partial " } else { "" };
        let extension_str = if func.is_extension { "this " } else { "" };

        for attr in &func.attributes {
            output.push_str(&format!(
                "{}@{}\n",
                indent_str,
                self.attribute_to_string(attr)
            ));
        }

        output.push_str(&format!(
            "{}{}{}{}{}{}fn {}",
            indent_str, access, static_str, virtual_str, override_str, async_str, partial_str
        ));
        output.push_str(&format!("{}", extension_str));
        output.push_str(&func.name);

        if !func.type_params.is_empty() {
            output.push_str(&format!("<{}>", func.type_params.join(", ")));
        }

        output.push_str("(");
        let params: Vec<String> = func
            .params
            .iter()
            .map(|p| self.format_parameter(p))
            .collect();
        output.push_str(&params.join(", "));
        output.push_str(")");

        output.push_str(&format!(" -> {}", self.type_to_string(&func.return_type)));

        if !func.where_constraints.is_empty() {
            output.push_str(&format!(
                " where {}",
                func.where_constraints.join(" where ")
            ));
        }

        if let Some(body) = &func.body {
            output.push_str(" {\n");
            output.push_str(&format!("{}    ", indent_str));
            output.push_str(body.replace("\n", &format!("\n{}    ", indent_str)));
            output.push_str("\n");
            output.push_str(&format!("{}}}\n\n", indent_str));
        } else {
            output.push_str(";\n\n");
        }
    }

    fn format_parameter(&self, param: &DotNet10Parameter) -> String {
        let ref_keyword = if param.is_ref { "ref " } else { "" };
        let out_keyword = if param.is_out { "out " } else { "" };
        let in_keyword = if param.in_param { "in " } else { "" };
        let params_keyword = if param.is_params { "params " } else { "" };

        let param_type_str = if param.is_out || param.is_ref {
            format!("{}<{}>", self.type_to_string(&param.param_type), param.name)
        } else {
            format!("{}: {}", param.name, self.type_to_string(&param.param_type))
        };

        let default = if let Some(d) = &param.default_value {
            format!(" = {}", d)
        } else {
            String::new()
        };

        format!(
            "{}{}{}{}{}{}",
            ref_keyword, out_keyword, in_keyword, params_keyword, param_type_str, default
        )
    }

    fn access_to_string(&self, access: DotNet10Access) -> String {
        match access {
            DotNet10Access::Public => "public".to_string(),
            DotNet10Access::Internal => "internal".to_string(),
            DotNet10Access::Protected => "protected".to_string(),
            DotNet10Access::Private => "private".to_string(),
            DotNet10Access::ProtectedInternal => "protected internal".to_string(),
            DotNet10Access::PrivateProtected => "private protected".to_string(),
        }
    }

    fn type_to_string(&self, ty: &DotNet10Type) -> String {
        match ty {
            DotNet10Type::Void => "void".to_string(),
            DotNet10Type::Boolean => "bool".to_string(),
            DotNet10Type::SByte => "sbyte".to_string(),
            DotNet10Type::Byte => "byte".to_string(),
            DotNet10Type::Int16 => "short".to_string(),
            DotNet10Type::UInt16 => "ushort".to_string(),
            DotNet10Type::Int32 => "int".to_string(),
            DotNet10Type::UInt32 => "uint".to_string(),
            DotNet10Type::Int64 => "long".to_string(),
            DotNet10Type::UInt64 => "ulong".to_string(),
            DotNet10Type::Single => "float".to_string(),
            DotNet10Type::Double => "double".to_string(),
            DotNet10Type::Decimal => "decimal".to_string(),
            DotNet10Type::Char => "char".to_string(),
            DotNet10Type::String => "string".to_string(),
            DotNet10Type::Object => "object".to_string(),
            DotNet10Type::Dynamic => "dynamic".to_string(),
            DotNet10Type::DateTime => "DateTime".to_string(),
            DotNet10Type::TimeSpan => "TimeSpan".to_string(),
            DotNet10Type::Guid => "Guid".to_string(),
            DotNet10Type::IntPtr => "nint".to_string(),
            DotNet10Type::UIntPtr => "nuint".to_string(),
            DotNet10Type::Nullable(inner) => format!("{}?", self.type_to_string(inner)),
            DotNet10Type::Array(inner) => format!("{}[]", self.type_to_string(inner)),
            DotNet10Type::List(inner) => format!("List<{}>", self.type_to_string(inner)),
            DotNet10Type::Dictionary(k, v) => {
                format!(
                    "Dictionary<{}, {}>",
                    self.type_to_string(k),
                    self.type_to_string(v)
                )
            }
            DotNet10Type::HashSet(inner) => format!("HashSet<{}>", self.type_to_string(inner)),
            DotNet10Type::Queue(inner) => format!("Queue<{}>", self.type_to_string(inner)),
            DotNet10Type::Stack(inner) => format!("Stack<{}>", self.type_to_string(inner)),
            DotNet10Type::Task => "Task".to_string(),
            DotNet10Type::TaskT(t) => format!("Task<{}>", self.type_to_string(t)),
            DotNet10Type::ValueTask => "ValueTask".to_string(),
            DotNet10Type::ValueTaskT(t) => format!("ValueTask<{}>", self.type_to_string(t)),
            DotNet10Type::Func(params, ret) => {
                let param_strs: Vec<String> =
                    params.iter().map(|t| self.type_to_string(t)).collect();
                format!(
                    "Func<{}, {}>",
                    param_strs.join(", "),
                    self.type_to_string(ret)
                )
            }
            DotNet10Type::Action(params) => {
                let param_strs: Vec<String> =
                    params.iter().map(|t| self.type_to_string(t)).collect();
                if param_strs.is_empty() {
                    "Action".to_string()
                } else {
                    format!("Action<{}>", param_strs.join(", "))
                }
            }
            DotNet10Type::IEnumerable(inner) => {
                format!("IEnumerable<{}>", self.type_to_string(inner))
            }
            DotNet10Type::IEnumerator(inner) => {
                format!("IEnumerator<{}>", self.type_to_string(inner))
            }
            DotNet10Type::IAsyncEnumerable(inner) => {
                format!("IAsyncEnumerable<{}>", self.type_to_string(inner))
            }
            DotNet10Type::IAsyncEnumerator(inner) => {
                format!("IAsyncEnumerator<{}>", self.type_to_string(inner))
            }
            DotNet10Type::IDisposable => "IDisposable".to_string(),
            DotNet10Type::IAsyncDisposable => "IAsyncDisposable".to_string(),
            DotNet10Type::Struct(name) => name.clone(),
            DotNet10Type::Record(name) => name.clone(),
            DotNet10Type::RecordStruct(name) => name.clone(),
            DotNet10Type::Class(name) => name.clone(),
            DotNet10Type::Interface(name) => name.clone(),
            DotNet10Type::Enum(name) => name.clone(),
            DotNet10Type::Delegate(name, _, _) => name.clone(),
            DotNet10Type::Tuple(types) => {
                let type_strs: Vec<String> = types.iter().map(|t| self.type_to_string(t)).collect();
                format!("({})", type_strs.join(", "))
            }
            DotNet10Type::ValueTuple(types) => {
                let type_strs: Vec<String> = types.iter().map(|t| self.type_to_string(t)).collect();
                format!("({})", type_strs.join(", "))
            }
            DotNet10Type::Span(inner) => format!("Span<{}>", self.type_to_string(inner)),
            DotNet10Type::ReadOnlySpan(inner) => {
                format!("ReadOnlySpan<{}>", self.type_to_string(inner))
            }
            DotNet10Type::Memory(inner) => format!("Memory<{}>", self.type_to_string(inner)),
            DotNet10Type::ReadOnlyMemory(inner) => {
                format!("ReadOnlyMemory<{}>", self.type_to_string(inner))
            }
            DotNet10Type::Index => "Index".to_string(),
            DotNet10Type::Range => "Range".to_string(),
            DotNet10Type::Utf8String => "Utf8String".to_string(),
            _ => "object".to_string(),
        }
    }

    fn attribute_to_string(&self, attr: &DotNet10Attribute) -> String {
        if attr.arguments.is_empty() {
            attr.name.clone()
        } else {
            format!("{}({})", attr.name, attr.arguments.join(", "))
        }
    }

    pub fn set_options(&mut self, options: DotNet10FFIOptions) {
        self.options = options;
    }
}

#[derive(Debug, Clone)]
pub enum DotNet10Declaration {
    Class(DotNet10Class),
    Struct(DotNet10Struct),
    Record(DotNet10Record),
    Interface(DotNet10Interface),
    Enum(DotNet10Enum),
    Delegate(DotNet10Delegate),
}

pub fn map_chim_type_to_dotnet10(chim_type: &crate::Type) -> DotNet10Type {
    match chim_type {
        crate::Type::CVoid => DotNet10Type::Void,
        crate::Type::CBool => DotNet10Type::Boolean,
        crate::Type::CChar => DotNet10Type::SByte,
        crate::Type::CShort => DotNet10Type::Int16,
        crate::Type::CInt => DotNet10Type::Int32,
        crate::Type::CLong => DotNet10Type::Int64,
        crate::Type::CLongLong => DotNet10Type::Int64,
        crate::Type::CUChar => DotNet10Type::Byte,
        crate::Type::CUShort => DotNet10Type::UInt16,
        crate::Type::CUInt => DotNet10Type::UInt32,
        crate::Type::CULong => DotNet10Type::UInt64,
        crate::Type::CULongLong => DotNet10Type::UInt64,
        crate::Type::CFloat => DotNet10Type::Single,
        crate::Type::CDouble => DotNet10Type::Double,
        crate::Type::CStr => DotNet10Type::String,
        crate::Type::CVoidPtr => DotNet10Type::IntPtr,
        crate::Type::ISize => DotNet10Type::IntPtr,
        crate::Type::USize => DotNet10Type::UIntPtr,
        crate::Type::Pointer { target, .. } => {
            DotNet10Type::Pointer(Box::new(map_chim_type_to_dotnet10(target)))
        }
        crate::Type::Array { element, length: _ } => {
            DotNet10Type::Array(Box::new(map_chim_type_to_dotnet10(element)))
        }
        crate::Type::Function { params, ret, .. } => {
            let param_types: Vec<DotNet10Type> = params
                .iter()
                .map(|p| map_chim_type_to_dotnet10(p))
                .collect();
            let ret_type = Box::new(map_chim_type_to_dotnet10(ret));
            DotNet10Type::Delegate(String::from("Func"), param_types, ret_type)
        }
        crate::Type::Struct { name, .. } => DotNet10Type::Struct(name.to_string()),
        _ => DotNet10Type::Object,
    }
}
