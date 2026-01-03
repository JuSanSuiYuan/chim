use crate::backend::CodegenBackend;
use crate::ir::{Module, Function, Instruction, Type};
use std::error::Error;

pub struct JuneBackend;

impl JuneBackend {
    pub fn new() -> Self {
        Self
    }
    
    fn generate_type(&self, ty: &Type) -> String {
        match ty {
            Type::Void => "()".to_string(),
            Type::Int32 => "i32".to_string(),
            Type::Int64 => "i64".to_string(),
            Type::Float32 => "f32".to_string(),
            Type::Float64 => "f64".to_string(),
            Type::Bool => "bool".to_string(),
            Type::String => "String".to_string(),
            Type::Ptr(inner) => format!("&{}", self.generate_type(inner)),
            _ => "Any".to_string(),
        }
    }
    
    fn generate_function(&self, func: &Function) -> String {
        let mut output = String::new();
        
        // 函数签名
        output.push_str("fn ");
        output.push_str(&func.name);
        output.push('(');
        
        // 参数
        let params: Vec<String> = func.params.iter()
            .map(|(name, ty)| format!("{}: {}", name, self.generate_type(ty)))
            .collect();
        output.push_str(&params.join(", "));
        
        output.push(')');
        
        // 返回类型
        if func.return_type != Type::Void {
            output.push_str(" -> ");
            output.push_str(&self.generate_type(&func.return_type));
        }
        
        output.push_str(" {\n");
        
        // 函数体
        for inst in &func.body {
            output.push_str(&self.generate_instruction(inst));
        }
        
        output.push_str("}\n");
        output
    }
    
    fn generate_instruction(&self, inst: &Instruction) -> String {
        match inst {
            Instruction::Alloca { dest, ty } => {
                format!("    let mut {}: {};\n", dest, self.generate_type(ty))
            },
            Instruction::Add { dest, left, right } => {
                format!("    let {} = {} + {};\n", dest, left, right)
            },
            Instruction::Sub { dest, left, right } => {
                format!("    let {} = {} - {};\n", dest, left, right)
            },
            Instruction::Mul { dest, left, right } => {
                format!("    let {} = {} * {};\n", dest, left, right)
            },
            Instruction::Div { dest, left, right } => {
                format!("    let {} = {} / {};\n", dest, left, right)
            },
            Instruction::Mod { dest, left, right } => {
                format!("    let {} = {} % {};\n", dest, left, right)
            },
            Instruction::Eq { dest, left, right } => {
                format!("    let {} = {} == {};\n", dest, left, right)
            },
            Instruction::Ne { dest, left, right } => {
                format!("    let {} = {} != {};\n", dest, left, right)
            },
            Instruction::Lt { dest, left, right } => {
                format!("    let {} = {} < {};\n", dest, left, right)
            },
            Instruction::Le { dest, left, right } => {
                format!("    let {} = {} <= {};\n", dest, left, right)
            },
            Instruction::Gt { dest, left, right } => {
                format!("    let {} = {} > {};\n", dest, left, right)
            },
            Instruction::Ge { dest, left, right } => {
                format!("    let {} = {} >= {};\n", dest, left, right)
            },
            Instruction::And { dest, left, right } => {
                format!("    let {} = {} && {};\n", dest, left, right)
            },
            Instruction::Or { dest, left, right } => {
                format!("    let {} = {} || {};\n", dest, left, right)
            },
            Instruction::Not { dest, src } => {
                format!("    let {} = !{};\n", dest, src)
            },
            Instruction::Store { dest, src } => {
                format!("    {} = {};\n", dest, src)
            },
            Instruction::Load { dest, src } => {
                format!("    let {} = {};\n", dest, src)
            },
            Instruction::Return(Some(value)) => {
                format!("    return {};\n", value)
            },
            Instruction::Return(None) => {
                "    return;\n".to_string()
            },
            Instruction::ReturnInPlace(value) => {
                format!("    return {}; // RVO优化\n", value)
            },
            Instruction::Call { dest, func, args } => {
                let args_str = args.join(", ");
                if let Some(d) = dest {
                    format!("    let {} = {}({});\n", d, func, args_str)
                } else {
                    format!("    {}({});\n", func, args_str)
                }
            },
            _ => format!("    // Unsupported: {:?}\n", inst),
        }
    }
}

impl CodegenBackend for JuneBackend {
    fn name(&self) -> &str {
        "June"
    }
    
    fn generate(&self, module: &Module) -> Result<String, Box<dyn Error>> {
        let mut output = String::new();
        
        // 文件头注释
        output.push_str("// Generated by Chim Compiler\n");
        output.push_str("// June Backend\n\n");
        
        // 生成所有函数
        for func in &module.functions {
            output.push_str(&self.generate_function(func));
            output.push('\n');
        }
        
        Ok(output)
    }
    
    fn file_extension(&self) -> &str {
        "june"
    }
}
