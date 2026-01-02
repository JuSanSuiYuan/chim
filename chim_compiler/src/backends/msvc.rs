use crate::backend::CodegenBackend;
use crate::ir::{Module, Function, Type};
use std::error::Error;

/// MSVC (Microsoft Visual C++) 后端
pub struct MSVCBackend;

impl MSVCBackend {
    pub fn new() -> Self { Self }
}

impl CodegenBackend for MSVCBackend {
    fn name(&self) -> &str { "MSVC" }
    
    fn generate(&self, module: &Module) -> Result<String, Box<dyn Error>> {
        let mut code = String::from("/* Chim -> MSVC */\n#include <stdio.h>\n#pragma warning(disable: 4996)\n\n");
        for func in &module.functions {
            let ret = match func.return_type { Type::Void => "void", Type::Int32 => "int", _ => "int" };
            code.push_str(&format!("__declspec(dllexport) {} {}() {{ return 0; }}\n", ret, func.name));
        }
        Ok(code)
    }
    
    fn file_extension(&self) -> &str { "c" }
}
