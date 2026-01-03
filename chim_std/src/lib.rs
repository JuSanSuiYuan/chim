//! Chim 标准库
//! 
//! 提供基础类型、集合、IO、并发等核心功能

pub mod prelude;
pub mod collections;
pub mod io;
pub mod string;
pub mod math;
pub mod concurrent;
pub mod result;

/// 重导出常用类型
pub use result::{Result, Option};
