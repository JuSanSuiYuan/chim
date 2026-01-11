use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CPPType {
    Void,
    Bool,
    Char,
    Short,
    Int,
    Long,
    LongLong,
    UnsignedChar,
    UnsignedShort,
    UnsignedInt,
    UnsignedLong,
    UnsignedLongLong,
    Float,
    Double,
    LongDouble,
    Pointer(Box<CPPType>),
    Reference(Box<CPPType>),
    ConstPointer(Box<CPPType>),
    ConstReference(Box<CPPType>),
    RValueReference(Box<CPPType>),
    Array(Box<CPPType>, usize),
    FunctionPointer(Box<CPPType>, Vec<CPPType>),
    Struct(String, Vec<(String, CPPType)>),
    Class(String, Vec<(String, CPPType)>),
    Template(String, Vec<CPPType>),
    Enum(String),
    Auto,
   decltype(Expr),
    StdString,
    StdVector(Box<CPPType>),
    StdMap(Box<CPPType>, Box<CPPType>),
    StdUniquePtr(Box<CPPType>),
    StdSharedPtr(Box<CPPType>),
    StdWeakPtr(Box<CPPType>),
    StdOptional(Box<CPPType>),
    StdVariant(Vec<CPPType>),
    StdAny,
    StdFunction(Box<CPPType>, Box<CPPType>),
    ChronoDuration(i64),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CPPTemplate {
    Vector,
    Map,
    Set,
    UniquePtr,
    SharedPtr,
    WeakPtr,
    Optional,
    Variant,
    Function,
    Pair,
    Tuple,
    Array,
    Chrono,
}

#[derive(Debug, Clone)]
pub struct CPPMethod {
    pub name: String,
    pub is_static: bool,
    pub is_virtual: bool,
    pub is_const: bool,
    pub is_explicit: bool,
    pub is_inline: bool,
    pub access: AccessSpecifier,
    pub params: Vec<CPPParameter>,
    pub return_type: CPPType,
    pub body: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AccessSpecifier {
    Public,
    Protected,
    Private,
}

#[derive(Debug, Clone)]
pub struct CPPParameter {
    pub name: String,
    pub param_type: CPPType,
    pub is_const: bool,
    pub is_reference: bool,
    pub default_value: Option<String>,
}

#[derive(Debug, Clone)]
pub struct CPPClass {
    pub name: String,
    pub is_template: bool,
    pub template_params: Vec<String>,
    pub base_classes: Vec<CPPBaseClass>,
    pub fields: Vec<CPPField>,
    pub methods: Vec<CPPMethod>,
    pub constructors: Vec<CPPConstructor>,
    pub destructor: Option<CPPDestructor>,
    pub access: AccessSpecifier,
    pub is_final: bool,
    pub is_abstract: bool,
}

#[derive(Debug, Clone)]
pub struct CPPBaseClass {
    pub name: String,
    pub access: AccessSpecifier,
    pub is_virtual: bool,
}

#[derive(Debug, Clone)]
pub struct CPPField {
    pub name: String,
    pub field_type: CPPType,
    pub access: AccessSpecifier,
    pub is_static: bool,
    pub is_constexpr: bool,
    pub is_mutable: bool,
    pub initializer: Option<String>,
}

#[derive(Debug, Clone)]
pub struct CPPConstructor {
    pub name: String,
    pub params: Vec<CPPParameter>,
    pub is_explicit: bool,
    pub is_constexpr: bool,
    pub is_inline: bool,
    pub initializer_list: Vec<(String, String)>,
    pub body: Option<String>,
}

#[derive(Debug, Clone)]
pub struct CPPDestructor {
    pub name: String,
    pub is_virtual: bool,
    pub is_final: bool,
    pub is_inline: bool,
    pub body: Option<String>,
}

#[derive(Debug, Clone)]
pub struct CPPNamespace {
    pub name: String,
    pub classes: Vec<CPPClass>,
    pub functions: Vec<CPPFunction>,
    pub namespaces: Vec<CPPNamespace>,
    pub enums: Vec<CPPEnum>,
    pub using_declarations: Vec<String>,
    pub using_directives: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct CPPFunction {
    pub name: String,
    pub return_type: CPPType,
    pub params: Vec<CPPParameter>,
    pub is_static: bool,
    pub is_inline: bool,
    pub is_extern: bool,
    pub is_constexpr: bool,
    pub is_noexcept: bool,
    pub is_virtual: bool,
    pub is_override: bool,
    pub is_final: bool,
    pub template_params: Vec<CPPTemplateParam>,
    pub body: Option<String>,
    pub linkage: LinkageType,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CPPTemplateParam {
    pub name: String,
    pub default_type: Option<CPPType>,
    pub default_value: Option<String>,
    pub is_type_param: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum LinkageType {
    External,
    Internal,
    NoLinkage,
}

#[derive(Debug, Clone)]
pub struct CPPEnum {
    pub name: String,
    pub underlying_type: CPPType,
    pub is_enum_class: bool,
    pub is_scoped: bool,
    pub values: Vec<(String, Option<String>)>,
}

#[derive(Debug, Clone)]
pub struct CPPInclude {
    pub path: String,
    pub is_system: bool,
    pub is_angled: bool,
}

#[derive(Debug, Clone)]
pub struct CPPExternBlock {
    pub linkage: LinkageType,
    pub declarations: Vec<CPPDeclaration>,
}

#[derive(Debug, Clone)]
pub enum CPPDeclaration {
    Function(CPPFunction),
    Class(CPPClass),
    Variable(String, CPPType, Option<String>),
    Enum(CPPEnum),
    Namespace(CPPNamespace),
    TemplateDeclaration(CPPTemplateDeclaration),
    StaticAssert(String, String),
    Typedef(String, CPPType),
    UsingDeclaration(String, CPPType),
    UsingDirective(String),
}

#[derive(Debug, Clone)]
pub struct CPPTemplateDeclaration {
    pub params: Vec<CPPTemplateParam>,
    pub declaration: Box<CPPDeclaration>,
}

#[derive(Debug, Clone)]
pub struct CPPGeneratorOptions {
    pub cxx_standard: CXXStandard,
    pub generate_namespaced_code: bool,
    pub generate_constructors: bool,
    pub generate_destructors: bool,
    pub generate_accessors: bool,
    pub generate_virtual_table: bool,
    pub enable_exceptions: bool,
    pub enable_rtti: bool,
    pub use_std_lib: bool,
    pub generate_move_operations: bool,
    pub generate_copy_operations: bool,
    pub use_header_guards: bool,
    pub inline_small_methods: bool,
    pub enable_template_instantiation: bool,
    pub use_std_initializer_list: bool,
    pub enable_coroutines: bool,
    pub modules: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CXXStandard {
    CPP98,
    CPP03,
    CPP11,
    CPP14,
    CPP17,
    CPP20,
    CPP23,
    CPP26,
}

impl Default for CPPGeneratorOptions {
    fn default() -> Self {
        Self {
            cxx_standard: CXXStandard::CPP20,
            generate_namespaced_code: true,
            generate_constructors: true,
            generate_destructors: true,
            generate_accessors: true,
            generate_virtual_table: true,
            enable_exceptions: true,
            enable_rtti: true,
            use_std_lib: true,
            generate_move_operations: true,
            generate_copy_operations: true,
            use_header_guards: true,
            inline_small_methods: true,
            enable_template_instantiation: true,
            use_std_initializer_list: true,
            enable_coroutines: false,
            modules: false,
        }
    }
}

#[derive(Debug, Default)]
pub struct CPPFFIGenerator {
    types: HashMap<String, CPPType>,
    namespaces: HashMap<String, CPPNamespace>,
    includes: Vec<CPPInclude>,
    options: CPPGeneratorOptions,
    current_namespace: Option<String>,
    name_mangling: HashMap<String, String>,
}

impl CPPFFIGenerator {
    pub fn new() -> Self {
        let mut generator = Self {
            types: HashMap::new(),
            namespaces: HashMap::new(),
            includes: Vec::new(),
            options: CPPGeneratorOptions::default(),
            current_namespace: None,
            name_mangling: HashMap::new(),
        };
        generator.init_builtin_types();
        generator.init_std_types();
        generator
    }

    fn init_builtin_types(&mut self) {
        self.types.insert("void".to_string(), CPPType::Void);
        self.types.insert("bool".to_string(), CPPType::Bool);
        self.types.insert("char".to_string(), CPPType::Char);
        self.types.insert("short".to_string(), CPPType::Short);
        self.types.insert("int".to_string(), CPPType::Int);
        self.types.insert("long".to_string(), CPPType::Long);
        self.types.insert("long long".to_string(), CPPType::LongLong);
        self.types.insert("unsigned char".to_string(), CPPType::UnsignedChar);
        self.types.insert("unsigned short".to_string(), CPPType::UnsignedShort);
        self.types.insert("unsigned int".to_string(), CPPType::UnsignedInt);
        self.types.insert("unsigned long".to_string(), CPPType::UnsignedLong);
        self.types.insert("unsigned long long".to_string(), CPPType::UnsignedLongLong);
        self.types.insert("float".to_string(), CPPType::Float);
        self.types.insert("double".to_string(), CPPType::Double);
        self.types.insert("long double".to_string(), CPPType::LongDouble);
        self.types.insert("size_t".to_string(), CPPType::UnsignedLongLong);
        self.types.insert("int8_t".to_string(), CPPType::Char);
        self.types.insert("uint8_t".to_string(), CPPType::UnsignedChar);
        self.types.insert("int16_t".to_string(), CPPType::Short);
        self.types.insert("uint16_t".to_string(), CPPType::UnsignedShort);
        self.types.insert("int32_t".to_string(), CPPType::Int);
        self.types.insert("uint32_t".to_string(), CPPType::UnsignedInt);
        self.types.insert("int64_t".to_string(), CPPType::LongLong);
        self.types.insert("uint64_t".to_string(), CPPType::UnsignedLongLong);
        self.types.insert("intptr_t".to_string(), CPPType::LongLong);
        self.types.insert("uintptr_t".to_string(), CPPType::UnsignedLongLong);
    }

    fn init_std_types(&mut self) {
        self.types.insert("std::string".to_string(), CPPType::StdString);
        self.types.insert("std::string_view".to_string(), CPPType::ConstReference(Box::new(CPPType::StdString)));
        self.types.insert("std::vector<T>".to_string(), CPPType::StdVector(Box::new(CPPType::Pointer(Box::new(CPPType::Auto))));
        self.types.insert("std::map<K, V>".to_string(), CPPType::StdMap(Box::new(CPPType::Auto), Box::new(CPPType::Auto)));
        self.types.insert("std::set<T>".to_string(), CPPType::StdVector(Box::new(CPPType::Auto)));
        self.types.insert("std::unique_ptr<T>".to_string(), CPPType::StdUniquePtr(Box::new(CPPType::Auto)));
        self.types.insert("std::shared_ptr<T>".to_string(), CPPType::StdSharedPtr(Box::new(CPPType::Auto)));
        self.types.insert("std::weak_ptr<T>".to_string(), CPPType::StdWeakPtr(Box::new(CPPType::Auto)));
        self.types.insert("std::optional<T>".to_string(), CPPType::StdOptional(Box::new(CPPType::Auto)));
        self.types.insert("std::variant<Ts...>".to_string(), CPPType::StdVariant(vec![]));
        self.types.insert("std::any".to_string(), CPPType::StdAny);
        self.types.insert("std::function<R(Args...)>".to_string(), CPPType::StdFunction(Box::new(CPPType::Auto), Box::new(CPPType::Auto)));
        self.types.insert("std::chrono::duration<T>".to_string(), CPPType::ChronoDuration(0));
        self.includes.push(CPPInclude {
            path: "string".to_string(),
            is_system: true,
            is_angled: true,
        });
        self.includes.push(CPPInclude {
            path: "memory".to_string(),
            is_system: true,
            is_angled: true,
        });
        self.includes.push(CPPInclude {
            path: "vector".to_string(),
            is_system: true,
            is_angled: true,
        });
        self.includes.push(CPPInclude {
            path: "map".to_string(),
            is_system: true,
            is_angled: true,
        });
        self.includes.push(CPPInclude {
            path: "optional".to_string(),
            is_system: true,
            is_angled: true,
        });
        self.includes.push(CPPInclude {
            path: "variant".to_string(),
            is_system: true,
            is_angled: true,
        });
        self.includes.push(CPPInclude {
            path: "functional".to_string(),
            is_system: true,
            is_angled: true,
        });
    }

    pub fn generate_header(&self, class: &CPPClass) -> String {
        let mut output = String::new();
        
        if self.options.use_header_guards {
            let guard_name = self.generate_header_guard(&class.name);
            output.push_str(&format!("#ifndef {}\n", guard_name));
            output.push_str(&format!("#define {}\n\n", guard_name));
        }
        
        for include in &self.includes {
            if include.is_system {
                if include.is_angled {
                    output.push_str(&format!("#include <{}>\n", include.path));
                } else {
                    output.push_str(&format!("#include <{}>\n", include.path));
                }
            } else {
                output.push_str(&format!("#include \"{}\"\n", include.path));
            }
        }
        output.push('\n');
        
        if let Some(ns) = &self.current_namespace {
            output.push_str(&format!("namespace {} {{\n\n", ns));
        }
        
        self.generate_class_declaration(class, &mut output);
        
        if self.current_namespace.is_some() {
            output.push_str("\n} // namespace\n");
        }
        
        if self.options.use_header_guards {
            output.push_str(&format!("\n#endif // {}\n", self.generate_header_guard(&class.name)));
        }
        
        output
    }

    fn generate_header_guard(&self, name: &str) -> String {
        let mut guard = String::new();
        for c in name.chars() {
            if c.is_alphanumeric() || c == '_' {
                guard.push(c.to_ascii_uppercase());
            } else {
                guard.push('_');
            }
        }
        guard.push_str("_H");
        guard
    }

    fn generate_class_declaration(&self, class: &CPPClass, output: &mut String) {
        if class.is_template {
            self.generate_template_declaration(class, output);
            return;
        }
        
        if class.is_abstract {
            output.push_str("struct ");
        } else if class.access == AccessSpecifier::Public {
            output.push_str("class ");
        } else {
            output.push_str("struct ");
        }
        
        output.push_str(&class.name);
        
        if !class.base_classes.is_empty() {
            output.push_str(" : ");
            let bases: Vec<String> = class.base_classes.iter()
                .map(|base| {
                    let access = match base.access {
                        AccessSpecifier::Public => "public ",
                        AccessSpecifier::Protected => "protected ",
                        AccessSpecifier::Private => "private ",
                    };
                    format!("{}{}", access, base.name)
                })
                .collect();
            output.push_str(&bases.join(", "));
        }
        
        output.push_str(" {\npublic:\n");
        
        self.generate_constructors(class, output);
        self.generate_destructor(class, output);
        self.generate_methods(class, output);
        self.generate_fields(class, output);
        
        output.push_str("};\n");
    }

    fn generate_template_declaration(&self, class: &CPPClass, output: &mut String) {
        output.push_str("template<");
        let params: Vec<String> = class.template_params.iter()
            .map(|p| format!("typename {}", p))
            .collect();
        output.push_str(&params.join(", "));
        output.push_str(">\n");
        
        output.push_str("struct ");
        output.push_str(&class.name);
        output.push_str(" {\n");
        
        self.generate_fields(class, output);
        self.generate_methods(class, output);
        
        output.push_str("};\n");
    }

    fn generate_constructors(&self, class: &CPPClass, output: &mut String) {
        for ctor in &class.constructors {
            output.push_str(&format!("    {}(", class.name));
            
            let params: Vec<String> = ctor.params.iter()
                .map(|p| format!("{} {}", self.type_to_string(&p.param_type), p.name))
                .collect();
            output.push_str(&params.join(", "));
            output.push_str(")");
            
            if ctor.is_explicit {
                output.push_str(" explicit");
            }
            if ctor.is_constexpr {
                output.push_str(" constexpr");
            }
            if ctor.is_inline {
                output.push_str(" inline");
            }
            
            if !ctor.initializer_list.is_empty() {
                output.push_str(" : ");
                let inits: Vec<String> = ctor.initializer_list.iter()
                    .map(|(mem, val)| format!("{}({})", mem, val))
                    .collect();
                output.push_str(&inits.join(", "));
            }
            
            if let Some(body) = &ctor.body {
                output.push_str(" {\n");
                for line in body.lines() {
                    output.push_str(&format!("        {}\n", line));
                }
                output.push_str("    }\n");
            } else {
                output.push_str(";\n");
            }
        }
        
        if self.options.generate_copy_operations {
            output.push_str(&format!("    {}() = default;\n", class.name));
            output.push_str(&format!("    {}(const {}&) = default;\n", class.name, class.name));
            output.push_str(&format!("    {}& operator=(const {}&) = default;\n", class.name, class.name));
        }
        
        if self.options.generate_move_operations {
            output.push_str(&format!("    {}({}&&) = default;\n", class.name, class.name));
            output.push_str(&format!("    {}& operator=({}&&) = default;\n", class.name, class.name));
        }
    }

    fn generate_destructor(&self, class: &CPPClass, output: &mut String) {
        if let Some(dtor) = &class.destructor {
            output.push_str(&format!("    ~{}()", class.name));
            
            if dtor.is_virtual {
                output.push_str(" virtual");
            }
            if dtor.is_final {
                output.push_str(" final");
            }
            if dtor.is_inline {
                output.push_str(" inline");
            }
            
            if let Some(body) = &dtor.body {
                output.push_str(" {\n");
                for line in body.lines() {
                    output.push_str(&format!("        {}\n", line));
                }
                output.push_str("    }\n");
            } else {
                output.push_str(";\n");
            }
        } else if self.options.generate_destructors {
            output.push_str(&format!("    ~{}() = default;\n", class.name));
        }
    }

    fn generate_methods(&self, class: &CPPClass, output: &mut String) {
        for method in &class.methods {
            if method.access != AccessSpecifier::Public {
                continue;
            }
            
            if method.is_static {
                output.push_str("    static ");
            }
            if method.is_virtual {
                output.push_str("    virtual ");
            }
            if method.is_explicit {
                output.push_str("    explicit ");
            }
            if method.is_inline {
                output.push_str("    inline ");
            }
            
            output.push_str(&format!("{} ", self.type_to_string(&method.return_type)));
            output.push_str(&method.name);
            output.push_str("(");
            
            let params: Vec<String> = method.params.iter()
                .map(|p| {
                    let mut s = String::new();
                    if p.is_const {
                        s.push_str("const ");
                    }
                    s.push_str(&self.type_to_string(&p.param_type));
                    if p.is_reference {
                        s.push_str("&");
                    }
                    s.push(' ');
                    s.push_str(&p.name);
                    if let Some(default) = &p.default_value {
                        s.push_str(&format!(" = {}", default));
                    }
                    s
                })
                .collect();
            output.push_str(&params.join(", "));
            output.push_str(")");
            
            if method.is_const {
                output.push_str(" const");
            }
            
            if method.is_noexcept {
                output.push_str(" noexcept");
            }
            
            if method.body.is_some() {
                output.push_str(" {\n");
                output.push_str("        // TODO: implementation\n");
                output.push_str("    }\n");
            } else {
                output.push_str(";\n");
            }
        }
    }

    fn generate_fields(&self, class: &CPPClass, output: &mut String) {
        for field in &class.fields {
            if field.access != AccessSpecifier::Public {
                continue;
            }
            
            if field.is_static {
                output.push_str("    static ");
            }
            if field.is_constexpr {
                output.push_str("    constexpr ");
            }
            if field.is_mutable {
                output.push_str("    mutable ");
            }
            
            output.push_str(&format!("{} ", self.type_to_string(&field.field_type)));
            output.push_str(&field.name);
            
            if let Some(init) = &field.initializer {
                output.push_str(&format!(" = {}", init));
            }
            output.push_str(";\n");
        }
    }

    fn type_to_string(&self, ty: &CPPType) -> String {
        match ty {
            CPPType::Void => "void".to_string(),
            CPPType::Bool => "bool".to_string(),
            CPPType::Char => "char".to_string(),
            CPPType::Short => "short".to_string(),
            CPPType::Int => "int".to_string(),
            CPPType::Long => "long".to_string(),
            CPPType::LongLong => "long long".to_string(),
            CPPType::UnsignedChar => "unsigned char".to_string(),
            CPPType::UnsignedShort => "unsigned short".to_string(),
            CPPType::UnsignedInt => "unsigned int".to_string(),
            CPPType::UnsignedLong => "unsigned long".to_string(),
            CPPType::UnsignedLongLong => "unsigned long long".to_string(),
            CPPType::Float => "float".to_string(),
            CPPType::Double => "double".to_string(),
            CPPType::LongDouble => "long double".to_string(),
            CPPType::Pointer(inner) => format!("{}*", self.type_to_string(inner)),
            CPPType::Reference(inner) => format!("{}&", self.type_to_string(inner)),
            CPPType::ConstPointer(inner) => format!("const {}*", self.type_to_string(inner)),
            CPPType::ConstReference(inner) => format!("const {}&", self.type_to_string(inner)),
            CPPType::RValueReference(inner) => format!("{}&&", self.type_to_string(inner)),
            CPPType::Array(inner, size) => format!("{}[{}]", self.type_to_string(inner), size),
            CPPType::FunctionPointer(ret, params) => {
                let param_strs: Vec<String> = params.iter()
                    .map(|p| self.type_to_string(p))
                    .collect();
                format!("{} (*)({})", self.type_to_string(ret), param_strs.join(", "))
            }
            CPPType::Struct(name, _) | CPPType::Class(name, _) => name.clone(),
            CPPType::Enum(name) => format!("enum class {}", name),
            CPPType::Auto => "auto".to_string(),
            CPPType::StdString => "std::string".to_string(),
            CPPType::StdVector(inner) => format!("std::vector<{}>", self.type_to_string(inner)),
            CPPType::StdMap(key, value) => {
                format!("std::map<{}, {}>", self.type_to_string(key), self.type_to_string(value))
            }
            CPPType::StdUniquePtr(inner) => format!("std::unique_ptr<{}>", self.type_to_string(inner)),
            CPPType::StdSharedPtr(inner) => format!("std::shared_ptr<{}>", self.type_to_string(inner)),
            CPPType::StdWeakPtr(inner) => format!("std::weak_ptr<{}>", self.type_to_string(inner)),
            CPPType::StdOptional(inner) => format!("std::optional<{}>", self.type_to_string(inner)),
            CPPType::StdVariant(types) => {
                let type_strs: Vec<String> = types.iter()
                    .map(|t| self.type_to_string(t))
                    .collect();
                format!("std::variant<{}>", type_strs.join(", "))
            }
            CPPType::StdAny => "std::any".to_string(),
            CPPType::StdFunction(ret, args) => {
                format!("std::function<{}({})>", self.type_to_string(ret), self.type_to_string(args))
            }
            CPPType::ChronoDuration(secs) => format!("std::chrono::seconds({})", secs),
            _ => "void".to_string(),
        }
    }

    pub fn mangle_name(&self, name: &str, params: &[CPPType]) -> String {
        let mut mangled = String::from("_Z");
        
        fn encode_len(s: &mut String, len: usize) {
            if len < 10 {
                s.push_str(&len.to_string());
            } else if len < 26 {
                s.push((b'a' + (len - 10)) as char);
            } else {
                s.push_str(&format!("{}", len));
            }
        }
        
        encode_len(&mut mangled, name.len());
        mangled.push_str(name);
        
        for param in params {
            mangled.push_str(&self.mangle_type(param));
        }
        
        mangled
    }

    fn mangle_type(&self, ty: &CPPType) -> String {
        match ty {
            CPPType::Void => "v".to_string(),
            CPPType::Bool => "b".to_string(),
            CPPType::Char => "c".to_string(),
            CPPType::Short => "s".to_string(),
            CPPType::Int => "i".to_string(),
            CPPType::Long => "l".to_string(),
            CPPType::LongLong => "x".to_string(),
            CPPType::UnsignedChar => "h".to_string(),
            CPPType::UnsignedShort => "t".to_string(),
            CPPType::UnsignedInt => "j".to_string(),
            CPPType::UnsignedLong => "m".to_string(),
            CPPType::UnsignedLongLong => "y".to_string(),
            CPPType::Float => "f".to_string(),
            CPPType::Double => "d".to_string(),
            CPPType::LongDouble => "e".to_string(),
            CPPType::Pointer(inner) => format!("P{}", self.mangle_type(inner)),
            CPPType::Reference(inner) => format!("R{}", self.mangle_type(inner)),
            CPPType::ConstPointer(inner) => format!("PK{}", self.mangle_type(inner)),
            CPPType::ConstReference(inner) => format!("KR{}", self.mangle_type(inner)),
            CPPType::Struct(name, _) | CPPType::Class(name, _) => {
                format!("U{}", name)
            }
            _ => "v".to_string(),
        }
    }

    pub fn demangle_name(&self, mangled: &str) -> Option<(String, Vec<CPPType>)> {
        if !mangled.starts_with("_Z") {
            return None;
        }
        
        let mut pos = 2;
        let len = mangled[pos..].parse::<usize>().ok()?;
        pos += len.to_string().len();
        let name = &mangled[pos..pos+len];
        pos += len;
        
        let mut params = Vec::new();
        while pos < mangled.len() {
            let ty = self.demangle_type(&mangled[pos..])?;
            params.push(ty.0);
            pos += ty.1;
        }
        
        Some((name.to_string(), params))
    }

    fn demangle_type(&self, s: &str) -> Option<(CPPType, usize)> {
        let first = s.chars().next()?;
        let consumed = match first {
            'v' => (CPPType::Void, 1),
            'b' => (CPPType::Bool, 1),
            'c' => (CPPType::Char, 1),
            's' => (CPPType::Short, 1),
            'i' => (CPPType::Int, 1),
            'l' => (CPPType::Long, 1),
            'x' => (CPPType::LongLong, 1),
            'h' => (CPPType::UnsignedChar, 1),
            't' => (CPPType::UnsignedShort, 1),
            'j' => (CPPType::UnsignedInt, 1),
            'm' => (CPPType::UnsignedLong, 1),
            'y' => (CPPType::UnsignedLongLong, 1),
            'f' => (CPPType::Float, 1),
            'd' => (CPPType::Double, 1),
            'e' => (CPPType::LongDouble, 1),
            'P' => {
                let (ty, consumed) = self.demangle_type(&s[1..])?;
                (CPPType::Pointer(Box::new(ty)), consumed + 1)
            }
            'R' => {
                let (ty, consumed) = self.demangle_type(&s[1..])?;
                (CPPType::Reference(Box::new(ty)), consumed + 1)
            }
            'K' => {
                if s.len() > 1 && s.chars().nth(1) == Some('R') {
                    let (ty, consumed) = self.demangle_type(&s[2..])?;
                    (CPPType::ConstReference(Box::new(ty)), consumed + 2)
                } else {
                    let (ty, consumed) = self.demangle_type(&s[1..])?;
                    (CPPType::ConstPointer(Box::new(ty)), consumed + 1)
                }
            }
            'U' => {
                let mut end = 1;
                while end < s.len() && s.chars().nth(end).map(|c| c.is_alphanumeric() || c == '_') == Some(true) {
                    end += 1;
                }
                let name = &s[1..end];
                (CPPType::Struct(name.to_string(), vec![]), end)
            }
            _ => return None,
        };
        Some(consumed)
    }

    pub fn generate_constructor_call(&self, class_name: &str, args: &[String]) -> String {
        format!("{}({})", class_name, args.join(", "))
    }

    pub fn generate_destructor_call(&self, var_name: &str) -> String {
        format!("{}.~{}()", var_name, var_name.split('.').last().unwrap_or(var_name))
    }

    pub fn generate_raii_wrapper(&self, cpp_type: &CPPType, chim_type: &str) -> String {
        match cpp_type {
            CPPType::StdUniquePtr(_) => {
                format!(
                    r#"struct {} {{
    using WrapperType = std::unique_ptr<{}>;
    WrapperType ptr;
    
    {}() = default;
    explicit {}(void* p) : ptr(static_cast<{}*>(p)) {{}}
    
    static {} from_chim({} p) {{
        return {{reinterpret_cast<{}*>(p)}};
    }}
    
    {} to_chim() const {{
        return reinterpret_cast<{}>(ptr.get());
    }}
    
    void release() {{ ptr.release(); }}
    explicit operator bool() const {{ return ptr != nullptr; }}
}};"#,
                    chim_type,
                    self.type_to_string(cpp_type),
                    chim_type,
                    chim_type,
                    self.type_to_string(cpp_type),
                    chim_type,
                    chim_type,
                    chim_type,
                    chim_type
                )
            }
            CPPType::StdSharedPtr(_) => {
                format!(
                    r#"struct {} {{
    using WrapperType = std::shared_ptr<{}>;
    WrapperType ptr;
    
    {}() = default;
    explicit {}(void* p) : ptr(static_cast<{}*>(p)) {{}}
    
    static {} from_chim({} p) {{
        return {{reinterpret_cast<{}*>(p)}};
    }}
    
    {} to_chim() const {{
        return reinterpret_cast<{}>(ptr.get());
    }}
    
    void reset() {{ ptr.reset(); }}
    long use_count() const {{ return ptr.use_count(); }}
    explicit operator bool() const {{ return ptr != nullptr; }}
    
    {} operator*() const {{ return *ptr; }}
    {}* operator->() const {{ return ptr.get(); }}
}};"#,
                    chim_type,
                    self.type_to_string(cpp_type),
                    chim_type,
                    chim_type,
                    self.type_to_string(cpp_type),
                    chim_type,
                    chim_type,
                    chim_type,
                    chim_type,
                    chim_type,
                    chim_type
                )
            }
            _ => format!("// No RAII wrapper for type {}", self.type_to_string(cpp_type)),
        }
    }
}

pub fn map_chim_type_to_cpp(chim_type: &crate::Type) -> CPPType {
    match chim_type {
        crate::Type::CVoid => CPPType::Void,
        crate::Type::CBool => CPPType::Bool,
        crate::Type::CChar => CPPType::Char,
        crate::Type::CShort => CPPType::Short,
        crate::Type::CInt => CPPType::Int,
        crate::Type::CLong => CPPType::Long,
        crate::Type::CLongLong => CPPType::LongLong,
        crate::Type::CUChar => CPPType::UnsignedChar,
        crate::Type::CUShort => CPPType::UnsignedShort,
        crate::Type::CUInt => CPPType::UnsignedInt,
        crate::Type::CULong => CPPType::UnsignedLong,
        crate::Type::CULongLong => CPPType::UnsignedLongLong,
        crate::Type::CFloat => CPPType::Float,
        crate::Type::CDouble => CPPType::Double,
        crate::Type::CStr => CPPType::ConstPointer(Box::new(CPPType::Char)),
        crate::Type::CVoidPtr => CPPType::Pointer(Box::new(CPPType::Void)),
        crate::Type::ISize => CPPType::LongLong,
        crate::Type::USize => CPPType::UnsignedLongLong,
        crate::Type::Pointer { target, .. } => {
            CPPType::Pointer(Box::new(map_chim_type_to_cpp(target)))
        }
        crate::Type::Function { params, ret, .. } => {
            let ret_type = Box::new(map_chim_type_to_cpp(ret));
            let param_types: Vec<CPPType> = params.iter()
                .map(|t| map_chim_type_to_cpp(t))
                .collect();
            CPPType::FunctionPointer(ret_type, param_types)
        }
        crate::Type::Struct { name, .. } => CPPType::Class(name.to_string(), vec![]),
        crate::Type::Tuple(types) => {
            if types.is_empty() {
                CPPType::Void
            } else {
                CPPType::StdTuple(types.iter().map(|t| map_chim_type_to_cpp(t)).collect())
            }
        }
        _ => CPPType::Void,
    }
}

#[derive(Debug, Clone)]
pub struct CPPStdTuple(Vec<CPPType>);

impl CPPStdTuple {
    pub fn new(types: Vec<CPPType>) -> Self {
        Self(types)
    }
}
