use crate::backend::CodegenBackend;
use crate::ir::{Module, Function, Instruction, Type};
use std::error::Error;

pub struct NativeBackend;

impl NativeBackend {
    pub fn new() -> Self {
        Self
    }
    
    fn generate_type(&self, ty: &Type) -> String {
        match ty {
            Type::Void => "void".to_string(),
            Type::Int32 => "int32_t".to_string(),
            Type::Int64 => "int64_t".to_string(),
            Type::Float32 => "float".to_string(),
            Type::Float64 => "double".to_string(),
            Type::Bool => "bool".to_string(),
            Type::String => "const char*".to_string(),
            Type::Ptr(inner) => format!("{}*", self.generate_type(inner)),
            _ => "void*".to_string(),
        }
    }
    
    fn generate_function(&self, func: &Function) -> String {
        let mut output = String::new();
        
        // 函数签名
        let ret_type = self.generate_type(&func.return_type);
        output.push_str(&format!("{} {}(", ret_type, func.name));
        
        // 参数
        if func.params.is_empty() {
            output.push_str("void");
        } else {
            let params: Vec<String> = func.params.iter()
                .map(|(name, ty)| format!("{} {}", self.generate_type(ty), name))
                .collect();
            output.push_str(&params.join(", "));
        }
        
        output.push_str(") {\n");
        
        // 局部变量声明
        output.push_str("    // 局部变量\n");
        
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
                format!("    int32_t {} = {} + {};\n", dest, left, right)
            },
            Instruction::Sub { dest, left, right } => {
                format!("    int32_t {} = {} - {};\n", dest, left, right)
            },
            Instruction::Mul { dest, left, right } => {
                format!("    int32_t {} = {} * {};\n", dest, left, right)
            },
            Instruction::Div { dest, left, right } => {
                format!("    int32_t {} = {} / {};\n", dest, left, right)
            },
            Instruction::Store { dest, src } => {
                format!("    {} = {};\n", dest, src)
            },
            Instruction::Load { dest, src } => {
                format!("    auto {} = {};\n", dest, src)
            },
            Instruction::Return(Some(value)) => {
                format!("    return {};\n", value)
            },
            Instruction::Return(None) => {
                "    return;\n".to_string()
            },
            Instruction::ReturnInPlace(value) => {
                // RVO: C++编译器会自动优化
                format!("    return {}; // RVO优化\n", value)
            },
            Instruction::Call { dest, func, args } => {
                let args_str = args.join(", ");
                if let Some(d) = dest {
                    format!("    auto {} = {}({});\n", d, func, args_str)
                } else {
                    format!("    {}({});\n", func, args_str)
                }
            },
            _ => format!("    // {:?}\n", inst),
        }
    }
}

impl CodegenBackend for NativeBackend {
    fn name(&self) -> &str {
        "Native C"
    }
    
    fn generate(&self, module: &Module) -> Result<String, Box<dyn Error>> {
        let mut output = String::new();
        
        // 头文件
        output.push_str("#include <stdint.h>\n");
        output.push_str("#include <stdbool.h>\n");
        output.push_str("#include <stdio.h>\n\n");
        
        // 前向声明
        for func in &module.functions {
            let ret_type = self.generate_type(&func.return_type);
            output.push_str(&format!("{} {}(", ret_type, func.name));
            if func.params.is_empty() {
                output.push_str("void");
            } else {
                let params: Vec<String> = func.params.iter()
                    .map(|(name, ty)| format!("{} {}", self.generate_type(ty), name))
                    .collect();
                output.push_str(&params.join(", "));
            }
            output.push_str(");\n");
        }
        output.push('\n');
        
        // 生成函数
        for func in &module.functions {
            output.push_str(&self.generate_function(func));
        }
        
        Ok(output)
    }
    
    fn file_extension(&self) -> &str {
        "c"
    }
}
