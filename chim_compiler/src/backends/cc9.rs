use crate::backend::CodegenBackend;
use crate::ir::{Module, Function, Type};
use std::error::Error;

/// 9cc 后端 - 小型C编译器
pub struct Cc9Backend;

impl Cc9Backend {
    pub fn new() -> Self { Self }
}

impl CodegenBackend for Cc9Backend {
    fn name(&self) -> &str { "9cc" }
    
    fn generate(&self, module: &Module) -> Result<String, Box<dyn Error>> {
        let mut code = String::from("/* Chim -> 9cc */\n#include <stdio.h>\n\n");
        for func in &module.functions {
            let ret = match func.return_type { Type::Void => "void", Type::Int32 => "int", _ => "int" };
            code.push_str(&format!("{} {}() {{ return 0; }}\n", ret, func.name));
        }
        Ok(code)
    }
    
    fn file_extension(&self) -> &str { "c" }
}
