use crate::backend::CodegenBackend;
use crate::ir::{Module, Function, Instruction, Type};
use std::error::Error;

/// Cranelift后端 - 生成Cranelift IR
/// Cranelift是一个快速的代码生成器，被Wasmtime使用
pub struct CraneliftBackend;

impl CraneliftBackend {
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
            Type::Bool => "i8".to_string(),
            Type::String => "i64".to_string(),
            _ => "i64".to_string(),
        }
    }
    
    fn generate_function(&self, func: &Function) -> String {
        let mut output = String::new();
        
        // 函数签名
        output.push_str(&format!("function u0:{}(", func.name));
        
        // 参数
        let params: Vec<String> = func.params.iter()
            .enumerate()
            .map(|(i, (_, ty))| format!("v{}: {}", i, self.generate_type(ty)))
            .collect();
        output.push_str(&params.join(", "));
        
        // 返回类型
        let ret_type = self.generate_type(&func.return_type);
        if !ret_type.is_empty() {
            output.push_str(&format!(") -> {} {{\n", ret_type));
        } else {
            output.push_str(") {\n");
        }
        
        // 入口块
        output.push_str("block0:\n");
        
        // 函数体
        for inst in &func.body {
            output.push_str(&self.generate_instruction(inst));
        }
        
        output.push_str("}\n\n");
        output
    }
    
    fn generate_instruction(&self, inst: &Instruction) -> String {
        match inst {
            Instruction::Add { dest, left, right } => {
                format!("    v{} = iadd v{}, v{}\n", dest, left, right)
            },
            Instruction::Sub { dest, left, right } => {
                format!("    v{} = isub v{}, v{}\n", dest, left, right)
            },
            Instruction::Mul { dest, left, right } => {
                format!("    v{} = imul v{}, v{}\n", dest, left, right)
            },
            Instruction::Div { dest, left, right } => {
                format!("    v{} = sdiv v{}, v{}\n", dest, left, right)
            },
            Instruction::Return(Some(value)) => {
                format!("    return v{}\n", value)
            },
            Instruction::Return(None) => {
                "    return\n".to_string()
            },
            Instruction::ReturnInPlace(value) => {
                format!("    return v{} ; RVO\n", value)
            },
            Instruction::Call { dest, func, args } => {
                let args_str: Vec<String> = args.iter()
                    .map(|a| format!("v{}", a))
                    .collect();
                if let Some(d) = dest {
                    format!("    v{} = call u0:{}({})\n", d, func, args_str.join(", "))
                } else {
                    format!("    call u0:{}({})\n", func, args_str.join(", "))
                }
            },
            _ => format!("    ; {:?}\n", inst),
        }
    }
}

impl CodegenBackend for CraneliftBackend {
    fn name(&self) -> &str {
        "Cranelift"
    }
    
    fn generate(&self, module: &Module) -> Result<String, Box<dyn Error>> {
        let mut output = String::new();
        
        output.push_str("; Chim generated Cranelift IR\n\n");
        
        // 生成函数
        for func in &module.functions {
            output.push_str(&self.generate_function(func));
        }
        
        Ok(output)
    }
    
    fn file_extension(&self) -> &str {
        "clif"
    }
}
