use crate::backend::CodegenBackend;
use crate::ir::{Module, Function, Type};
use std::error::Error;

/// UCC (Universal C Compiler) 后端
pub struct UCCBackend;

impl UCCBackend {
    pub fn new() -> Self { Self }
}

impl CodegenBackend for UCCBackend {
    fn name(&self) -> &str { "UCC" }
    
    fn generate(&self, module: &Module) -> Result<String, Box<dyn Error>> {
        let mut code = String::from("/* Chim -> UCC */\n#include <stdio.h>\n\n");
        for func in &module.functions {
            let ret = match func.return_type { Type::Void => "void", Type::Int32 => "int", _ => "int" };
            code.push_str(&format!("{} {}() {{ return 0; }}\n", ret, func.name));
        }
        Ok(code)
    }
    
    fn file_extension(&self) -> &str { "c" }
}
