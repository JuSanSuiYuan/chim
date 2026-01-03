use crate::backend::CodegenBackend;
use crate::ir::{Module, Function, Instruction, Type};
use std::error::Error;

pub struct RBackend;

impl RBackend {
    pub fn new() -> Self {
        Self
    }
    
    fn generate_type(&self, ty: &Type) -> String {
        match ty {
            Type::Void => "NULL".to_string(),
            Type::Int32 | Type::Int64 => "numeric".to_string(),
            Type::Float32 | Type::Float64 => "numeric".to_string(),
            Type::Bool => "logical".to_string(),
            Type::String => "character".to_string(),
            Type::Ptr(_) => "list".to_string(),
            _ => "ANY".to_string(),
        }
    }
    
    fn generate_function(&self, func: &Function) -> String {
        let mut output = String::new();
        
        // 函数签名
        output.push_str(&format!("{} <- function(", func.name));
        
        // 参数
        let params: Vec<String> = func.params.iter()
            .map(|(name, _)| name.clone())
            .collect();
        output.push_str(&params.join(", "));
        
        output.push_str(") {\n");
        
        // 函数体
        for inst in &func.body {
            output.push_str(&self.generate_instruction(inst));
        }
        
        output.push_str("}\n\n");
        output
    }
    
    fn generate_instruction(&self, inst: &Instruction) -> String {
        match inst {
            Instruction::Alloca { dest, .. } => {
                format!("  {} <- NULL\n", dest)
            },
            Instruction::Add { dest, left, right } => {
                format!("  {} <- {} + {}\n", dest, left, right)
            },
            Instruction::Sub { dest, left, right } => {
                format!("  {} <- {} - {}\n", dest, left, right)
            },
            Instruction::Mul { dest, left, right } => {
                format!("  {} <- {} * {}\n", dest, left, right)
            },
            Instruction::Div { dest, left, right } => {
                format!("  {} <- {} / {}\n", dest, left, right)
            },
            Instruction::Mod { dest, left, right } => {
                format!("  {} <- {} %% {}\n", dest, left, right)
            },
            Instruction::Eq { dest, left, right } => {
                format!("  {} <- {} == {}\n", dest, left, right)
            },
            Instruction::Ne { dest, left, right } => {
                format!("  {} <- {} != {}\n", dest, left, right)
            },
            Instruction::Lt { dest, left, right } => {
                format!("  {} <- {} < {}\n", dest, left, right)
            },
            Instruction::Le { dest, left, right } => {
                format!("  {} <- {} <= {}\n", dest, left, right)
            },
            Instruction::Gt { dest, left, right } => {
                format!("  {} <- {} > {}\n", dest, left, right)
            },
            Instruction::Ge { dest, left, right } => {
                format!("  {} <- {} >= {}\n", dest, left, right)
            },
            Instruction::And { dest, left, right } => {
                format!("  {} <- {} && {}\n", dest, left, right)
            },
            Instruction::Or { dest, left, right } => {
                format!("  {} <- {} || {}\n", dest, left, right)
            },
            Instruction::Not { dest, src } => {
                format!("  {} <- !{}\n", dest, src)
            },
            Instruction::Store { dest, src } => {
                format!("  {} <- {}\n", dest, src)
            },
            Instruction::Load { dest, src } => {
                format!("  {} <- {}\n", dest, src)
            },
            Instruction::Return(Some(value)) => {
                format!("  return({})\n", value)
            },
            Instruction::Return(None) => {
                "  return(NULL)\n".to_string()
            },
            Instruction::ReturnInPlace(value) => {
                format!("  return({})  # RVO优化\n", value)
            },
            Instruction::Call { dest, func, args } => {
                let args_str = args.join(", ");
                if let Some(d) = dest {
                    format!("  {} <- {}({})\n", d, func, args_str)
                } else {
                    format!("  {}({})\n", func, args_str)
                }
            },
            _ => format!("  # Unsupported: {:?}\n", inst),
        }
    }
}

impl CodegenBackend for RBackend {
    fn name(&self) -> &str {
        "R"
    }
    
    fn generate(&self, module: &Module) -> Result<String, Box<dyn Error>> {
        let mut output = String::new();
        
        // 生成函数
        for func in &module.functions {
            output.push_str(&self.generate_function(func));
        }
        
        Ok(output)
    }
    
    fn file_extension(&self) -> &str {
        "r"
    }
}
