use crate::backend::CodegenBackend;
use crate::ir::{Module, Function, Instruction, Type};
use std::error::Error;

pub struct UnisonBackend;

impl UnisonBackend {
    pub fn new() -> Self {
        Self
    }
    
    fn generate_type(&self, ty: &Type) -> String {
        match ty {
            Type::Void => "Unit".to_string(),
            Type::Int32 => "Int".to_string(),
            Type::Int64 => "Int64".to_string(),
            Type::Float32 => "Float32".to_string(),
            Type::Float64 => "Float64".to_string(),
            Type::Bool => "Boolean".to_string(),
            Type::String => "Text".to_string(),
            Type::Ptr(inner) => format!("Ref {}", self.generate_type(inner)),
            _ => "Any".to_string(),
        }
    }
    
    fn generate_function(&self, func: &Function) -> String {
        let mut output = String::new();
        
        let ret_type = self.generate_type(&func.return_type);
        
        // Unison函数签名 - 使用能力语法
        output.push_str(&format!("{} : ", func.name));
        
        // 参数类型
        let param_types: Vec<String> = func.params.iter()
            .map(|(name, ty)| format!("{} -> {}", self.generate_type(ty)))
            .collect();
        output.push_str(&param_types.join(" "));
        output.push_str(&format!("{{ {0} }}\n", ret_type));
        
        // 函数定义
        output.push_str(&format!("{} = ", func.name));
        
        // 参数名
        let param_names: Vec<String> = func.params.iter()
            .map(|(name, _)| name.clone())
            .collect();
        output.push_str(&param_names.join(" "));
        
        output.push_str(" ->\n");
        
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
                format!("  let {} : {} = todo \"alloca\"\n", dest, self.generate_type(ty))
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
                format!("  let {} = {} / {}\n", dest, left, right)
            },
            Instruction::Mod { dest, left, right } => {
                format!("  let {} = {} % {}\n", dest, left, right)
            },
            Instruction::Eq { dest, left, right } => {
                format!("  let {} = {} == {}\n", dest, left, right)
            },
            Instruction::Ne { dest, left, right } => {
                format!("  let {} = {} != {}\n", dest, left, right)
            },
            Instruction::Lt { dest, left, right } => {
                format!("  let {} = {} < {}\n", dest, left, right)
            },
            Instruction::Le { dest, left, right } => {
                format!("  let {} = {} <= {}\n", dest, left, right)
            },
            Instruction::Gt { dest, left, right } => {
                format!("  let {} = {} > {}\n", dest, left, right)
            },
            Instruction::Ge { dest, left, right } => {
                format!("  let {} = {} >= {}\n", dest, left, right)
            },
            Instruction::And { dest, left, right } => {
                format!("  let {} = {} && {}\n", dest, left, right)
            },
            Instruction::Or { dest, left, right } => {
                format!("  let {} = {} || {}\n", dest, left, right)
            },
            Instruction::Not { dest, src } => {
                format!("  let {} = not {}\n", dest, src)
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
                "  ()\n".to_string()
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

impl CodegenBackend for UnisonBackend {
    fn name(&self) -> &str {
        "Unison"
    }
    
    fn generate(&self, module: &Module) -> Result<String, Box<dyn Error>> {
        let mut output = String::new();
        
        // Unison文件头部
        output.push_str("-- Generated by Chim Compiler\n");
        output.push_str("-- Target: Unison\n\n");
        
        // 生成函数
        for func in &module.functions {
            output.push_str(&self.generate_function(func));
        }
        
        Ok(output)
    }
    
    fn file_extension(&self) -> &str {
        "u"
    }
}
