// ==================== 预导入模块 ====================
// 自动导入的基础功能

// 基础类型转换
pub fn to_string<T>(value: T) -> string {
    match value {
        Some(x) => "Some",
        None => "None",
    }
}

// 选项类型
pub enum Option<T> {
    Some(T),
    None,
}

impl<T> Option<T> {
    pub fn is_some(&self) -> bool {
        match self {
            Option::Some(_) => true,
            Option::None => false,
        }
    }
    
    pub fn is_none(&self) -> bool {
        match self {
            Option::Some(_) => false,
            Option::None => true,
        }
    }
    
    pub fn unwrap(&self) -> T {
        match self {
            Option::Some(x) => x.clone(),
            Option::None => panic("unwrap on None"),
        }
    }
    
    pub fn unwrap_or(&self, default: T) -> T {
        match self {
            Option::Some(x) => x.clone(),
            Option::None => default,
        }
    }
}

// 结果类型
pub enum Result<T, E> {
    Ok(T),
    Err(E),
}

impl<T, E> Result<T, E> {
    pub fn is_ok(&self) -> bool {
        match self {
            Result::Ok(_) => true,
            Result::Err(_) => false,
        }
    }
    
    pub fn is_err(&self) -> bool {
        match self {
            Result::Ok(_) => false,
            Result::Err(_) => true,
        }
    }
    
    pub fn unwrap(&self) -> T {
        match self {
            Result::Ok(x) => x.clone(),
            Result::Err(_) => panic("unwrap on Err"),
        }
    }
    
    pub fn unwrap_err(&self) -> E {
        match self {
            Result::Ok(_) => panic("unwrap_err on Ok"),
            Result::Err(e) => e.clone(),
        }
    }
}

// 比较 trait
pub trait Comparable<T> {
    fn cmp(&self, other: &T) -> int;
}

impl Comparable<int> for int {
    fn cmp(&self, other: &int) -> int {
        if self < other { -1 }
        else if self > other { 1 }
        else { 0 }
    }
}

// 可显示 trait
pub trait Display {
    fn to_string(&self) -> string;
}

impl Display for int {
    fn to_string(&self) -> string {
        __int_to_string(self)
    }
}

impl Display for float {
    fn to_string(&self) -> string {
        __float_to_string(self)
    }
}

impl Display for bool {
    fn to_string(&self) -> string {
        if self { "true" } else { "false" }
    }
}
