use crate::backend::CodegenBackend;
use crate::ir::{Module, Function, Instruction, Type};
use std::error::Error;

pub struct GoBackend;

impl GoBackend {
    pub fn new() -> Self {
        Self
    }
    
    fn generate_type(&self, ty: &Type) -> String {
        match ty {
            Type::Void => "".to_string(),
            Type::Int32 => "int32".to_string(),
            Type::Int64 => "int64".to_string(),
            Type::Float32 => "float32".to_string(),
            Type::Float64 => "float64".to_string(),
            Type::Bool => "bool".to_string(),
            Type::String => "string".to_string(),
            Type::Ptr(inner) => format!("*{}", self.generate_type(inner)),
            _ => "interface{}".to_string(),
        }
    }
    
    fn generate_function(&self, func: &Function) -> String {
        let mut output = String::new();
        
        // 函数签名
        output.push_str(&format!("func {}(", func.name));
        
        // 参数
        let params: Vec<String> = func.params.iter()
            .map(|(name, ty)| format!("{} {}", name, self.generate_type(ty)))
            .collect();
        output.push_str(&params.join(", "));
        
        let ret_type = self.generate_type(&func.return_type);
        if !ret_type.is_empty() {
            output.push_str(&format!(") {} {{\n", ret_type));
        } else {
            output.push_str(") {\n");
        }
        
        // 函数体
        for inst in &func.body {
            output.push_str(&self.generate_instruction(inst));
        }
        
        output.push_str("}\n\n");
        output
    }
    
    fn generate_instruction(&self, inst: &Instruction) -> String {
        match inst {
            Instruction::Alloca { dest, ty } => {
                format!("\tvar {} {}\n", dest, self.generate_type(ty))
            },
            Instruction::Add { dest, left, right } => {
                format!("\t{} := {} + {}\n", dest, left, right)
            },
            Instruction::Sub { dest, left, right } => {
                format!("\t{} := {} - {}\n", dest, left, right)
            },
            Instruction::Mul { dest, left, right } => {
                format!("\t{} := {} * {}\n", dest, left, right)
            },
            Instruction::Div { dest, left, right } => {
                format!("\t{} := {} / {}\n", dest, left, right)
            },
            Instruction::Mod { dest, left, right } => {
                format!("\t{} := {} % {}\n", dest, left, right)
            },
            Instruction::Eq { dest, left, right } => {
                format!("\t{} := {} == {}\n", dest, left, right)
            },
            Instruction::Ne { dest, left, right } => {
                format!("\t{} := {} != {}\n", dest, left, right)
            },
            Instruction::Lt { dest, left, right } => {
                format!("\t{} := {} < {}\n", dest, left, right)
            },
            Instruction::Le { dest, left, right } => {
                format!("\t{} := {} <= {}\n", dest, left, right)
            },
            Instruction::Gt { dest, left, right } => {
                format!("\t{} := {} > {}\n", dest, left, right)
            },
            Instruction::Ge { dest, left, right } => {
                format!("\t{} := {} >= {}\n", dest, left, right)
            },
            Instruction::And { dest, left, right } => {
                format!("\t{} := {} && {}\n", dest, left, right)
            },
            Instruction::Or { dest, left, right } => {
                format!("\t{} := {} || {}\n", dest, left, right)
            },
            Instruction::Not { dest, src } => {
                format!("\t{} := !{}\n", dest, src)
            },
            Instruction::Store { dest, src } => {
                format!("\t{} = {}\n", dest, src)
            },
            Instruction::Load { dest, src } => {
                format!("\t{} := {}\n", dest, src)
            },
            Instruction::Return(Some(value)) => {
                format!("\treturn {}\n", value)
            },
            Instruction::Return(None) => {
                "\treturn\n".to_string()
            },
            Instruction::ReturnInPlace(value) => {
                format!("\treturn {} // RVO优化\n", value)
            },
            Instruction::Call { dest, func, args } => {
                let args_str = args.join(", ");
                if let Some(d) = dest {
                    format!("\t{} := {}({})\n", d, func, args_str)
                } else {
                    format!("\t{}({})\n", func, args_str)
                }
            },
            _ => format!("\t// Unsupported: {:?}\n", inst),
        }
    }
}

impl CodegenBackend for GoBackend {
    fn name(&self) -> &str {
        "Go"
    }
    
    fn generate(&self, module: &Module) -> Result<String, Box<dyn Error>> {
        let mut output = String::new();
        
        output.push_str("package main\n\n");
        output.push_str("import \"fmt\"\n\n");
        
        // 生成函数
        for func in &module.functions {
            output.push_str(&self.generate_function(func));
        }
        
        Ok(output)
    }
    
    fn file_extension(&self) -> &str {
        "go"
    }
}
