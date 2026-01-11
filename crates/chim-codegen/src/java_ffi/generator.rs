use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum JavaType {
    Void,
    Boolean,
    Byte,
    Char,
    Short,
    Int,
    Long,
    Float,
    Double,
    String,
    Object,
    Class,
    ClassLoader,
    ObjectArray(Box<JavaType>),
    PrimitiveArray(PrimitiveArrayType),
    List(Box<JavaType>),
    Map(Box<JavaType>, Box<JavaType>),
    Set(Box<JavaType>),
    Optional(Box<JavaType>),
    OptionalInt,
    OptionalLong,
    OptionalDouble,
    Stream(Box<JavaType>),
    CompletableFuture(Box<JavaType>),
    Path,
    File,
    InputStream,
    OutputStream,
    Reader,
    Writer,
    BufferedReader,
    PrintStream,
    PrintWriter,
    Thread,
    Runnable,
    Callable(Box<JavaType>),
    Exception,
    RuntimeException,
    IllegalArgumentException,
    NullPointerException,
    IndexOutOfBoundsException,
    UnsupportedOperationException,
    NoSuchElementException,
    Collection(Box<JavaType>),
    Iterator(Box<JavaType>),
    Iterable(Box<JavaType>),
    Comparator(Box<JavaType>),
    Comparable(Box<JavaType>),
    Function(Box<JavaType>, Box<JavaType>),
    Supplier(Box<JavaType>),
    Consumer(Box<JavaType>),
    Predicate(Box<JavaType>),
    UnaryOperator(Box<JavaType>),
    BinaryOperator(Box<JavaType>),
    Custom(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PrimitiveArrayType {
    BooleanArray,
    ByteArray,
    CharArray,
    ShortArray,
    IntArray,
    LongArray,
    FloatArray,
    DoubleArray,
}

#[derive(Debug, Clone)]
pub struct JavaMethod {
    pub name: String,
    pub return_type: JavaType,
    pub params: Vec<JavaParameter>,
    pub is_static: bool,
    pub is_final: bool,
    pub is_synchronized: bool,
    pub is_native: bool,
    pub access: JavaAccess,
    pub throws: Vec<JavaType>,
    pub default_value: Option<String>,
    pub annotations: Vec<JavaAnnotation>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum JavaAccess {
    Public,
    Protected,
    Private,
    PackagePrivate,
}

#[derive(Debug, Clone)]
pub struct JavaParameter {
    pub name: String,
    pub param_type: JavaType,
    pub is_final: bool,
}

#[derive(Debug, Clone)]
pub struct JavaField {
    pub name: String,
    pub field_type: JavaType,
    pub is_static: bool,
    pub is_final: bool,
    pub is_volatile: bool,
    pub is_transient: bool,
    pub access: JavaAccess,
    pub initializer: Option<String>,
    pub annotations: Vec<JavaAnnotation>,
}

#[derive(Debug, Clone)]
pub struct JavaAnnotation {
    pub name: String,
    pub values: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct JavaClass {
    pub name: String,
    pub package: String,
    pub is_public: bool,
    pub is_final: bool,
    pub is_abstract: bool,
    pub extends: Option<String>,
    pub implements: Vec<String>,
    pub fields: Vec<JavaField>,
    pub methods: Vec<JavaMethod>,
    pub constructors: Vec<JavaConstructor>,
    pub inner_classes: Vec<JavaClass>,
    pub annotations: Vec<JavaAnnotation>,
    pub is_interface: bool,
    pub is_enum: bool,
    pub is_record: bool,
    pub enum_constants: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct JavaConstructor {
    pub params: Vec<JavaParameter>,
    pub access: JavaAccess,
    pub body: Option<String>,
    pub annotations: Vec<JavaAnnotation>,
}

#[derive(Debug, Clone)]
pub struct JavaPackage {
    pub name: String,
    pub classes: Vec<JavaClass>,
    pub imports: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct JavaInterface {
    pub name: String,
    pub package: String,
    pub extends: Vec<String>,
    pub methods: Vec<JavaMethod>,
    pub constants: Vec<(String, String)>,
    pub nested_types: Vec<JavaClass>,
    pub annotations: Vec<JavaAnnotation>,
    pub is_functional: bool,
}

#[derive(Debug, Clone)]
pub struct JavaEnum {
    pub name: String,
    pub package: String,
    pub implements: Vec<String>,
    pub constants: Vec<JavaEnumConstant>,
    pub fields: Vec<JavaField>,
    pub methods: Vec<JavaMethod>,
    pub constructors: Vec<JavaConstructor>,
    pub annotations: Vec<JavaAnnotation>,
}

#[derive(Debug, Clone)]
pub struct JavaEnumConstant {
    pub name: String,
    pub args: Vec<String>,
    pub class_body: Option<JavaClass>,
}

#[derive(Debug, Clone)]
pub struct JavaRecord {
    pub name: String,
    pub package: String,
    pub components: Vec<JavaRecordComponent>,
    pub canonical_constructor: JavaConstructor,
    pub methods: Vec<JavaMethod>,
    pub annotations: Vec<JavaAnnotation>,
}

#[derive(Debug, Clone)]
pub struct JavaRecordComponent {
    pub name: String,
    pub component_type: JavaType,
    pub annotations: Vec<JavaAnnotation>,
}

#[derive(Debug, Clone)]
pub struct JavaAnnotationDef {
    pub name: String,
    pub package: String,
    pub elements: Vec<JavaAnnotationElement>,
    pub meta_annotations: Vec<JavaAnnotation>,
}

#[derive(Debug, Clone)]
pub struct JavaAnnotationElement {
    pub name: String,
    pub element_type: JavaType,
    pub default_value: Option<String>,
    pub required: bool,
}

#[derive(Debug, Clone)]
pub struct JNIBinding {
    pub java_class: String,
    pub java_method: String,
    pub signature: String,
    pub mangled_name: String,
    pub is_static: bool,
    pub is_constructor: bool,
    pub return_type: JavaType,
    pub param_types: Vec<JavaType>,
    pub native_code: Option<String>,
}

#[derive(Debug, Clone)]
pub struct JavaGeneratorOptions {
    pub java_version: JavaVersion,
    pub generate_jni: bool,
    pub generate_bridges: bool,
    pub enable_null_checks: bool,
    pub enable_exception_translation: bool,
    pub use_smart_pointers: bool,
    pub generate_to_string: bool,
    pub generate_equals: bool,
    pub generate_hash_code: bool,
    pub generate_builder: bool,
    pub use_annotations: bool,
    pub enable_modules: bool,
    pub use_module_info: bool,
    pub generate_records: bool,
    pub sealed_classes: bool,
    pub pattern_matching: bool,
    pub text_blocks: bool,
    pub switch_expressions: bool,
    pub sealed_interfaces: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum JavaVersion {
    Java8,
    Java9,
    Java10,
    Java11,
    Java12,
    Java13,
    Java14,
    Java15,
    Java16,
    Java17,
    Java18,
    Java19,
    Java20,
    Java21,
    Java22,
    Java23,
}

impl Default for JavaGeneratorOptions {
    fn default() -> Self {
        Self {
            java_version: JavaVersion::Java17,
            generate_jni: true,
            generate_bridges: true,
            enable_null_checks: true,
            enable_exception_translation: true,
            use_smart_pointers: true,
            generate_to_string: true,
            generate_equals: true,
            generate_hash_code: true,
            generate_builder: false,
            use_annotations: true,
            enable_modules: true,
            use_module_info: true,
            generate_records: true,
            sealed_classes: true,
            pattern_matching: true,
            text_blocks: true,
            switch_expressions: true,
            sealed_interfaces: true,
        }
    }
}

#[derive(Debug, Default)]
pub struct JavaFFIGenerator {
    types: HashMap<String, JavaType>,
    packages: HashMap<String, JavaPackage>,
    jni_bindings: Vec<JNIBinding>,
    options: JavaGeneratorOptions,
    class_path: Vec<String>,
}

impl JavaFFIGenerator {
    pub fn new() -> Self {
        let mut generator = Self {
            types: HashMap::new(),
            packages: HashMap::new(),
            jni_bindings: Vec::new(),
            options: JavaGeneratorOptions::default(),
            class_path: Vec::new(),
        };
        generator.init_builtin_types();
        generator
    }

    fn init_builtin_types(&mut self) {
        self.types.insert("void".to_string(), JavaType::Void);
        self.types.insert("boolean".to_string(), JavaType::Boolean);
        self.types.insert("byte".to_string(), JavaType::Byte);
        self.types.insert("char".to_string(), JavaType::Char);
        self.types.insert("short".to_string(), JavaType::Short);
        self.types.insert("int".to_string(), JavaType::Int);
        self.types.insert("long".to_string(), JavaType::Long);
        self.types.insert("float".to_string(), JavaType::Float);
        self.types.insert("double".to_string(), JavaType::Double);
        self.types.insert("String".to_string(), JavaType::String);
        self.types.insert("Object".to_string(), JavaType::Object);
        self.types.insert("Class".to_string(), JavaType::Class);
        self.types.insert("ClassLoader".to_string(), JavaType::ClassLoader);
        self.types.insert("List".to_string(), JavaType::List(Box::new(JavaType::Object)));
        self.types.insert("Map".to_string(), JavaType::Map(Box::new(JavaType::Object), Box::new(JavaType::Object)));
        self.types.insert("Set".to_string(), JavaType::Set(Box::new(JavaType::Object)));
        self.types.insert("Optional".to_string(), JavaType::Optional(Box::new(JavaType::Object)));
        self.types.insert("Path".to_string(), JavaType::Path);
        self.types.insert("File".to_string(), JavaType::File);
        self.types.insert("Exception".to_string(), JavaType::Exception);
        self.types.insert("RuntimeException".to_string(), JavaType::RuntimeException);
        self.types.insert("Runnable".to_string(), JavaType::Runnable);
        self.types.insert("Thread".to_string(), JavaType::Thread);
    }

    pub fn generate_java_class(&self, class: &JavaClass) -> String {
        let mut output = String::new();
        
        self.generate_package_declaration(&class.package, &mut output);
        self.generate_imports(&class.package, &mut output);
        
        for annotation in &class.annotations {
            self.generate_annotation(annotation, &mut output);
        }
        
        if class.is_enum {
            self.generate_enum_declaration(class, &mut output);
        } else if class.is_record {
            self.generate_record_declaration(class, &mut output);
        } else if class.is_interface {
            self.generate_interface_declaration(class, &mut output);
        } else {
            self.generate_class_declaration(class, &mut output);
        }
        
        output
    }

    fn generate_package_declaration(&self, package: &str, output: &mut String) {
        output.push_str(&format!("package {};\n\n", package));
    }

    fn generate_imports(&self, _package: &str, output: &mut String) {
        output.push_str("import java.util.*;\n");
        output.push_str("import java.io.*;\n");
        output.push_str("import java.nio.file.*;\n");
        output.push_str("import java.util.concurrent.*;\n");
        output.push_str("import java.util.function.*;\n");
        output.push_str("import java.util.stream.*;\n");
        output.push('\n');
    }

    fn generate_class_declaration(&self, class: &JavaClass, output: &mut String) {
        let access_modifier = match class.access {
            JavaAccess::Public => "public ",
            JavaAccess::Protected => "protected ",
            JavaAccess::Private => "private ",
            JavaAccess::PackagePrivate => "",
        };
        
        let class_keyword = if class.is_interface {
            "interface"
        } else {
            "class"
        };
        
        let modifiers = format!("{}{}{}",
            access_modifier,
            if class.is_final { "final " } else { "" },
            if class.is_abstract { "abstract " } else { "" }
        );
        
        output.push_str(&format!("{} {} {}", modifiers, class_keyword, class.name));
        
        if let Some(extends) = &class.extends {
            output.push_str(&format!(" extends {}", extends));
        }
        
        if !class.implements.is_empty() {
            output.push_str(&format!(" implements {}", class.implements.join(", ")));
        }
        
        output.push_str(" {\n");
        
        for field in &class.fields {
            self.generate_field(field, output);
        }
        
        for constructor in &class.constructors {
            self.generate_constructor(constructor, &class.name, output);
        }
        
        for method in &class.methods {
            self.generate_method(method, output);
        }
        
        for inner in &class.inner_classes {
            output.push_str("\n");
            output.push_str(&self.generate_java_class(inner));
        }
        
        output.push_str("}\n");
    }

    fn generate_enum_declaration(&self, class: &JavaClass, output: &mut String) {
        output.push_str("public enum ");
        output.push_str(&class.name);
        
        if !class.implements.is_empty() {
            output.push_str(&format!(" implements {}", class.implements.join(", ")));
        }
        
        output.push_str(" {\n");
        
        for (i, constant) in class.enum_constants.iter().enumerate() {
            if i > 0 {
                output.push_str(",\n");
            }
            output.push_str(&format!("    {}", constant));
        }
        
        output.push_str(";\n\n");
        
        for field in &class.fields {
            self.generate_field(field, output);
        }
        
        for constructor in &class.constructors {
            self.generate_constructor(constructor, &class.name, output);
        }
        
        for method in &class.methods {
            self.generate_method(method, output);
        }
        
        output.push_str("}\n");
    }

    fn generate_record_declaration(&self, record: &JavaRecord, output: &mut String) {
        output.push_str("public record ");
        output.push_str(&record.name);
        output.push_str("(");
        
        let components: Vec<String> = record.components.iter()
            .map(|c| format!("{} {}", self.type_to_string(&c.component_type), c.name))
            .collect();
        output.push_str(&components.join(", "));
        output.push_str(")");
        
        if !record.implements.is_empty() {
            output.push_str(&format!(" implements {}", record.implements.join(", ")));
        }
        
        output.push_str(" {\n");
        
        for method in &record.methods {
            self.generate_method(method, output);
        }
        
        output.push_str("}\n");
    }

    fn generate_interface_declaration(&self, interface: &JavaInterface, output: &mut String) {
        output.push_str("public interface ");
        output.push_str(&interface.name);
        
        if !interface.extends.is_empty() {
            output.push_str(&format!(" extends {}", interface.extends.join(", ")));
        }
        
        output.push_str(" {\n");
        
        for (name, value) in &interface.constants {
            output.push_str(&format!("    {} {} = {};\n", 
                if interface.constants.is_empty() { "int" } else { "Object" },
                name, value));
        }
        
        for method in &interface.methods {
            output.push_str("\n    ");
            self.generate_method_signature(method, output);
            output.push_str(";\n");
        }
        
        for nested in &interface.nested_types {
            output.push_str("\n    ");
            output.push_str(&self.generate_java_class(nested));
        }
        
        output.push_str("}\n");
    }

    fn generate_field(&self, field: &JavaField, output: &mut String) {
        for annotation in &field.annotations {
            self.generate_annotation(annotation, output);
        }
        
        let access = match field.access {
            JavaAccess::Public => "    public ",
            JavaAccess::Protected => "    protected ",
            JavaAccess::Private => "    private ",
            JavaAccess::PackagePrivate => "    ",
        };
        
        let modifiers = format!("{}{}{}{}",
            access,
            if field.is_static { "static " } else { "" },
            if field.is_final { "final " } else { "" },
            if field.is_volatile { "volatile " } else { "" }
        );
        
        output.push_str(&format!("{}{} {}", modifiers, self.type_to_string(&field.field_type), field.name));
        
        if let Some(init) = &field.initializer {
            output.push_str(&format!(" = {}", init));
        }
        
        output.push_str(";\n");
    }

    fn generate_constructor(&self, ctor: &JavaConstructor, class_name: &str, output: &mut String) {
        let access = match ctor.access {
            JavaAccess::Public => "    public ",
            JavaAccess::Protected => "    protected ",
            JavaAccess::Private => "    private ",
            JavaAccess::PackagePrivate => "    ",
        };
        
        output.push_str(access);
        output.push_str(class_name);
        output.push_str("(");
        
        let params: Vec<String> = ctor.params.iter()
            .map(|p| format!("{} {}", self.type_to_string(&p.param_type), p.name))
            .collect();
        output.push_str(&params.join(", "));
        output.push_str(")");
        
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

    fn generate_method(&self, method: &JavaMethod, output: &mut String) {
        for annotation in &method.annotations {
            self.generate_annotation(annotation, output);
        }
        
        let access = match method.access {
            JavaAccess::Public => "    public ",
            JavaAccess::Protected => "    protected ",
            JavaAccess::Private => "    private ",
            JavaAccess::PackagePrivate => "    ",
        };
        
        let modifiers = format!("{}{}{}{}",
            access,
            if method.is_static { "static " } else { "" },
            if method.is_final { "final " } else { "" },
            if method.is_synchronized { "synchronized " } else { "" }
        );
        
        output.push_str(modifiers);
        output.push_str(&self.type_to_string(&method.return_type));
        output.push(' ');
        output.push_str(&method.name);
        output.push('(');
        
        let params: Vec<String> = method.params.iter()
            .map(|p| format!("{} {}", self.type_to_string(&p.param_type), p.name))
            .collect();
        output.push_str(&params.join(", "));
        output.push_str(")");
        
        if !method.throws.is_empty() {
            let exceptions: Vec<String> = method.throws.iter()
                .map(|t| self.type_to_string(t))
                .collect();
            output.push_str(&format!(" throws {}", exceptions.join(", ")));
        }
        
        if method.is_native {
            output.push_str(" native;\n");
        } else if let Some(body) = &method.default_value {
            output.push_str(&format!(" default {};\n", body));
        } else {
            output.push_str(" {\n");
            output.push_str("        // TODO: implementation\n");
            output.push_str("    }\n");
        }
    }

    fn generate_method_signature(&self, method: &JavaMethod, output: &mut String) {
        if method.is_static {
            output.push_str("static ");
        }
        if method.is_default {
            output.push_str("default ");
        }
        
        output.push_str(&self.type_to_string(&method.return_type));
        output.push(' ');
        output.push_str(&method.name);
        output.push('(');
        
        let params: Vec<String> = method.params.iter()
            .map(|p| format!("{} {}", self.type_to_string(&p.param_type), p.name))
            .collect();
        output.push_str(&params.join(", "));
        output.push(')');
    }

    fn generate_annotation(&self, annotation: &JavaAnnotation, output: &mut String) {
        output.push_str(&format!("@{}\n", annotation.name));
    }

    fn type_to_string(&self, ty: &JavaType) -> String {
        match ty {
            JavaType::Void => "void".to_string(),
            JavaType::Boolean => "boolean".to_string(),
            JavaType::Byte => "byte".to_string(),
            JavaType::Char => "char".to_string(),
            JavaType::Short => "short".to_string(),
            JavaType::Int => "int".to_string(),
            JavaType::Long => "long".to_string(),
            JavaType::Float => "float".to_string(),
            JavaType::Double => "double".to_string(),
            JavaType::String => "String".to_string(),
            JavaType::Object => "Object".to_string(),
            JavaType::Class => "Class<?>".to_string(),
            JavaType::ClassLoader => "ClassLoader".to_string(),
            JavaType::ObjectArray(inner) => format!("{}[]", self.type_to_string(inner)),
            JavaType::PrimitiveArray(arr_type) => match arr_type {
                PrimitiveArrayType::BooleanArray => "boolean[]".to_string(),
                PrimitiveArrayType::ByteArray => "byte[]".to_string(),
                PrimitiveArrayType::CharArray => "char[]".to_string(),
                PrimitiveArrayType::ShortArray => "short[]".to_string(),
                PrimitiveArrayType::IntArray => "int[]".to_string(),
                PrimitiveArrayType::LongArray => "long[]".to_string(),
                PrimitiveArrayType::FloatArray => "float[]".to_string(),
                PrimitiveArrayType::DoubleArray => "double[]".to_string(),
            },
            JavaType::List(inner) => format!("List<{}>", self.type_to_string(inner)),
            JavaType::Map(key, value) => {
                format!("Map<{}, {}>", self.type_to_string(key), self.type_to_string(value))
            }
            JavaType::Set(inner) => format!("Set<{}>", self.type_to_string(inner)),
            JavaType::Optional(inner) => format!("Optional<{}>", self.type_to_string(inner)),
            JavaType::OptionalInt => "OptionalInt".to_string(),
            JavaType::OptionalLong => "OptionalLong".to_string(),
            JavaType::OptionalDouble => "OptionalDouble".to_string(),
            JavaType::Stream(inner) => format!("Stream<{}>", self.type_to_string(inner)),
            JavaType::CompletableFuture(inner) => {
                format!("CompletableFuture<{}>", self.type_to_string(inner))
            }
            JavaType::Path => "Path".to_string(),
            JavaType::File => "File".to_string(),
            JavaType::InputStream => "InputStream".to_string(),
            JavaType::OutputStream => "OutputStream".to_string(),
            JavaType::Reader => "Reader".to_string(),
            JavaType::Writer => "Writer".to_string(),
            JavaType::BufferedReader => "BufferedReader".to_string(),
            JavaType::PrintStream => "PrintStream".to_string(),
            JavaType::PrintWriter => "PrintWriter".to_string(),
            JavaType::Thread => "Thread".to_string(),
            JavaType::Runnable => "Runnable".to_string(),
            JavaType::Callable(inner) => format!("Callable<{}>", self.type_to_string(inner)),
            JavaType::Exception => "Exception".to_string(),
            JavaType::RuntimeException => "RuntimeException".to_string(),
            JavaType::IllegalArgumentException => "IllegalArgumentException".to_string(),
            JavaType::NullPointerException => "NullPointerException".to_string(),
            JavaType::IndexOutOfBoundsException => "IndexOutOfBoundsException".to_string(),
            JavaType::UnsupportedOperationException => "UnsupportedOperationException".to_string(),
            JavaType::NoSuchElementException => "NoSuchElementException".to_string(),
            JavaType::Collection(inner) => format!("Collection<{}>", self.type_to_string(inner)),
            JavaType::Iterator(inner) => format!("Iterator<{}>", self.type_to_string(inner)),
            JavaType::Iterable(inner) => format!("Iterable<{}>", self.type_to_string(inner)),
            JavaType::Comparator(inner) => format!("Comparator<{}>", self.type_to_string(inner)),
            JavaType::Comparable(inner) => format!("Comparable<{}>", self.type_to_string(inner)),
            JavaType::Function(in, out) => {
                format!("Function<{}, {}>", self.type_to_string(in), self.type_to_string(out))
            }
            JavaType::Supplier(ret) => format!("Supplier<{}>", self.type_to_string(ret)),
            JavaType::Consumer(in) => format!("Consumer<{}>", self.type_to_string(in)),
            JavaType::Predicate(in) => format!("Predicate<{}>", self.type_to_string(in)),
            JavaType::UnaryOperator(in) => format!("UnaryOperator<{}>", self.type_to_string(in)),
            JavaType::BinaryOperator(in) => format!("BinaryOperator<{}>", self.type_to_string(in)),
            JavaType::Custom(name) => name.clone(),
        }
    }

    pub fn generate_jni_header(&self, class: &JavaClass, native_methods: &[JavaMethod]) -> String {
        let mut output = String::new();
        
        output.push_str("/* DO NOT EDIT THIS FILE - it is machine generated */\n");
        output.push_str(&format!("#include <jni.h>\n"));
        output.push_str("/* Header for class {} */\n\n", class.name.replace('.', '_'));
        
        output.push_str("#ifndef _Included_");
        output.push_str(&class.name.replace('.', '_'));
        output.push_str("\n#define _Included_");
        output.push_str(&class.name.replace('.', '_'));
        output.push_str("\n\n");
        
        output.push_str("#ifdef __cplusplus\nextern \"C\" {\n#endif\n\n");
        
        for method in native_methods {
            self.generate_jni_method(class, method, &mut output);
        }
        
        output.push_str("#ifdef __cplusplus\n}\n#endif\n\n");
        output.push_str("#endif\n");
        
        output
    }

    fn generate_jni_method(&self, class: &JavaClass, method: &JavaMethod, output: &mut String) {
        let class_path = class.name.replace('.', '_');
        let method_name = if method.is_constructor {
            "<init>"
        } else {
            &method.name
        };
        
        output.push_str("/*\n");
        output.push_str(" * Class:     ");
        output.push_str(&class.name.replace('.', '_'));
        output.push_str("\n * Method:    ");
        output.push_str(method_name);
        output.push_str("\n * Signature: ");
        output.push_str(&self.generate_jni_signature(method));
        output.push_str("\n */\n");
        
        output.push_str("JNIEXPORT ");
        output.push_str(&self.jni_type_to_string(&method.return_type));
        output.push_str(" JNICALL\nJava_");
        output.push_str(&class_path.replace('_', "_1"));
        
        for c in class.name.chars() {
            if c == '.' {
                output.push_str("_0002");
            } else {
                output.push(c);
            }
        }
        output.push('_');
        output.push_str(method_name.replace('_', "_1"));
        
        output.push_str("\n  (JNIEnv *, ");
        
        if method.is_static {
            output.push_str("jclass");
        } else {
            output.push_str("jobject");
        }
        
        output.push_str(");\n\n");
    }

    fn generate_jni_signature(&self, method: &JavaMethod) -> String {
        let mut sig = String::from("(");
        
        for param in &method.params {
            sig.push_str(&self.jni_type_to_signature(&param.param_type));
        }
        
        sig.push(')');
        sig.push_str(&self.jni_type_to_signature(&method.return_type));
        
        sig
    }

    fn jni_type_to_string(&self, ty: &JavaType) -> String {
        match ty {
            JavaType::Void => "void".to_string(),
            JavaType::Boolean => "jboolean".to_string(),
            JavaType::Byte => "jbyte".to_string(),
            JavaType::Char => "jchar".to_string(),
            JavaType::Short => "jshort".to_string(),
            JavaType::Int => "jint".to_string(),
            JavaType::Long => "jlong".to_string(),
            JavaType::Float => "jfloat".to_string(),
            JavaType::Double => "jdouble".to_string(),
            JavaType::String => "jstring".to_string(),
            JavaType::Object => "jobject".to_string(),
            JavaType::Class => "jclass".to_string(),
            JavaType::ObjectArray(inner) => format!("jobjectArray /* {}[] */", self.type_to_string(inner)),
            JavaType::PrimitiveArray(arr_type) => match arr_type {
                PrimitiveArrayType::BooleanArray => "jbooleanArray".to_string(),
                PrimitiveArrayType::ByteArray => "jbyteArray".to_string(),
                PrimitiveArrayType::CharArray => "jcharArray".to_string(),
                PrimitiveArrayType::ShortArray => "jshortArray".to_string(),
                PrimitiveArrayType::IntArray => "jintArray".to_string(),
                PrimitiveArrayType::LongArray => "jlongArray".to_string(),
                PrimitiveArrayType::FloatArray => "jfloatArray".to_string(),
                PrimitiveArrayType::DoubleArray => "jdoubleArray".to_string(),
            },
            JavaType::List(_) | JavaType::Map(_, _) => "jobject".to_string(),
            _ => "jobject".to_string(),
        }
    }

    fn jni_type_to_signature(&self, ty: &JavaType) -> String {
        match ty {
            JavaType::Void => "V".to_string(),
            JavaType::Boolean => "Z".to_string(),
            JavaType::Byte => "B".to_string(),
            JavaType::Char => "C".to_string(),
            JavaType::Short => "S".to_string(),
            JavaType::Int => "I".to_string(),
            JavaType::Long => "J".to_string(),
            JavaType::Float => "F".to_string(),
            JavaType::Double => "D".to_string(),
            JavaType::String => "Ljava/lang/String;".to_string(),
            JavaType::Object => "Ljava/lang/Object;".to_string(),
            JavaType::Class => "Ljava/lang/Class;".to_string(),
            JavaType::ObjectArray(inner) => {
                format!("[{}", self.jni_type_to_signature(inner))
            }
            JavaType::PrimitiveArray(arr_type) => match arr_type {
                PrimitiveArrayType::BooleanArray => "[Z".to_string(),
                PrimitiveArrayType::ByteArray => "[B".to_string(),
                PrimitiveArrayType::CharArray => "[C".to_string(),
                PrimitiveArrayType::ShortArray => "[S".to_string(),
                PrimitiveArrayType::IntArray => "[I".to_string(),
                PrimitiveArrayType::LongArray => "[J".to_string(),
                PrimitiveArrayType::FloatArray => "[F".to_string(),
                PrimitiveArrayType::DoubleArray => "[D".to_string(),
            },
            JavaType::List(inner) => {
                format!("Ljava/util/List<{}>;", self.jni_type_to_signature(inner))
            }
            JavaType::Map(key, value) => {
                format!("Ljava/util/Map<{}, {}>;", 
                    self.jni_type_to_signature(key), 
                    self.jni_type_to_signature(value))
            }
            JavaType::Optional(inner) => {
                format!("Ljava/util/Optional<{}>;", self.jni_type_to_signature(inner))
            }
            _ => "Ljava/lang/Object;".to_string(),
        }
    }

    pub fn generate_jni_bridge(&self, class: &JavaClass, method: &JavaMethod) -> String {
        let class_path = class.name.replace('.', '_');
        let func_name = format!("Java_{}_", class_path.replace('_', "_1"));
        
        let mut output = String::new();
        
        output.push_str("extern \"C\" JNIEXPORT ");
        output.push_str(&self.jni_type_to_string(&method.return_type));
        output.push_str(" JNICALL\n");
        output.push_str(&func_name);
        output.push_str(&method.name.replace('_', "_1"));
        output.push_str("\n(JNIEnv* env, ");
        
        if method.is_static {
            output.push_str("jclass clazz");
        } else {
            output.push_str("jobject obj");
        }
        
        output.push_str(") {\n");
        
        if self.options.enable_null_checks {
            output.push_str("    if (env->ExceptionCheck()) {\n");
            output.push_str("        return;\n");
            output.push_str("    }\n\n");
        }
        
        output.push_str("    // TODO: Implement native method bridge\n");
        
        match &method.return_type {
            JavaType::Void => {
                output.push_str("    return;\n");
            }
            JavaType::Int => {
                output.push_str("    return 0;\n");
            }
            JavaType::Long => {
                output.push_str("    return 0L;\n");
            }
            JavaType::Boolean => {
                output.push_str("    return JNI_FALSE;\n");
            }
            JavaType::Double | JavaType::Float => {
                output.push_str("    return 0.0;\n");
            }
            _ => {
                output.push_str("    return nullptr;\n");
            }
        }
        
        output.push_str("}\n");
        
        output
    }

    pub fn add_jni_binding(&mut self, binding: JNIBinding) {
        self.jni_bindings.push(binding);
    }

    pub fn get_jni_bindings(&self) -> &[JNIBinding] {
        &self.jni_bindings
    }
}

pub fn map_chim_type_to_java(chim_type: &crate::Type) -> JavaType {
    match chim_type {
        crate::Type::CVoid => JavaType::Void,
        crate::Type::CBool => JavaType::Boolean,
        crate::Type::CChar => JavaType::Char,
        crate::Type::CShort => JavaType::Short,
        crate::Type::CInt => JavaType::Int,
        crate::Type::CLong => JavaType::Long,
        crate::Type::CLongLong => JavaType::Long,
        crate::Type::CUChar => JavaType::Int,
        crate::Type::CUShort => JavaType::Int,
        crate::Type::CUInt => JavaType::Long,
        crate::Type::CULong => JavaType::Long,
        crate::Type::CULongLong => JavaType::Long,
        crate::Type::CFloat => JavaType::Float,
        crate::Type::CDouble => JavaType::Double,
        crate::Type::CStr => JavaType::String,
        crate::Type::CVoidPtr => JavaType::Long,
        crate::Type::ISize => JavaType::Long,
        crate::Type::USize => JavaType::Long,
        crate::Type::Pointer { target, .. } => {
            if matches!(**target, crate::Type::CVoid) {
                JavaType::Long
            } else {
                JavaType::Long
            }
        }
        crate::Type::Array { element, length: _ } => {
            JavaType::ObjectArray(Box::new(map_chim_type_to_java(element)))
        }
        crate::Type::Function { .. } => JavaType::Custom("Runnable".to_string()),
        crate::Type::Struct { name, .. } => JavaType::Custom(name.to_string()),
        _ => JavaType::Object,
    }
}

#[derive(Debug, Clone)]
pub struct JavaNativeInterface {
    pub env: *mut JNIInvokeInterface,
    pub vm: *mut JavaVM,
}

impl JavaNativeInterface {
    pub fn new() -> Self {
        Self {
            env: std::ptr::null_mut(),
            vm: std::ptr::null_mut(),
        }
    }
    
    pub fn throw_exception(&self, message: &str) {
        // unsafe {
        //     ((*self.env).ExceptionDescribe)(self.env);
        //     ((*self.env).ExceptionClear)(self.env);
        //     let jstring = ((*self.env).NewStringUTF)(self.env, message.as_ptr() as *const i8);
        //     let exception_class = ((*self.env).FindClass)(self.env, b"java/lang/RuntimeException\0".as_ptr() as *const i8);
        //     ((*self.env).ThrowNew)(self.env, exception_class, jstring);
        // }
    }
}
