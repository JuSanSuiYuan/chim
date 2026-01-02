use crate::backend::CodegenBackend;
use crate::ir::{Module, Function, Type};
use std::error::Error;

/// PGI (NVIDIA HPC) 后端
pub struct PGIBackend;

impl PGIBackend {
    pub fn new() -> Self { Self }
}

impl CodegenBackend for PGIBackend {
    fn name(&self) -> &str { "PGI" }
    
    fn generate(&self, module: &Module) -> Result<String, Box<dyn Error>> {
        let mut code = String::from("/* Chim -> PGI/NVIDIA HPC */\n#include <stdio.h>\n!$acc routine\n\n");
        for func in &module.functions {
            let ret = match func.return_type { Type::Void => "void", Type::Int32 => "int", _ => "int" };
            code.push_str(&format!("{} {}() {{ return 0; }}\n", ret, func.name));
        }
        Ok(code)
    }
    
    fn file_extension(&self) -> &str { "c" }
}
