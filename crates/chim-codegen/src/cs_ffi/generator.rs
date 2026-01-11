use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CSharpType {
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
    Nullable(Box<CSharpType>),
    Array(Box<CSharpType>),
    List(Box<CSharpType>),
    Dictionary(Box<CSharpType>, Box<CSharpType>),
    HashSet(Box<CSharpType>),
    Queue(Box<CSharpType>),
    Stack(Box<CSharpType>),
    LinkedList(Box<CSharpType>),
    SortedSet(Box<CSharpType>),
    SortedDictionary(Box<CSharpType>, Box<CSharpType>),
    ConcurrentDictionary(Box<CSharpType>, Box<CSharpType>),
    ConcurrentQueue(Box<CSharpType>),
    ConcurrentStack(Box<CSharpType>),
    Task,
    TaskT(Box<CSharpType>),
    ValueTask,
    ValueTaskT(Box<CSharpType>),
    Tuple(Vec<CSharpType>),
    ValueTuple(Vec<CSharpType>),
    Func(Vec<CSharpType>, Box<CSharpType>),
    Action(Vec<CSharpType>),
    Predicate(Box<CSharpType>),
    Comparison(Box<CSharpType>),
    Converter(Box<CSharpType>, Box<CSharpType>),
    IEnumerable(Box<CSharpType>),
    IEnumerator(Box<CSharpType>),
    ICollection(Box<CSharpType>),
    IList(Box<CSharpType>),
    ISet(Box<CSharpType>),
    IDictionary(Box<CSharpType>, Box<CSharpType>),
    IComparable(Box<CSharpType>),
    IEquatable(Box<CSharpType>),
    IEqualityComparer(Box<CSharpType>),
    IComparer(Box<CSharpType>),
    IDisposable,
    IAsyncDisposable,
    ICloneable,
    IFormattable,
    IConvertible,
    IFormatProvider,
    IFormatter,
    ISerializable,
    Attribute,
    Exception,
    ArgumentException,
    ArgumentNullException,
    ArgumentOutOfRangeException,
    NullReferenceException,
    InvalidOperationException,
    NotSupportedException,
    NotImplementedException,
    IndexOutOfRangeException,
    DivideByZeroException,
    OverflowException,
    StackOverflowException,
    OutOfMemoryException,
    AccessViolationException,
    Custom(String),
    Struct(String, Vec<(String, CSharpType)>),
    Class(String, Vec<(String, CSharpType)>),
    Record(String, Vec<(String, CSharpType)>),
    Interface(String, Vec<(String, CSharpType)>),
    Enum(String, CSharpType, Vec<(String, i64)>),
    Delegate(String, Vec<CSharpType>, Box<CSharpType>),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CSharpModifier {
    Static,
    Abstract,
    Sealed,
    Partial,
    New,
    Override,
    Virtual,
    ReadOnly,
    Const,
    Volatile,
    Unsafe,
    Extern,
    Async,
}

#[derive(Debug, Clone)]
pub struct CSharpMethod {
    pub name: String,
    pub return_type: CSharpType,
    pub parameters: Vec<CSharpParameter>,
    pub modifiers: Vec<CSharpModifier>,
    pub access: CSharpAccess,
    pub is_extension: bool,
    pub is_partial: bool,
    pub is_unsafe: bool,
    pub body: Option<String>,
    pub attributes: Vec<CSharpAttribute>,
    pub type_parameters: Vec<String>,
    pub constraints: Vec<CSharpConstraint>,
    pub throw_exceptions: Vec<CSharpType>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CSharpAccess {
    Public,
    Internal,
    Protected,
    Private,
    ProtectedInternal,
    PrivateProtected,
}

#[derive(Debug, Clone)]
pub struct CSharpParameter {
    pub name: String,
    pub param_type: CSharpType,
    pub is_params: bool,
    pub is_ref: bool,
    pub is_out: bool,
    pub is_in: bool,
    pub default_value: Option<String>,
    pub attributes: Vec<CSharpAttribute>,
}

#[derive(Debug, Clone)]
pub struct CSharpField {
    pub name: String,
    pub field_type: CSharpType,
    pub modifiers: Vec<CSharpModifier>,
    pub access: CSharpAccess,
    pub initializer: Option<String>,
    pub attributes: Vec<CSharpAttribute>,
}

#[derive(Debug, Clone)]
pub struct CSharpProperty {
    pub name: String,
    pub property_type: CSharpType,
    pub getter: Option<CSharpMethod>,
    pub setter: Option<CSharpMethod>,
    pub modifiers: Vec<CSharpModifier>,
    pub access: CSharpAccess,
    pub is_indexer: bool,
    pub is_auto_property: bool,
    pub initializer: Option<String>,
    pub attributes: Vec<CSharpAttribute>,
}

#[derive(Debug, Clone)]
pub struct CSharpEvent {
    pub name: String,
    pub event_type: CSharpType,
    pub add_method: Option<CSharpMethod>,
    pub remove_method: Option<CSharpMethod>,
    pub modifiers: Vec<CSharpModifier>,
    pub access: CSharpAccess,
    pub attributes: Vec<CSharpAttribute>,
}

#[derive(Debug, Clone)]
pub struct CSharpConstructor {
    pub parameters: Vec<CSharpParameter>,
    pub modifiers: Vec<CSharpModifier>,
    pub access: CSharpAccess,
    pub initializer: ConstructorInitializer,
    pub body: Option<String>,
    pub attributes: Vec<CSharpAttribute>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ConstructorInitializer {
    None,
    This(Vec<String>),
    Base(Vec<String>),
}

#[derive(Debug, Clone)]
pub struct CSharpIndexer {
    pub parameters: Vec<CSharpParameter>,
    pub property_type: CSharpType,
    pub getter: Option<CSharpMethod>,
    pub setter: Option<CSharpMethod>,
    pub modifiers: Vec<CSharpModifier>,
    pub access: CSharpAccess,
}

#[derive(Debug, Clone)]
pub struct CSharpClass {
    pub name: String,
    pub namespace: String,
    pub modifiers: Vec<CSharpModifier>,
    pub access: CSharpAccess,
    pub base_class: Option<String>,
    pub interfaces: Vec<String>,
    pub type_parameters: Vec<String>,
    pub constraints: Vec<CSharpConstraint>,
    pub fields: Vec<CSharpField>,
    pub properties: Vec<CSharpProperty>,
    pub methods: Vec<CSharpMethod>,
    pub constructors: Vec<CSharpConstructor>,
    pub events: Vec<CSharpEvent>,
    pub indexers: Vec<CSharpIndexer>,
    pub nested_types: Vec<CSharpClass>,
    pub attributes: Vec<CSharpAttribute>,
    pub is_record: bool,
    pub is_struct: bool,
    pub is_interface: bool,
    pub is_enum: bool,
    pub is_partial: bool,
}

#[derive(Debug, Clone)]
pub struct CSharpStruct {
    pub name: String,
    pub namespace: String,
    pub modifiers: Vec<CSharpModifier>,
    pub access: CSharpAccess,
    pub interfaces: Vec<String>,
    pub fields: Vec<CSharpField>,
    pub properties: Vec<CSharpProperty>,
    pub methods: Vec<CSharpMethod>,
    pub constructors: Vec<CSharpConstructor>,
    pub indexers: Vec<CSharpIndexer>,
    pub attributes: Vec<CSharpAttribute>,
    pub is_readonly: bool,
    pub is_ref: bool,
}

#[derive(Debug, Clone)]
pub struct CSharpInterface {
    pub name: String,
    pub namespace: String,
    pub base_interfaces: Vec<String>,
    pub type_parameters: Vec<String>,
    pub constraints: Vec<CSharpConstraint>,
    pub methods: Vec<CSharpMethod>,
    pub properties: Vec<CSharpProperty>,
    pub events: Vec<CSharpEvent>,
    pub indexers: Vec<CSharpIndexer>,
    pub attributes: Vec<CSharpAttribute>,
    pub is_functional: bool,
}

#[derive(Debug, Clone)]
pub struct CSharpEnum {
    pub name: String,
    pub namespace: String,
    pub underlying_type: CSharpType,
    pub values: Vec<(String, i64)>,
    pub access: CSharpAccess,
    pub attributes: Vec<CSharpAttribute>,
    pub is_flags: bool,
}

#[derive(Debug, Clone)]
pub struct CSharpDelegate {
    pub name: String,
    pub namespace: String,
    pub return_type: CSharpType,
    pub parameters: Vec<CSharpParameter>,
    pub access: CSharpAccess,
    pub attributes: Vec<CSharpAttribute>,
    pub is_multicast: bool,
}

#[derive(Debug, Clone)]
pub struct CSharpNamespace {
    pub name: String,
    pub classes: Vec<CSharpClass>,
    pub structs: Vec<CSharpStruct>,
    pub interfaces: Vec<CSharpInterface>,
    pub enums: Vec<CSharpEnum>,
    pub delegates: Vec<CSharpDelegate>,
    pub aliases: Vec<(String, String)>,
}

#[derive(Debug, Clone)]
pub struct CSharpAttribute {
    pub name: String,
    pub arguments: Vec<(String, String)>,
    pub named_arguments: Vec<(String, String, String)>,
}

#[derive(Debug, Clone)]
pub struct CSharpConstraint {
    pub type_parameter: String,
    pub constraints: Vec<CSharpConstraintType>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CSharpConstraintType {
    Class,
    ClassName(String),
    Struct,
    New,
    unmanaged,
    notnull,
    default,
    Enum,
    unmanagedRestriction,
}

#[derive(Debug, Clone)]
pub struct DllImport {
    pub method_name: String,
    pub dll_name: String,
    pub entry_point: Option<String>,
    pub charset: Charset,
    pub set_last_error: bool,
    pub exact_spelling: bool,
    pub calling_convention: CallingConvention,
    pub preserve_sig: bool,
    pub best_fit_mapping: bool,
    pub throw_on_unmappable_char: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Charset {
    Ansi,
    Unicode,
    Auto,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CallingConvention {
    Cdecl,
    StdCall,
    ThisCall,
    FastCall,
    MarshalAs(Vec<UnmanagedType>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UnmanagedType {
    Bool,
    I1,
    U1,
    I2,
    U2,
    I4,
    U4,
    I8,
    U8,
    R4,
    R8,
    CStr,
    BStr,
    LPStr,
    LPWStr,
    LPTStr,
    ByValTStr,
    ByValArray,
    SysInt,
    SysUInt,
    VBByRefStr,
    AnsiBStr,
    TBStr,
    VariantBool,
    FunctionPtr,
    SafeArray,
    FixedArray,
    FixedSysString,
    BStr,
    LPStruct,
    IInspectable,
    HString,
    TypeName,
    Interface,
}

#[derive(Debug, Clone)]
pub struct CSharpGeneratorOptions {
    pub language_version: CSharpVersion,
    pub target_framework: TargetFramework,
    pub enable_nullable_reference_types: bool,
    pub enable_annotations: bool,
    pub generate_xml_docs: bool,
    pub use_records: bool,
    public_enums_by_default: bool,
    pub use_property_syntax: bool,
    pub use_expression_bodied_members: bool,
    pub use_pattern_matching: bool,
    pub use_switch_expressions: bool,
    pub use_global_using_directives: bool,
    pub implicit_usings: bool,
    pub file_scoped_namespaces: bool,
    pub record_structs: bool,
    pub init_only_setters: bool,
    pub static_abstract_members: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CSharpVersion {
    CSharp1,
    CSharp2,
    CSharp3,
    CSharp4,
    CSharp5,
    CSharp6,
    CSharp7,
    CSharp71,
    CSharp8,
    CSharp9,
    CSharp10,
    CSharp11,
    CSharp12,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TargetFramework {
    net48,
    netstandard20,
    netstandard21,
    netcoreapp31,
    net50,
    net60,
    net70,
    net80,
    net90,
}

impl Default for CSharpGeneratorOptions {
    fn default() -> Self {
        Self {
            language_version: CSharpVersion::CSharp11,
            target_framework: TargetFramework::net80,
            enable_nullable_reference_types: true,
            enable_annotations: true,
            generate_xml_docs: true,
            use_records: true,
            public_enums_by_default: true,
            use_property_syntax: true,
            use_expression_bodied_members: true,
            use_pattern_matching: true,
            use_switch_expressions: true,
            use_global_using_directives: true,
            implicit_usings: true,
            file_scoped_namespaces: false,
            record_structs: true,
            init_only_setters: true,
            static_abstract_members: true,
        }
    }
}

#[derive(Debug, Default)]
pub struct CSharpFFIGenerator {
    namespaces: HashMap<String, CSharpNamespace>,
    types: HashMap<String, CSharpType>,
    dll_imports: Vec<DllImport>,
    options: CSharpGeneratorOptions,
    using_directives: Vec<String>,
    global_using_directives: Vec<String>,
}

impl CSharpFFIGenerator {
    pub fn new() -> Self {
        let mut generator = Self {
            namespaces: HashMap::new(),
            types: HashMap::new(),
            dll_imports: Vec::new(),
            options: CSharpGeneratorOptions::default(),
            using_directives: Vec::new(),
            global_using_directives: Vec::new(),
        };
        generator.init_builtin_types();
        generator.init_using_directives();
        generator
    }

    fn init_builtin_types(&mut self) {
        self.types.insert("void".to_string(), CSharpType::Void);
        self.types.insert("bool".to_string(), CSharpType::Boolean);
        self.types.insert("sbyte".to_string(), CSharpType::SByte);
        self.types.insert("byte".to_string(), CSharpType::Byte);
        self.types.insert("short".to_string(), CSharpType::Int16);
        self.types.insert("ushort".to_string(), CSharpType::UInt16);
        self.types.insert("int".to_string(), CSharpType::Int32);
        self.types.insert("uint".to_string(), CSharpType::UInt32);
        self.types.insert("long".to_string(), CSharpType::Int64);
        self.types.insert("ulong".to_string(), CSharpType::UInt64);
        self.types.insert("float".to_string(), CSharpType::Single);
        self.types.insert("double".to_string(), CSharpType::Double);
        self.types.insert("decimal".to_string(), CSharpType::Decimal);
        self.types.insert("char".to_string(), CSharpType::Char);
        self.types.insert("string".to_string(), CSharpType::String);
        self.types.insert("object".to_string(), CSharpType::Object);
        self.types.insert("dynamic".to_string(), CSharpType::Dynamic);
        self.types.insert("DateTime".to_string(), CSharpType::DateTime);
        self.types.insert("TimeSpan".to_string(), CSharpType::TimeSpan);
        self.types.insert("Guid".to_string(), CSharpType::Guid);
        self.types.insert("IntPtr".to_string(), CSharpType::IntPtr);
        self.types.insert("UIntPtr".to_string(), CSharpType::UIntPtr);
        self.types.insert("Task".to_string(), CSharpType::Task);
        self.types.insert("IDisposable".to_string(), CSharpType::IDisposable);
        self.types.insert("IEnumerable".to_string(), CSharpType::IEnumerable(Box::new(CSharpType::Object)));
        self.types.insert("IEnumerator".to_string(), CSharpType::IEnumerator(Box::new(CSharpType::Object)));
        self.types.insert("IList".to_string(), CSharpType::IList(Box::new(CSharpType::Object)));
        self.types.insert("IDictionary".to_string(), CSharpType::IDictionary(Box::new(CSharpType::Object), Box::new(CSharpType::Object)));
    }

    fn init_using_directives(&mut self) {
        self.global_using_directives = vec![
            "System".to_string(),
            "System.Collections.Generic".to_string(),
            "System.Linq".to_string(),
            "System.Threading.Tasks".to_string(),
            "System.IO".to_string(),
        ];
        
        self.using_directives = vec![
            "System".to_string(),
            "System.Runtime.InteropServices".to_string(),
        ];
    }

    pub fn generate_file(&self, namespace: &str) -> String {
        let mut output = String::new();
        
        if self.options.use_global_using_directives {
            for directive in &self.global_using_directives {
                output.push_str(&format!("global using {};\n", directive));
            }
            output.push('\n');
        }
        
        if self.options.file_scoped_namespaces {
            output.push_str(&format!("namespace {};\n\n", namespace));
        } else {
            output.push_str(&format!("namespace {} {{\n", namespace));
        }
        
        let ns = self.namespaces.get(namespace);
        if let Some(n) = ns {
            for attr in &n.aliases {
                output.push_str(&format!("using {} = {};\n", attr.0, attr.1));
            }
            output.push('\n');
        }
        
        if !self.options.file_scoped_namespaces {
            output.push_str("}\n");
        }
        
        output
    }

    pub fn generate_class(&self, class: &CSharpClass) -> String {
        let mut output = String::new();
        
        self.generate_attributes(&class.attributes, &mut output);
        
        self.generate_class_declaration(class, &mut output);
        
        if !self.options.file_scoped_namespaces {
            output.push_str("}\n");
        }
        
        output
    }

    fn generate_class_declaration(&self, class: &CSharpClass, output: &mut String) {
        for directive in &self.using_directives {
            output.push_str(&format!("using {};\n", directive));
        }
        output.push('\n');
        
        if self.options.file_scoped_namespaces {
            output.push_str(&format!("namespace {};\n\n", class.namespace));
        } else {
            output.push_str(&format!("namespace {} {{\n", class.namespace));
        }
        
        let access_modifier = match class.access {
            CSharpAccess::Public => "public ",
            CSharpAccess::Internal => "internal ",
            CSharpAccess::Protected => "protected ",
            CSharpAccess::Private => "private ",
            CSharpAccess::ProtectedInternal => "protected internal ",
            CSharpAccess::PrivateProtected => "private protected ",
        };
        
        let modifiers: Vec<String> = class.modifiers.iter()
            .map(|m| match m {
                CSharpModifier::Static => "static",
                CSharpModifier::Abstract => "abstract",
                CSharpModifier::Sealed => "sealed",
                CSharpModifier::Partial => "partial",
                CSharpModifier::New => "new",
                _ => "",
            })
            .filter(|s| !s.is_empty())
            .collect();
        
        let class_keyword = if class.is_struct {
            "struct"
        } else if class.is_interface {
            "interface"
        } else if class.is_record {
            if self.options.record_structs { "record struct" } else { "record" }
        } else if class.is_enum {
            "enum"
        } else {
            "class"
        };
        
        output.push_str(access_modifier);
        if !modifiers.is_empty() {
            output.push_str(&modifiers.join(" "));
            output.push(' ');
        }
        output.push_str(class_keyword);
        output.push(' ');
        output.push_str(&class.name);
        
        if !class.type_parameters.is_empty() {
            output.push('<');
            output.push_str(&class.type_parameters.join(", "));
            output.push('>');
        }
        
        if !class.constraints.is_empty() {
            output.push(' ');
            for constraint in &class.constraints {
                self.generate_constraint(constraint, output);
            }
        }
        
        if let Some(base) = &class.base_class {
            if !class.is_interface && !class.is_enum {
                output.push_str(" : ");
                output.push_str(base);
                if !class.interfaces.is_empty() {
                    output.push_str(", ");
                    output.push_str(&class.interfaces.join(", "));
                }
            }
        } else if !class.interfaces.is_empty() && !class.is_enum {
            output.push_str(" : ");
            output.push_str(&class.interfaces.join(", "));
        }
        
        output.push_str(" {\n");
        
        if class.is_enum {
            self.generate_enum_values(&class.fields, output);
        } else {
            for field in &class.fields {
                self.generate_field(field, output);
            }
            
            for property in &class.properties {
                self.generate_property(property, output);
            }
            
            for constructor in &class.constructors {
                self.generate_constructor(constructor, &class.name, output);
            }
            
            for method in &class.methods {
                self.generate_method(method, output);
            }
            
            for event in &class.events {
                self.generate_event(event, output);
            }
            
            for indexer in &class.indexers {
                self.generate_indexer(indexer, output);
            }
        }
        
        for nested in &class.nested_types {
            output.push_str("\n    ");
            let nested_str = self.generate_class(nested);
            for line in nested_str.lines() {
                output.push_str(line);
                output.push('\n');
            }
        }
        
        output.push_str("}\n");
    }

    fn generate_constraint(&self, constraint: &CSharpConstraint, output: &mut String) {
        output.push_str(&format!("where {} : ", constraint.type_parameter));
        let constraints: Vec<String> = constraint.constraints.iter()
            .map(|c| match c {
                CSharpConstraintType::Class => "class".to_string(),
                CSharpConstraintType::ClassName(name) => name.clone(),
                CSharpConstraintType::Struct => "struct".to_string(),
                CSharpConstraintType::New => "new()".to_string(),
                CSharpConstraintType::unmanaged => "unmanaged".to_string(),
                CSharpConstraintType::notnull => "notnull".to_string(),
                CSharpConstraintType::default => "default".to_string(),
                CSharpConstraintType::Enum => "Enum".to_string(),
                CSharpConstraintType::unmanagedRestriction => "unmanaged".to_string(),
            })
            .collect();
        output.push_str(&constraints.join(", "));
    }

    fn generate_attributes(&self, attributes: &[CSharpAttribute], output: &mut String) {
        for attr in attributes {
            output.push('[');
            output.push_str(&attr.name);
            if !attr.arguments.is_empty() {
                output.push('(');
                let args: Vec<String> = attr.arguments.iter()
                    .map(|(n, v)| format!("{} = {}", n, v))
                    .collect();
                output.push_str(&args.join(", "));
                output.push(')');
            }
            output.push_str("]\n");
        }
    }

    fn generate_enum_values(&self, fields: &[CSharpField], output: &mut String) {
        for (i, field) in fields.iter().enumerate() {
            if i > 0 {
                output.push_str(",\n");
            }
            output.push_str(&format!("    {}", field.name));
            if let Some(init) = &field.initializer {
                output.push_str(&format!(" = {}", init));
            }
        }
        output.push_str("\n");
    }

    fn generate_field(&self, field: &CSharpField, output: &mut String) {
        for attr in &field.attributes {
            self.generate_single_attribute(attr, output);
        }
        
        let access = match field.access {
            CSharpAccess::Public => "    public ",
            CSharpAccess::Internal => "    internal ",
            CSharpAccess::Protected => "    protected ",
            CSharpAccess::Private => "    private ",
            CSharpAccess::ProtectedInternal => "    protected internal ",
            CSharpAccess::PrivateProtected => "    private protected ",
        };
        
        let modifiers: Vec<String> = field.modifiers.iter()
            .map(|m| match m {
                CSharpModifier::Static => "static",
                CSharpModifier::ReadOnly => "readonly",
                CSharpModifier::Const => "const",
                CSharpModifier::Volatile => "volatile",
                CSharpModifier::Unsafe => "unsafe",
                _ => "",
            })
            .filter(|s| !s.is_empty())
            .collect();
        
        output.push_str(access);
        if !modifiers.is_empty() {
            output.push_str(&modifiers.join(" "));
            output.push(' ');
        }
        output.push_str(&self.type_to_string(&field.field_type));
        output.push(' ');
        output.push_str(&field.name);
        
        if let Some(init) = &field.initializer {
            output.push_str(&format!(" = {}", init));
        }
        
        output.push_str(";\n");
    }

    fn generate_property(&self, property: &CSharpProperty, output: &mut String) {
        for attr in &property.attributes {
            self.generate_single_attribute(attr, output);
        }
        
        let access = match property.access {
            CSharpAccess::Public => "    public ",
            CSharpAccess::Internal => "    internal ",
            CSharpAccess::Protected => "    protected ",
            CSharpAccess::Private => "    private ",
            CSharpAccess::ProtectedInternal => "    protected internal ",
            CSharpAccess::PrivateProtected => "    private protected ",
        };
        
        let modifiers: Vec<String> = property.modifiers.iter()
            .map(|m| match m {
                CSharpModifier::Static => "static",
                CSharpModifier::Virtual => "virtual",
                CSharpModifier::Override => "override",
                CSharpModifier::Abstract => "abstract",
                CSharpModifier::Sealed => "sealed",
                _ => "",
            })
            .filter(|s| !s.is_empty())
            .collect();
        
        output.push_str(access);
        if !modifiers.is_empty() {
            output.push_str(&modifiers.join(" "));
            output.push(' ');
        }
        output.push_str(&self.type_to_string(&property.property_type));
        output.push(' ');
        output.push_str(&property.name);
        
        if property.is_indexer {
            output.push('[');
            let params: Vec<String> = property.getter.as_ref().unwrap().parameters.iter()
                .map(|p| format!("{} {}", self.type_to_string(&p.param_type), p.name))
                .collect();
            output.push_str(&params.join(", "));
            output.push(']');
        }
        
        if property.is_auto_property {
            output.push_str(" { get; set; }");
            if let Some(init) = &property.initializer {
                output.push_str(&format!(" = {};", init));
            } else {
                output.push_str(";\n");
            }
        } else {
            output.push_str(" {\n");
            if let Some(getter) = &property.getter {
                output.push_str(&format!("        get => {};\n", getter.body.as_ref().unwrap_or(&"".to_string())));
            }
            if let Some(setter) = &property.setter {
                output.push_str(&format!("        set => {};\n", setter.body.as_ref().unwrap_or(&"".to_string())));
            }
            output.push_str("    }\n");
        }
    }

    fn generate_method(&self, method: &CSharpMethod, output: &mut String) {
        for attr in &method.attributes {
            self.generate_single_attribute(attr, output);
        }
        
        let access = match method.access {
            CSharpAccess::Public => "    public ",
            CSharpAccess::Internal => "    internal ",
            CSharpAccess::Protected => "    protected ",
            CSharpAccess::Private => "    private ",
            CSharpAccess::ProtectedInternal => "    protected internal ",
            CSharpAccess::PrivateProtected => "    private protected ",
        };
        
        let modifiers: Vec<String> = method.modifiers.iter()
            .map(|m| match m {
                CSharpModifier::Static => "static",
                CSharpModifier::Virtual => "virtual",
                CSharpModifier::Override => "override",
                CSharpModifier::Abstract => "abstract",
                CSharpModifier::Sealed => "sealed",
                CSharpModifier::Async => "async",
                CSharpModifier::Extern => "extern",
                CSharpModifier::Unsafe => "unsafe",
                _ => "",
            })
            .filter(|s| !s.is_empty())
            .collect();
        
        output.push_str(access);
        if !modifiers.is_empty() {
            output.push_str(&modifiers.join(" "));
            output.push(' ');
        }
        if method.is_extension {
            output.push_str("static ");
        }
        output.push_str(&self.type_to_string(&method.return_type));
        output.push(' ');
        output.push_str(&method.name);
        
        if !method.type_parameters.is_empty() {
            output.push('<');
            output.push_str(&method.type_parameters.join(", "));
            output.push('>');
        }
        
        output.push('(');
        let params: Vec<String> = method.parameters.iter()
            .map(|p| self.param_to_string(p))
            .collect();
        output.push_str(&params.join(", "));
        output.push_str(")");
        
        if !method.throw_exceptions.is_empty() {
            let exceptions: Vec<String> = method.throw_exceptions.iter()
                .map(|t| self.type_to_string(t))
                .collect();
            output.push_str(&format!(" throws {}", exceptions.join(", ")));
        }
        
        if self.options.use_expression_bodied_members && method.body.is_some() {
            output.push_str(" => ");
            output.push_str(method.body.as_ref().unwrap());
            output.push_str(";\n");
        } else if method.is_unsafe {
            output.push_str(";\n");
        } else if let Some(body) = &method.body {
            output.push_str(" {\n");
            for line in body.lines() {
                output.push_str(&format!("        {}\n", line));
            }
            output.push_str("    }\n");
        } else {
            output.push_str(";\n");
        }
    }

    fn generate_constructor(&self, ctor: &CSharpConstructor, class_name: &str, output: &mut String) {
        for attr in &ctor.attributes {
            self.generate_single_attribute(attr, output);
        }
        
        let access = match ctor.access {
            CSharpAccess::Public => "    public ",
            CSharpAccess::Internal => "    internal ",
            CSharpAccess::Protected => "    protected ",
            CSharpAccess::Private => "    private ",
            CSharpAccess::ProtectedInternal => "    protected internal ",
            CSharpAccess::PrivateProtected => "    private protected ",
        };
        
        output.push_str(access);
        output.push_str(class_name);
        output.push('(');
        
        let params: Vec<String> = ctor.parameters.iter()
            .map(|p| self.param_to_string(p))
            .collect();
        output.push_str(&params.join(", "));
        output.push(')');
        
        match &ctor.initializer {
            ConstructorInitializer::This(args) if !args.is_empty() => {
                output.push_str(" : this(");
                output.push_str(&args.join(", "));
                output.push_str(")");
            }
            ConstructorInitializer::Base(args) if !args.is_empty() => {
                output.push_str(" : base(");
                output.push_str(&args.join(", "));
                output.push_str(")");
            }
            _ => {}
        }
        
        if let Some(body) = &ctor.body {
            output.push_str(" {\n");
            for line in body.lines() {
                output.push_str(&format!("        {}\n", line));
            }
            output.push_str("    }\n");
        } else {
            output.push_str(" {}\n");
        }
    }

    fn generate_event(&self, event: &CSharpEvent, output: &mut String) {
        for attr in &event.attributes {
            self.generate_single_attribute(attr, output);
        }
        
        let access = match event.access {
            CSharpAccess::Public => "    public ",
            CSharpAccess::Internal => "    internal ",
            CSharpAccess::Protected => "    protected ",
            CSharpAccess::Private => "    private ",
            _ => "    ",
        };
        
        let modifiers: Vec<String> = event.modifiers.iter()
            .map(|m| match m {
                CSharpModifier::Static => "static",
                CSharpModifier::Abstract => "abstract",
                CSharpModifier::Override => "override",
                _ => "",
            })
            .filter(|s| !s.is_empty())
            .collect();
        
        output.push_str(access);
        if !modifiers.is_empty() {
            output.push_str(&modifiers.join(" "));
            output.push(' ');
        }
        output.push_str("event ");
        output.push_str(&self.type_to_string(&event.event_type));
        output.push(' ');
        output.push_str(&event.name);
        output.push_str(";\n");
    }

    fn generate_indexer(&self, indexer: &CSharpIndexer, output: &mut String) {
        let access = match indexer.access {
            CSharpAccess::Public => "    public ",
            _ => "    ",
        };
        
        output.push_str(access);
        output.push_str(&self.type_to_string(&indexer.property_type));
        output.push_str(" this[");
        let params: Vec<String> = indexer.parameters.iter()
            .map(|p| self.param_to_string(p))
            .collect();
        output.push_str(&params.join(", "));
        output.push_str("] {\n");
        
        if let Some(getter) = &indexer.getter {
            output.push_str(&format!("        get => {};\n", getter.body.as_ref().unwrap_or(&"".to_string())));
        }
        if let Some(setter) = &indexer.setter {
            output.push_str(&format!("        set => {};\n", setter.body.as_ref().unwrap_or(&"".to_string())));
        }
        
        output.push_str("    }\n");
    }

    fn generate_single_attribute(&self, attr: &CSharpAttribute, output: &mut String) {
        output.push_str(&format!("[{}", attr.name));
        if !attr.arguments.is_empty() || !attr.named_arguments.is_empty() {
            output.push('(');
            let mut args: Vec<String> = attr.arguments.iter()
                .map(|(n, v)| format!("{} = {}", n, v))
                .collect();
            for (name, type_name, value) in &attr.named_arguments {
                args.push(format!("{} = {} = {}", name, type_name, value));
            }
            output.push_str(&args.join(", "));
            output.push(')');
        }
        output.push_str("]\n");
    }

    fn param_to_string(&self, param: &CSharpParameter) -> String {
        let mut s = String::new();
        
        if param.is_ref {
            s.push_str("ref ");
        } else if param.is_out {
            s.push_str("out ");
        } else if param.is_in {
            s.push_str("in ");
        }
        
        if param.is_params {
            s.push_str("params ");
        }
        
        s.push_str(&self.type_to_string(&param.param_type));
        s.push(' ');
        s.push_str(&param.name);
        
        if let Some(default) = &param.default_value {
            s.push_str(&format!(" = {}", default));
        }
        
        s
    }

    fn type_to_string(&self, ty: &CSharpType) -> String {
        match ty {
            CSharpType::Void => "void".to_string(),
            CSharpType::Boolean => "bool".to_string(),
            CSharpType::SByte => "sbyte".to_string(),
            CSharpType::Byte => "byte".to_string(),
            CSharpType::Int16 => "short".to_string(),
            CSharpType::UInt16 => "ushort".to_string(),
            CSharpType::Int32 => "int".to_string(),
            CSharpType::UInt32 => "uint".to_string(),
            CSharpType::Int64 => "long".to_string(),
            CSharpType::UInt64 => "ulong".to_string(),
            CSharpType::Single => "float".to_string(),
            CSharpType::Double => "double".to_string(),
            CSharpType::Decimal => "decimal".to_string(),
            CSharpType::Char => "char".to_string(),
            CSharpType::String => "string".to_string(),
            CSharpType::Object => "object".to_string(),
            CSharpType::Dynamic => "dynamic".to_string(),
            CSharpType::DateTime => "DateTime".to_string(),
            CSharpType::TimeSpan => "TimeSpan".to_string(),
            CSharpType::Guid => "Guid".to_string(),
            CSharpType::IntPtr => "nint".to_string(),
            CSharpType::UIntPtr => "nuint".to_string(),
            CSharpType::Nullable(inner) => format!("{}?", self.type_to_string(inner)),
            CSharpType::Array(inner) => format!("{}[]", self.type_to_string(inner)),
            CSharpType::List(inner) => format!("List<{}>", self.type_to_string(inner)),
            CSharpType::Dictionary(k, v) => {
                format!("Dictionary<{}, {}>", self.type_to_string(k), self.type_to_string(v))
            }
            CSharpType::HashSet(inner) => format!("HashSet<{}>", self.type_to_string(inner)),
            CSharpType::Task => "Task".to_string(),
            CSharpType::TaskT(inner) => format!("Task<{}>", self.type_to_string(inner)),
            CSharpType::ValueTask => "ValueTask".to_string(),
            CSharpType::ValueTaskT(inner) => format!("ValueTask<{}>", self.type_to_string(inner)),
            CSharpType::Tuple(types) => {
                let type_strs: Vec<String> = types.iter()
                    .map(|t| self.type_to_string(t))
                    .collect();
                format!("({})", type_strs.join(", "))
            }
            CSharpType::ValueTuple(types) => {
                let type_strs: Vec<String> = types.iter()
                    .map(|t| self.type_to_string(t))
                    .collect();
                format!("({})", type_strs.join(", "))
            }
            CSharpType::Func(args, ret) => {
                let arg_strs: Vec<String> = args.iter()
                    .map(|t| self.type_to_string(t))
                    .collect();
                format!("Func<{}, {}>", arg_strs.join(", "), self.type_to_string(ret))
            }
            CSharpType::Action(args) => {
                let arg_strs: Vec<String> = args.iter()
                    .map(|t| self.type_to_string(t))
                    .collect();
                if arg_strs.is_empty() {
                    "Action".to_string()
                } else {
                    format!("Action<{}>", arg_strs.join(", "))
                }
            }
            CSharpType::IEnumerable(inner) => format!("IEnumerable<{}>", self.type_to_string(inner)),
            CSharpType::IList(inner) => format!("IList<{}>", self.type_to_string(inner)),
            CSharpType::IDictionary(k, v) => {
                format!("IDictionary<{}, {}>", self.type_to_string(k), self.type_to_string(v))
            }
            CSharpType::IDisposable => "IDisposable".to_string(),
            CSharpType::IAsyncDisposable => "IAsyncDisposable".to_string(),
            CSharpType::Exception => "Exception".to_string(),
            CSharpType::ArgumentException => "ArgumentException".to_string(),
            CSharpType::Custom(name) => name.clone(),
            CSharpType::Struct(name, _) => name.clone(),
            CSharpType::Class(name, _) => name.clone(),
            CSharpType::Record(name, _) => name.clone(),
            CSharpType::Interface(name, _) => name.clone(),
            CSharpType::Enum(name, _, _) => name.clone(),
            CSharpType::Delegate(name, _, _) => name.clone(),
        }
    }

    pub fn generate_dll_import(&self, import: &DllImport) -> String {
        let mut output = String::new();
        
        output.push_str("[DllImport(\"");
        output.push_str(&import.dll_name);
        output.push_str("\"");
        
        if let Some(ep) = &import.entry_point {
            output.push_str(&format!(", EntryPoint = \"{}\"", ep));
        }
        
        match import.charset {
            Charset::Ansi => output.push_str(", CharSet = CharSet.Ansi"),
            Charset::Unicode => output.push_str(", CharSet = CharSet.Unicode"),
            Charset::Auto => output.push_str(", CharSet = CharSet.Auto"),
        }
        
        if import.set_last_error {
            output.push_str(", SetLastError = true");
        }
        
        if import.exact_spelling {
            output.push_str(", ExactSpelling = true");
        }
        
        match import.calling_convention {
            CallingConvention::Cdecl => output.push_str(", CallingConvention = CallingConvention.Cdecl"),
            CallingConvention::StdCall => output.push_str(", CallingConvention = CallingConvention.StdCall"),
            CallingConvention::ThisCall => output.push_str(", CallingConvention = CallingConvention.ThisCall"),
            CallingConvention::FastCall => output.push_str(", CallingConvention = CallingConvention.FastCall"),
            CallingConvention::MarshalAs(types) => {
                let type_strs: Vec<String> = types.iter()
                    .map(|t| format!("UnmanagedType::{:?}", t))
                    .collect();
                output.push_str(&format!(", CallingConvention = CallingConvention.Cdecl, MarshalAs({})", type_strs.join(", ")));
            }
        }
        
        if import.preserve_sig {
            output.push_str(", PreserveSig = true");
        }
        
        if import.best_fit_mapping {
            output.push_str(", BestFitMapping = true");
        }
        
        if import.throw_on_unmappable_char {
            output.push_str(", ThrowOnUnmappableChar = true");
        }
        
        output.push_str(")]\n");
        
        output
    }

    pub fn add_dll_import(&mut self, import: DllImport) {
        self.dll_imports.push(import);
    }

    pub fn get_dll_imports(&self) -> &[DllImport] {
        &self.dll_imports
    }
}

pub fn map_chim_type_to_cs(chim_type: &crate::Type) -> CSharpType {
    match chim_type {
        crate::Type::CVoid => CSharpType::Void,
        crate::Type::CBool => CSharpType::Boolean,
        crate::Type::CChar => CSharpType::Char,
        crate::Type::CShort => CSharpType::Int16,
        crate::Type::CInt => CSharpType::Int32,
        crate::Type::CLong => CSharpType::Int64,
        crate::Type::CLongLong => CSharpType::Int64,
        crate::Type::CUChar => CSharpType::Byte,
        crate::Type::CUShort => CSharpType::UInt16,
        crate::Type::CUInt => CSharpType::UInt32,
        crate::Type::CULong => CSharpType::UInt64,
        crate::Type::CULongLong => CSharpType::UInt64,
        crate::Type::CFloat => CSharpType::Single,
        crate::Type::CDouble => CSharpType::Double,
        crate::Type::CStr => CSharpType::String,
        crate::Type::CVoidPtr => CSharpType::IntPtr,
        crate::Type::ISize => CSharpType::Int64,
        crate::Type::USize => CSharpType::UInt64,
        crate::Type::Pointer { target, .. } => {
            if matches!(**target, crate::Type::CVoid) {
                CSharpType::IntPtr
            } else {
                CSharpType::IntPtr
            }
        }
        crate::Type::Array { element, length: _ } => {
            CSharpType::Array(Box::new(map_chim_type_to_cs(element)))
        }
        crate::Type::Function { params, ret, .. } => {
            let ret_type = Box::new(map_chim_type_to_cs(ret));
            let param_types: Vec<CSharpType> = params.iter()
                .map(|t| map_chim_type_to_cs(t))
                .collect();
            CSharpType::Func(param_types, ret_type)
        }
        crate::Type::Struct { name, .. } => CSharpType::Class(name.to_string(), vec![]),
        crate::Type::Tuple(types) => {
            if types.is_empty() {
                CSharpType::Tuple(vec![])
            } else {
                CSharpType::Tuple(types.iter().map(|t| map_chim_type_to_cs(t)).collect())
            }
        }
        _ => CSharpType::Object,
    }
}
