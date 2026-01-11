use crate::backend::CodegenBackend;
use crate::ir::{Module, Function, Instruction, Type};
use std::error::Error;

pub struct AgdaBackend;

impl AgdaBackend {
    pub fn new() -> Self {
        Self
    }
    
    fn generate_type(&self, ty: &Type) -> String {
        match ty {
            Type::Void => "⊤".to_string(),
            Type::Int32 => "ℤ".to_string(),
            Type::Int64 => "ℤ".to_string(),
            Type::Float32 => "Float".to_string(),
            Type::Float64 => "Float".to_string(),
            Type::Bool => "Bool".to_string(),
            Type::String => "String".to_string(),
            Type::Ptr(inner) => format!("Ptr {}", self.generate_type(inner)),
            _ => "Set".to_string(),
        }
    }
    
    fn generate_function(&self, func: &Function) -> String {
        let mut output = String::new();
        
        let ret_type = self.generate_type(&func.return_type);
        
        // Agda函数签名
        output.push_str(&format!("{} :", func.name));
        
        // 参数类型
        let param_types: Vec<String> = func.params.iter()
            .map(|(name, ty)| format!("{} → {}", self.generate_type(ty)))
            .collect();
        output.push_str(&param_types.join(" "));
        output.push_str(&format!(" → {}\n", ret_type));
        
        // 函数定义
        output.push_str(&format!("{} ", func.name));
        
        // 参数名
        let param_names: Vec<String> = func.params.iter()
            .map(|(name, _)| name.clone())
            .collect();
        output.push_str(&param_names.join(" "));
        
        output.push_str(" =\n");
        
        // 函数体
        for inst in &func.body {
            output.push_str(&self.generate_instruction(inst));
        }
        
        output.push_str("\n");
        output
    }
    
    fn generate_instruction(&self, inst: &Instruction) -> String {
        match inst {
            Instruction::Alloca { dest, ty } => {
                format!("  let {} : {} in\n", dest, self.generate_type(ty))
            },
            Instruction::Add { dest, left, right } => {
                format!("  let {} = {} + {}\n", dest, left, right)
            },
            Instruction::Sub { dest, left, right } => {
                format!("  let {} = {} - {}\n", dest, left, right)
            },
            Instruction::Mul { dest, left, right } => {
                format!("  let {} = {} * {}\n", dest, left, right)
            },
            Instruction::Div { dest, left, right } => {
                format!("  let {} = {} div {}\n", dest, left, right)
            },
            Instruction::Mod { dest, left, right } => {
                format!("  let {} = {} mod {}\n", dest, left, right)
            },
            Instruction::Eq { dest, left, right } => {
                format!("  let {} = {} ≡ {}\n", dest, left, right)
            },
            Instruction::Ne { dest, left, right } => {
                format!("  let {} = not ({} ≡ {})\n", dest, left, right)
            },
            Instruction::Lt { dest, left, right } => {
                format!("  let {} = {} < {}\n", dest, left, right)
            },
            Instruction::Le { dest, left, right } => {
                format!("  let {} = {} ≤ {}\n", dest, left, right)
            },
            Instruction::Gt { dest, left, right } => {
                format!("  let {} = {} > {}\n", dest, left, right)
            },
            Instruction::Ge { dest, left, right } => {
                format!("  let {} = {} ≥ {}\n", dest, left, right)
            },
            Instruction::And { dest, left, right } => {
                format!("  let {} = {} ∧ {}\n", dest, left, right)
            },
            Instruction::Or { dest, left, right } => {
                format!("  let {} = {} ∨ {}\n", dest, left, right)
            },
            Instruction::Not { dest, src } => {
                format!("  let {} = ¬ {}\n", dest, src)
            },
            Instruction::Store { dest, src } => {
                format!("  {} := {}\n", dest, src)
            },
            Instruction::Load { dest, src } => {
                format!("  let {} = !{}\n", dest, src)
            },
            Instruction::Return(Some(value)) => {
                format!("  {}\n", value)
            },
            Instruction::Return(None) => {
                "  tt\n".to_string()
            },
            Instruction::ReturnInPlace(value) => {
                format!("  {} -- RVO优化\n", value)
            },
            Instruction::Call { dest, func, args } => {
                let args_str = args.join(" ");
                if let Some(d) = dest {
                    format!("  let {} = {} {}\n", d, func, args_str)
                } else {
                    format!("  {} {}\n", func, args_str)
                }
            },
            _ => format!("  -- Unsupported: {:?}\n", inst),
        }
    }
}

impl CodegenBackend for AgdaBackend {
    fn name(&self) -> &str {
        "Agda"
    }
    
    fn generate(&self, module: &Module) -> Result<String, Box<dyn Error>> {
        let mut output = String::new();
        
        // Agda模块头部
        output.push_str("module Main where\n\n");
        output.push_str("open import Agda.Builtin.Nat\n");
        output.push_str("open import Agda.Builtin.Int\n");
        output.push_str("open import Agda.Builtin.Float\n");
        output.push_str("open import Agda.Builtin.Bool\n");
        output.push_str("open import Agda.Builtin.String\n");
        output.push_str("open import Agda.Builtin.Unit\n\n");
        
        // 生成函数
        for func in &module.functions {
            output.push_str(&self.generate_function(func));
        }
        
        Ok(output)
    }
    
    fn file_extension(&self) -> &str {
        "agda"
    }
}
