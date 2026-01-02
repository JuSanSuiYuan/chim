use crate::backend::CodegenBackend;
use crate::ir::{Module, Function, Instruction, Type};
use std::error::Error;

pub struct WASMBackend;

impl WASMBackend {
    pub fn new() -> Self {
        Self
    }
    
    fn generate_type(&self, ty: &Type) -> String {
        match ty {
            Type::Void => "".to_string(),
            Type::Int32 => "i32".to_string(),
            Type::Int64 => "i64".to_string(),
            Type::Float32 => "f32".to_string(),
            Type::Float64 => "f64".to_string(),
            Type::Bool => "i32".to_string(),
            _ => "i32".to_string(),
        }
    }
    
    fn generate_function(&self, func: &Function) -> String {
        let mut output = String::new();
        
        // 函数签名
        output.push_str(&format!("  (func ${}", func.name));
        
        // 参数
        if !func.params.is_empty() {
            for (name, ty) in &func.params {
                let wasm_type = self.generate_type(ty);
                if !wasm_type.is_empty() {
                    output.push_str(&format!(" (param ${} {})", name, wasm_type));
                }
            }
        }
        
        // 返回类型
        let ret_type = self.generate_type(&func.return_type);
        if !ret_type.is_empty() {
            output.push_str(&format!(" (result {})", ret_type));
        }
        
        output.push('\n');
        
        // 函数体
        for inst in &func.body {
            output.push_str(&self.generate_instruction(inst));
        }
        
        output.push_str("  )\n");
        output
    }
    
    fn generate_instruction(&self, inst: &Instruction) -> String {
        match inst {
            Instruction::Add { dest, left, right } => {
                format!("    local.get ${}\n    local.get ${}\n    i32.add\n    local.set ${}\n", 
                    left, right, dest)
            },
            Instruction::Sub { dest, left, right } => {
                format!("    local.get ${}\n    local.get ${}\n    i32.sub\n    local.set ${}\n",
                    left, right, dest)
            },
            Instruction::Mul { dest, left, right } => {
                format!("    local.get ${}\n    local.get ${}\n    i32.mul\n    local.set ${}\n",
                    left, right, dest)
            },
            Instruction::Return(Some(value)) => {
                format!("    local.get ${}\n    return\n", value)
            },
            Instruction::Return(None) => {
                "    return\n".to_string()
            },
            Instruction::ReturnInPlace(value) => {
                // RVO: 直接返回，不需要额外拷贝
                format!("    ;; RVO优化\n    local.get ${}\n    return\n", value)
            },
            Instruction::Call { dest, func, args } => {
                let mut s = String::new();
                for arg in args {
                    s.push_str(&format!("    local.get ${}\n", arg));
                }
                s.push_str(&format!("    call ${}\n", func));
                if let Some(d) = dest {
                    s.push_str(&format!("    local.set ${}\n", d));
                }
                s
            },
            _ => format!("    ;; {:?}\n", inst),
        }
    }
}

impl CodegenBackend for WASMBackend {
    fn name(&self) -> &str {
        "WebAssembly"
    }
    
    fn generate(&self, module: &Module) -> Result<String, Box<dyn Error>> {
        let mut output = String::new();
        
        output.push_str("(module\n");
        
        // 生成函数
        for func in &module.functions {
            output.push_str(&self.generate_function(func));
        }
        
        // 导出main函数
        if module.functions.iter().any(|f| f.name == "main") {
            output.push_str("  (export \"main\" (func $main))\n");
        }
        
        output.push_str(")\n");
        
        Ok(output)
    }
    
    fn file_extension(&self) -> &str {
        "wat"
    }
}
