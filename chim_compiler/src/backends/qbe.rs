use crate::backend::CodegenBackend;
use crate::ir::{Module, Function, Instruction, Type};
use std::error::Error;

/// QBE (Quick Backend) 后端
/// QBE是一个轻量级的编译器后端，比LLVM简单但足够快速
pub struct QBEBackend;

impl QBEBackend {
    pub fn new() -> Self {
        Self
    }
    
    fn generate_type(&self, ty: &Type) -> String {
        match ty {
            Type::Void => "".to_string(),
            Type::Int32 => "w".to_string(),  // word (32位)
            Type::Int64 => "l".to_string(),  // long (64位)
            Type::Float32 => "s".to_string(), // single
            Type::Float64 => "d".to_string(), // double
            Type::Bool => "w".to_string(),
            Type::String => "l".to_string(),  // 指针
            Type::Ptr(_) => "l".to_string(),
            _ => "l".to_string(),
        }
    }
    
    fn generate_function(&self, func: &Function) -> String {
        let mut output = String::new();
        
        // 函数签名
        let ret_type = self.generate_type(&func.return_type);
        if ret_type.is_empty() {
            output.push_str("export function $");
        } else {
            output.push_str(&format!("export function {} $", ret_type));
        }
        output.push_str(&func.name);
        
        // 参数
        output.push('(');
        let params: Vec<String> = func.params.iter()
            .map(|(name, ty)| format!("{} %{}", self.generate_type(ty), name))
            .collect();
        output.push_str(&params.join(", "));
        output.push_str(") {\n");
        
        // 入口块
        output.push_str("@start\n");
        
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
                format!("    %{} =w add %{}, %{}\n", dest, left, right)
            },
            Instruction::Sub { dest, left, right } => {
                format!("    %{} =w sub %{}, %{}\n", dest, left, right)
            },
            Instruction::Mul { dest, left, right } => {
                format!("    %{} =w mul %{}, %{}\n", dest, left, right)
            },
            Instruction::Div { dest, left, right } => {
                format!("    %{} =w div %{}, %{}\n", dest, left, right)
            },
            Instruction::Store { dest, src } => {
                format!("    storew %{}, %{}\n", src, dest)
            },
            Instruction::Load { dest, src } => {
                format!("    %{} =w loadw %{}\n", dest, src)
            },
            Instruction::Alloca { dest, .. } => {
                format!("    %{} =l alloc4 4\n", dest)
            },
            Instruction::Return(Some(value)) => {
                format!("    ret %{}\n", value)
            },
            Instruction::Return(None) => {
                "    ret\n".to_string()
            },
            Instruction::ReturnInPlace(value) => {
                // RVO: QBE会优化
                format!("    ret %{} # RVO\n", value)
            },
            Instruction::Call { dest, func, args } => {
                let args_str: Vec<String> = args.iter()
                    .map(|a| format!("w %{}", a))
                    .collect();
                if let Some(d) = dest {
                    format!("    %{} =w call ${}({})\n", d, func, args_str.join(", "))
                } else {
                    format!("    call ${}({})\n", func, args_str.join(", "))
                }
            },
            Instruction::Br(label) => {
                format!("    jmp @{}\n", label)
            },
            Instruction::CondBr { cond, true_bb, false_bb } => {
                format!("    jnz %{}, @{}, @{}\n", cond, true_bb, false_bb)
            },
            Instruction::Label(name) => {
                format!("@{}\n", name)
            },
            _ => format!("    # {:?}\n", inst),
        }
    }
}

impl CodegenBackend for QBEBackend {
    fn name(&self) -> &str {
        "QBE"
    }
    
    fn generate(&self, module: &Module) -> Result<String, Box<dyn Error>> {
        let mut output = String::new();
        
        // QBE头部
        output.push_str("# Chim generated QBE IL\n\n");
        
        // 外部函数声明
        output.push_str("export function $println(l %s) {\n@start\n    ret\n}\n\n");
        output.push_str("export function $print(l %s) {\n@start\n    ret\n}\n\n");
        
        // 生成函数
        for func in &module.functions {
            output.push_str(&self.generate_function(func));
        }
        
        Ok(output)
    }
    
    fn file_extension(&self) -> &str {
        "ssa"
    }
}
