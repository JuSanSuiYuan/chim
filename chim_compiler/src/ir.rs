use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Void,
    Int32,
    Int64,
    Float32,
    Float64,
    Bool,
    String,
    Ptr(Box<Type>),
    Ref(Box<Type>),
    MutRef(Box<Type>),
    Array(Box<Type>, usize),
    Struct(String),
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Void => write!(f, "void"),
            Type::Int32 => write!(f, "i32"),
            Type::Int64 => write!(f, "i64"),
            Type::Float32 => write!(f, "f32"),
            Type::Float64 => write!(f, "f64"),
            Type::Bool => write!(f, "bool"),
            Type::String => write!(f, "string"),
            Type::Ptr(inner) => write!(f, "ptr<{}>", inner),
            Type::Ref(inner) => write!(f, "ref<{}>", inner),
            Type::MutRef(inner) => write!(f, "mut_ref<{}>", inner),
            Type::Array(inner, size) => write!(f, "[{}; {}]", inner, size),
            Type::Struct(name) => write!(f, "struct.{}", name),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    pub name: String,
    pub params: Vec<(String, Type)>,
    pub return_type: Type,
    pub body: Vec<Instruction>,
    pub is_public: bool,
    pub is_kernel: bool,
}

impl Function {
    pub fn new(name: String, return_type: Type) -> Self {
        Self {
            name,
            params: Vec::new(),
            return_type,
            body: Vec::new(),
            is_public: false,
            is_kernel: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Struct {
    pub name: String,
    pub fields: Vec<(String, Type)>,
    pub size: usize,
}

impl Struct {
    pub fn new(name: String) -> Self {
        Self {
            name,
            fields: Vec::new(),
            size: 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Global {
    pub name: String,
    pub ty: Type,
    pub value: Option<Value>,
    pub is_const: bool,
}

impl Global {
    pub fn new(name: String, ty: Type, is_const: bool) -> Self {
        Self {
            name,
            ty,
            value: None,
            is_const,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    ConstInt(i64),
    ConstFloat(f64),
    ConstBool(bool),
    ConstString(String),
    ConstNull,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Instruction {
    // 内存操作
    Alloca {
        dest: String,
        ty: Type,
    },
    Load {
        dest: String,
        src: String,
    },
    Store {
        dest: String,
        src: String,
    },
    GetPointer {
        dest: String,
        src: String,
        offset: i32,
    },
    
    // 算术运算
    Add {
        dest: String,
        left: String,
        right: String,
    },
    Sub {
        dest: String,
        left: String,
        right: String,
    },
    Mul {
        dest: String,
        left: String,
        right: String,
    },
    Div {
        dest: String,
        left: String,
        right: String,
    },
    Mod {
        dest: String,
        left: String,
        right: String,
    },
    
    // 比较运算
    Eq {
        dest: String,
        left: String,
        right: String,
    },
    Ne {
        dest: String,
        left: String,
        right: String,
    },
    Lt {
        dest: String,
        left: String,
        right: String,
    },
    Le {
        dest: String,
        left: String,
        right: String,
    },
    Gt {
        dest: String,
        left: String,
        right: String,
    },
    Ge {
        dest: String,
        left: String,
        right: String,
    },
    
    // 逻辑运算
    And {
        dest: String,
        left: String,
        right: String,
    },
    Or {
        dest: String,
        left: String,
        right: String,
    },
    Not {
        dest: String,
        src: String,
    },
    
    // 函数调用
    Call {
        dest: Option<String>,
        func: String,
        args: Vec<String>,
    },
    
    // 控制流
    Br(String),
    CondBr {
        cond: String,
        true_bb: String,
        false_bb: String,
    },
    Label(String),
    Return(Option<String>),
    ReturnInPlace(String),  // RVO优化：原地构造返回值
    
    // 引用操作
    Borrow {
        dest: String,
        src: String,
        mutable: bool,
    },
    Deref {
        dest: String,
        src: String,
    },
    
    // 循环
    Phi {
        dest: String,
        incoming: Vec<(String, String)>,
    },
    
    // 数组/结构体操作
    ExtractValue {
        dest: String,
        src: String,
        index: usize,
    },
    InsertValue {
        dest: String,
        src: String,
        value: String,
        index: usize,
    },
    GetElementPtr {
        dest: String,
        src: String,
        indices: Vec<i32>,
    },
    
    // 特殊操作
    Nop,
    Unreachable,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Module {
    pub functions: Vec<Function>,
    pub structs: Vec<Struct>,
    pub globals: Vec<Global>,
}

impl Module {
    pub fn new() -> Self {
        Self {
            functions: Vec::new(),
            structs: Vec::new(),
            globals: Vec::new(),
        }
    }
    
    pub fn add_function(&mut self, func: Function) {
        self.functions.push(func);
    }
    
    pub fn add_struct(&mut self, struct_: Struct) {
        self.structs.push(struct_);
    }
    
    pub fn add_global(&mut self, global: Global) {
        self.globals.push(global);
    }
    
    pub fn get_function(&self, name: &str) -> Option<&Function> {
        self.functions.iter().find(|f| f.name == name)
    }
    
    pub fn get_struct(&self, name: &str) -> Option<&Struct> {
        self.structs.iter().find(|s| s.name == name)
    }
}

impl Default for Module {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct BasicBlock {
    pub name: String,
    pub instructions: Vec<Instruction>,
}

impl BasicBlock {
    pub fn new(name: String) -> Self {
        Self {
            name,
            instructions: Vec::new(),
        }
    }
}
