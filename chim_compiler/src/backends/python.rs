use crate::backend::CodegenBackend;
use crate::ir::{Module, Function, Instruction, Type};
use std::error::Error;

pub struct PythonBackend;

impl PythonBackend {
    pub fn new() -> Self {
        Self
    }
    
    fn generate_type(&self, ty: &Type) -> String {
        match ty {
            Type::Void => "None".to_string(),
            Type::Int32 | Type::Int64 => "int".to_string(),
            Type::Float32 | Type::Float64 => "float".to_string(),
            Type::Bool => "bool".to_string(),
            Type::String => "str".to_string(),
            Type::Ptr(_) => "Any".to_string(),
            _ => "Any".to_string(),
        }
    }
    
    fn generate_function(&self, func: &Function) -> String {
        let mut output = String::new();
        
        // 函数签名
        output.push_str(&format!("def {}(", func.name));
        
        // 参数
        let params: Vec<String> = func.params.iter()
            .map(|(name, ty)| format!("{}: {}", name, self.generate_type(ty)))
            .collect();
        output.push_str(&params.join(", "));
        
        let ret_type = self.generate_type(&func.return_type);
        output.push_str(&format!(") -> {}:\n", ret_type));
        
        // 函数体
        let mut has_body = false;
        for inst in &func.body {
            let code = self.generate_instruction(inst);
            if !code.trim().is_empty() {
                output.push_str(&code);
                has_body = true;
            }
        }
        
        if !has_body {
            output.push_str("    pass\n");
        }
        
        output.push_str("\n");
        output
    }
    
    fn generate_instruction(&self, inst: &Instruction) -> String {
        match inst {
            Instruction::Alloca { dest, .. } => {
                format!("    {}: Any\n", dest)
            },
            Instruction::Add { dest, left, right } => {
                format!("    {} = {} + {}\n", dest, left, right)
            },
            Instruction::Sub { dest, left, right } => {
                format!("    {} = {} - {}\n", dest, left, right)
            },
            Instruction::Mul { dest, left, right } => {
                format!("    {} = {} * {}\n", dest, left, right)
            },
            Instruction::Div { dest, left, right } => {
                format!("    {} = {} / {}\n", dest, left, right)
            },
            Instruction::Mod { dest, left, right } => {
                format!("    {} = {} % {}\n", dest, left, right)
            },
            Instruction::Eq { dest, left, right } => {
                format!("    {} = {} == {}\n", dest, left, right)
            },
            Instruction::Ne { dest, left, right } => {
                format!("    {} = {} != {}\n", dest, left, right)
            },
            Instruction::Lt { dest, left, right } => {
                format!("    {} = {} < {}\n", dest, left, right)
            },
            Instruction::Le { dest, left, right } => {
                format!("    {} = {} <= {}\n", dest, left, right)
            },
            Instruction::Gt { dest, left, right } => {
                format!("    {} = {} > {}\n", dest, left, right)
            },
            Instruction::Ge { dest, left, right } => {
                format!("    {} = {} >= {}\n", dest, left, right)
            },
            Instruction::And { dest, left, right } => {
                format!("    {} = {} and {}\n", dest, left, right)
            },
            Instruction::Or { dest, left, right } => {
                format!("    {} = {} or {}\n", dest, left, right)
            },
            Instruction::Not { dest, src } => {
                format!("    {} = not {}\n", dest, src)
            },
            Instruction::Store { dest, src } => {
                format!("    {} = {}\n", dest, src)
            },
            Instruction::Load { dest, src } => {
                format!("    {} = {}\n", dest, src)
            },
            Instruction::Return(Some(value)) => {
                format!("    return {}\n", value)
            },
            Instruction::Return(None) => {
                "    return None\n".to_string()
            },
            Instruction::ReturnInPlace(value) => {
                format!("    return {}  # RVO优化\n", value)
            },
            Instruction::Call { dest, func, args } => {
                let args_str = args.join(", ");
                if let Some(d) = dest {
                    format!("    {} = {}({})\n", d, func, args_str)
                } else {
                    format!("    {}({})\n", func, args_str)
                }
            },
            _ => format!("    # Unsupported: {:?}\n", inst),
        }
    }
}

impl CodegenBackend for PythonBackend {
    fn name(&self) -> &str {
        "Python"
    }
    
    fn generate(&self, module: &Module) -> Result<String, Box<dyn Error>> {
        let mut output = String::new();
        
        output.push_str("from typing import Any\n\n");
        
        // 生成函数
        for func in &module.functions {
            output.push_str(&self.generate_function(func));
        }
        
        Ok(output)
    }
    
    fn file_extension(&self) -> &str {
        "py"
    }
}
