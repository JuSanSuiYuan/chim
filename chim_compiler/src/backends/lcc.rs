use crate::backend::CodegenBackend;
use crate::ir::{Module, Function, Type};
use std::error::Error;

/// LCC 后端 - 可重定向的C编译器
pub struct LCCBackend;

impl LCCBackend {
    pub fn new() -> Self { Self }
}

impl CodegenBackend for LCCBackend {
    fn name(&self) -> &str { "LCC" }
    
    fn generate(&self, module: &Module) -> Result<String, Box<dyn Error>> {
        let mut code = String::from("/* Chim -> LCC (Retargetable) */\n#include <stdio.h>\n\n");
        for func in &module.functions {
            let ret = match func.return_type { Type::Void => "void", Type::Int32 => "int", _ => "int" };
            code.push_str(&format!("{} {}() {{ return 0; }}\n", ret, func.name));
        }
        Ok(code)
    }
    
    fn file_extension(&self) -> &str { "c" }
}
