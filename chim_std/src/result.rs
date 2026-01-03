//! Result 和 Option 类型定义

/// Result 类型：表示可能失败的操作
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Result<T, E> {
    Ok(T),
    Err(E),
}

impl<T, E> Result<T, E> {
    pub fn is_ok(&self) -> bool {
        matches!(self, Result::Ok(_))
    }
    
    pub fn is_err(&self) -> bool {
        matches!(self, Result::Err(_))
    }
    
    pub fn unwrap(self) -> T {
        match self {
            Result::Ok(v) => v,
            Result::Err(_) => panic!("called `Result::unwrap()` on an `Err` value"),
        }
    }
    
    pub fn unwrap_or(self, default: T) -> T {
        match self {
            Result::Ok(v) => v,
            Result::Err(_) => default,
        }
    }
    
    pub fn map<U, F: FnOnce(T) -> U>(self, f: F) -> Result<U, E> {
        match self {
            Result::Ok(v) => Result::Ok(f(v)),
            Result::Err(e) => Result::Err(e),
        }
    }
}

/// Option 类型：表示可能为空的值
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Option<T> {
    Some(T),
    None,
}

impl<T> Option<T> {
    pub fn is_some(&self) -> bool {
        matches!(self, Option::Some(_))
    }
    
    pub fn is_none(&self) -> bool {
        matches!(self, Option::None)
    }
    
    pub fn unwrap(self) -> T {
        match self {
            Option::Some(v) => v,
            Option::None => panic!("called `Option::unwrap()` on a `None` value"),
        }
    }
    
    pub fn unwrap_or(self, default: T) -> T {
        match self {
            Option::Some(v) => v,
            Option::None => default,
        }
    }
    
    pub fn map<U, F: FnOnce(T) -> U>(self, f: F) -> Option<U> {
        match self {
            Option::Some(v) => Option::Some(f(v)),
            Option::None => Option::None,
        }
    }
}
