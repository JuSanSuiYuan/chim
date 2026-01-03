use crate::backend::CodegenBackend;
use crate::ir::{Module, Function, Instruction, Type};
use std::error::Error;

pub struct DelphiBackend;

impl DelphiBackend {
    pub fn new() -> Self {
        Self
    }
    
    fn generate_type(&self, ty: &Type) -> String {
        match ty {
            Type::Void => "".to_string(),
            Type::Int32 => "Integer".to_string(),
            Type::Int64 => "Int64".to_string(),
            Type::Float32 => "Single".to_string(),
            Type::Float64 => "Double".to_string(),
            Type::Bool => "Boolean".to_string(),
            Type::String => "String".to_string(),
            Type::Ptr(inner) => format!("^{}", self.generate_type(inner)),
            _ => "Pointer".to_string(),
        }
    }
    
    fn generate_function(&self, func: &Function) -> String {
        let mut output = String::new();
        
        // 函数签名
        let ret_type = self.generate_type(&func.return_type);
        if ret_type.is_empty() {
            output.push_str(&format!("procedure {}(", func.name));
        } else {
            output.push_str(&format!("function {}(", func.name));
        }
        
        // 参数
        let params: Vec<String> = func.params.iter()
            .map(|(name, ty)| format!("{}: {}", name, self.generate_type(ty)))
            .collect();
        output.push_str(&params.join("; "));
        
        if !ret_type.is_empty() {
            output.push_str(&format!("): {};\n", ret_type));
        } else {
            output.push_str(");\n");
        }
        
        output.push_str("begin\n");
        
        // 函数体
        for inst in &func.body {
            output.push_str(&self.generate_instruction(inst));
        }
        
        output.push_str("end;\n\n");
        output
    }
    
    fn generate_instruction(&self, inst: &Instruction) -> String {
        match inst {
            Instruction::Alloca { dest, ty } => {
                format!("  var {}: {};\n", dest, self.generate_type(ty))
            },
            Instruction::Add { dest, left, right } => {
                format!("  {} := {} + {};\n", dest, left, right)
            },
            Instruction::Sub { dest, left, right } => {
                format!("  {} := {} - {};\n", dest, left, right)
            },
            Instruction::Mul { dest, left, right } => {
                format!("  {} := {} * {};\n", dest, left, right)
            },
            Instruction::Div { dest, left, right } => {
                format!("  {} := {} / {};\n", dest, left, right)
            },
            Instruction::Mod { dest, left, right } => {
                format!("  {} := {} mod {};\n", dest, left, right)
            },
            Instruction::Eq { dest, left, right } => {
                format!("  {} := {} = {};\n", dest, left, right)
            },
            Instruction::Ne { dest, left, right } => {
                format!("  {} := {} <> {};\n", dest, left, right)
            },
            Instruction::Lt { dest, left, right } => {
                format!("  {} := {} < {};\n", dest, left, right)
            },
            Instruction::Le { dest, left, right } => {
                format!("  {} := {} <= {};\n", dest, left, right)
            },
            Instruction::Gt { dest, left, right } => {
                format!("  {} := {} > {};\n", dest, left, right)
            },
            Instruction::Ge { dest, left, right } => {
                format!("  {} := {} >= {};\n", dest, left, right)
            },
            Instruction::And { dest, left, right } => {
                format!("  {} := {} and {};\n", dest, left, right)
            },
            Instruction::Or { dest, left, right } => {
                format!("  {} := {} or {};\n", dest, left, right)
            },
            Instruction::Not { dest, src } => {
                format!("  {} := not {};\n", dest, src)
            },
            Instruction::Store { dest, src } => {
                format!("  {} := {};\n", dest, src)
            },
            Instruction::Load { dest, src } => {
                format!("  {} := {};\n", dest, src)
            },
            Instruction::Return(Some(value)) => {
                format!("  Result := {};\n", value)
            },
            Instruction::Return(None) => {
                "  Exit;\n".to_string()
            },
            Instruction::ReturnInPlace(value) => {
                format!("  Result := {}; {{ RVO优化 }}\n", value)
            },
            Instruction::Call { dest, func, args } => {
                let args_str = args.join(", ");
                if let Some(d) = dest {
                    format!("  {} := {}({});\n", d, func, args_str)
                } else {
                    format!("  {}({});\n", func, args_str)
                }
            },
            _ => format!("  {{ Unsupported: {:?} }}\n", inst),
        }
    }
}

impl CodegenBackend for DelphiBackend {
    fn name(&self) -> &str {
        "Delphi"
    }
    
    fn generate(&self, module: &Module) -> Result<String, Box<dyn Error>> {
        let mut output = String::new();
        
        output.push_str("unit GeneratedCode;\n\n");
        output.push_str("interface\n\n");
        output.push_str("implementation\n\n");
        
        // 生成函数
        for func in &module.functions {
            output.push_str(&self.generate_function(func));
        }
        
        output.push_str("end.\n");
        
        Ok(output)
    }
    
    fn file_extension(&self) -> &str {
        "pas"
    }
}
