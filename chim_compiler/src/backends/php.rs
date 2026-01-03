use crate::backend::CodegenBackend;
use crate::ir::{Module, Function, Instruction, Type};
use std::error::Error;

pub struct PhpBackend;

impl PhpBackend {
    pub fn new() -> Self {
        Self
    }
    
    fn generate_type(&self, ty: &Type) -> String {
        match ty {
            Type::Void => "void".to_string(),
            Type::Int32 | Type::Int64 => "int".to_string(),
            Type::Float32 | Type::Float64 => "float".to_string(),
            Type::Bool => "bool".to_string(),
            Type::String => "string".to_string(),
            Type::Ptr(_) => "mixed".to_string(),
            _ => "mixed".to_string(),
        }
    }
    
    fn generate_function(&self, func: &Function) -> String {
        let mut output = String::new();
        
        // 函数签名
        output.push_str("function ");
        output.push_str(&func.name);
        output.push('(');
        
        // 参数（PHP 8+ 类型声明）
        let params: Vec<String> = func.params.iter()
            .map(|(name, ty)| format!("{} ${}", self.generate_type(ty), name))
            .collect();
        output.push_str(&params.join(", "));
        
        output.push_str("): ");
        output.push_str(&self.generate_type(&func.return_type));
        output.push_str("\n{\n");
        
        // 函数体
        for inst in &func.body {
            output.push_str(&self.generate_instruction(inst));
        }
        
        output.push_str("}\n");
        output
    }
    
    fn generate_instruction(&self, inst: &Instruction) -> String {
        match inst {
            Instruction::Alloca { dest, .. } => {
                format!("    ${} = null;\n", dest)
            },
            Instruction::Add { dest, left, right } => {
                format!("    ${} = {} + {};\n", dest, self.format_var(left), self.format_var(right))
            },
            Instruction::Sub { dest, left, right } => {
                format!("    ${} = {} - {};\n", dest, self.format_var(left), self.format_var(right))
            },
            Instruction::Mul { dest, left, right } => {
                format!("    ${} = {} * {};\n", dest, self.format_var(left), self.format_var(right))
            },
            Instruction::Div { dest, left, right } => {
                format!("    ${} = {} / {};\n", dest, self.format_var(left), self.format_var(right))
            },
            Instruction::Mod { dest, left, right } => {
                format!("    ${} = {} % {};\n", dest, self.format_var(left), self.format_var(right))
            },
            Instruction::Eq { dest, left, right } => {
                format!("    ${} = {} === {};\n", dest, self.format_var(left), self.format_var(right))
            },
            Instruction::Ne { dest, left, right } => {
                format!("    ${} = {} !== {};\n", dest, self.format_var(left), self.format_var(right))
            },
            Instruction::Lt { dest, left, right } => {
                format!("    ${} = {} < {};\n", dest, self.format_var(left), self.format_var(right))
            },
            Instruction::Le { dest, left, right } => {
                format!("    ${} = {} <= {};\n", dest, self.format_var(left), self.format_var(right))
            },
            Instruction::Gt { dest, left, right } => {
                format!("    ${} = {} > {};\n", dest, self.format_var(left), self.format_var(right))
            },
            Instruction::Ge { dest, left, right } => {
                format!("    ${} = {} >= {};\n", dest, self.format_var(left), self.format_var(right))
            },
            Instruction::And { dest, left, right } => {
                format!("    ${} = {} && {};\n", dest, self.format_var(left), self.format_var(right))
            },
            Instruction::Or { dest, left, right } => {
                format!("    ${} = {} || {};\n", dest, self.format_var(left), self.format_var(right))
            },
            Instruction::Not { dest, src } => {
                format!("    ${} = !{};\n", dest, self.format_var(src))
            },
            Instruction::Store { dest, src } => {
                format!("    ${} = {};\n", dest, self.format_var(src))
            },
            Instruction::Load { dest, src } => {
                format!("    ${} = {};\n", dest, self.format_var(src))
            },
            Instruction::Return(Some(value)) => {
                format!("    return {};\n", self.format_var(value))
            },
            Instruction::Return(None) => {
                "    return;\n".to_string()
            },
            Instruction::ReturnInPlace(value) => {
                format!("    return {}; // RVO优化\n", self.format_var(value))
            },
            Instruction::Call { dest, func, args } => {
                let args_str = args.iter()
                    .map(|arg| self.format_var(arg))
                    .collect::<Vec<_>>()
                    .join(", ");
                if let Some(d) = dest {
                    format!("    ${} = {}({});\n", d, func, args_str)
                } else {
                    format!("    {}({});\n", func, args_str)
                }
            },
            _ => format!("    // Unsupported: {:?}\n", inst),
        }
    }
    
    fn format_var(&self, var: &str) -> String {
        if var.chars().next().map_or(false, |c| c.is_alphabetic()) {
            format!("${}", var)
        } else {
            var.to_string()
        }
    }
}

impl CodegenBackend for PhpBackend {
    fn name(&self) -> &str {
        "PHP"
    }
    
    fn generate(&self, module: &Module) -> Result<String, Box<dyn Error>> {
        let mut output = String::new();
        
        // PHP开始标签
        output.push_str("<?php\n");
        output.push_str("// Generated by Chim Compiler\n");
        output.push_str("// PHP Backend (PHP 8+)\n");
        output.push_str("declare(strict_types=1);\n\n");
        
        // 生成所有函数
        for func in &module.functions {
            output.push_str(&self.generate_function(func));
            output.push('\n');
        }
        
        Ok(output)
    }
    
    fn file_extension(&self) -> &str {
        "php"
    }
}
