use crate::backend::CodegenBackend;
use crate::ir::{Module, Function, Type};
use std::error::Error;

/// CompCert 后端 - 经过验证的C编译器
pub struct CompCertBackend;

impl CompCertBackend {
    pub fn new() -> Self { Self }
}

impl CodegenBackend for CompCertBackend {
    fn name(&self) -> &str { "CompCert" }
    
    fn generate(&self, module: &Module) -> Result<String, Box<dyn Error>> {
        let mut code = String::from("/* Chim -> CompCert (Verified) */\n#include <stdio.h>\n\n");
        for func in &module.functions {
            let ret = match func.return_type { Type::Void => "void", Type::Int32 => "int", _ => "int" };
            code.push_str(&format!("{} {}() {{ return 0; }}\n", ret, func.name));
        }
        Ok(code)
    }
    
    fn file_extension(&self) -> &str { "c" }
}
