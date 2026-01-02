use crate::backend::CodegenBackend;
use crate::ir::{Module, Function, Instruction, Type};
use std::error::Error;

/// Swift 后端代码生成器
/// 
/// 生成 Swift 5.0+ 代码
/// 特性：
/// - 强类型系统
/// - Optional 类型
/// - 值类型和引用类型
/// - 协议导向编程
pub struct SwiftBackend {
    /// 使用 struct 还是 class
    pub prefer_structs: bool,
    /// 生成 Optional 类型
    pub use_optionals: bool,
}

impl SwiftBackend {
    pub fn new() -> Self {
        Self {
            prefer_structs: true,
            use_optionals: true,
        }
    }

    fn generate_type(&self, ty: &Type) -> String {
        match ty {
            Type::Void => "Void".to_string(),
            Type::Int32 => "Int32".to_string(),
            Type::Int64 => "Int64".to_string(),
            Type::Float32 => "Float".to_string(),
            Type::Float64 => "Double".to_string(),
            Type::Bool => "Bool".to_string(),
            Type::String => "String".to_string(),
            Type::Ptr(inner) | Type::Ref(inner) | Type::MutRef(inner) => {
                if self.use_optionals {
                    format!("{}?", self.generate_type(inner))
                } else {
                    format!("UnsafeMutablePointer<{}>", self.generate_type(inner))
                }
            },
            Type::Array(inner, size) => {
                if *size == 0 {
                    format!("[{}]", self.generate_type(inner))
                } else {
                    format!("[{}] /* size: {} */", self.generate_type(inner), size)
                }
            },
            Type::Struct(name) => name.clone(),
        }
    }

    fn generate_function(&self, func: &Function) -> String {
        let mut code = String::new();
        
        // 函数签名
        let return_type = if matches!(func.return_type, Type::Void) {
            String::new()
        } else {
            format!(" -> {}", self.generate_type(&func.return_type))
        };
        
        let params = func.params
            .iter()
            .map(|(name, ty)| format!("{}: {}", name, self.generate_type(ty)))
            .collect::<Vec<_>>()
            .join(", ");
        
        code.push_str(&format!("func {}({}){} {{\n", func.name, params, return_type));
        
        // 函数体
        for inst in &func.body {
            code.push_str(&self.generate_instruction(inst));
        }
        
        code.push_str("}\n");
        code
    }

    fn generate_instruction(&self, inst: &Instruction) -> String {
        match inst {
            Instruction::Alloca { dest, ty } => {
                format!("    var {}: {} // alloca\n", dest, self.generate_type(ty))
            },
            Instruction::Store { dest, src } => {
                format!("    {} = {}\n", dest, src)
            },
            Instruction::Load { dest, src } => {
                format!("    let {} = {}\n", dest, src)
            },
            Instruction::Add { dest, left, right } => {
                format!("    let {} = {} + {}\n", dest, left, right)
            },
            Instruction::Sub { dest, left, right } => {
                format!("    let {} = {} - {}\n", dest, left, right)
            },
            Instruction::Mul { dest, left, right } => {
                format!("    let {} = {} * {}\n", dest, left, right)
            },
            Instruction::Div { dest, left, right } => {
                format!("    let {} = {} / {}\n", dest, left, right)
            },
            Instruction::Mod { dest, left, right } => {
                format!("    let {} = {} % {}\n", dest, left, right)
            },
            Instruction::Eq { dest, left, right } => {
                format!("    let {} = {} == {}\n", dest, left, right)
            },
            Instruction::Ne { dest, left, right } => {
                format!("    let {} = {} != {}\n", dest, left, right)
            },
            Instruction::Lt { dest, left, right } => {
                format!("    let {} = {} < {}\n", dest, left, right)
            },
            Instruction::Le { dest, left, right } => {
                format!("    let {} = {} <= {}\n", dest, left, right)
            },
            Instruction::Gt { dest, left, right } => {
                format!("    let {} = {} > {}\n", dest, left, right)
            },
            Instruction::Ge { dest, left, right } => {
                format!("    let {} = {} >= {}\n", dest, left, right)
            },
            Instruction::And { dest, left, right } => {
                format!("    let {} = {} && {}\n", dest, left, right)
            },
            Instruction::Or { dest, left, right } => {
                format!("    let {} = {} || {}\n", dest, left, right)
            },
            Instruction::Not { dest, src } => {
                format!("    let {} = !{}\n", dest, src)
            },
            Instruction::Call { dest, func, args } => {
                if let Some(dest) = dest {
                    format!("    let {} = {}({})\n", dest, func, args.join(", "))
                } else {
                    format!("    {}({})\n", func, args.join(", "))
                }
            },
            Instruction::Return(value) => {
                if let Some(val) = value {
                    format!("    return {}\n", val)
                } else {
                    "    return\n".to_string()
                }
            },
            Instruction::ReturnInPlace(value) => {
                format!("    return {} // RVO\n", value)
            },
            Instruction::Label(name) => {
                format!("    // label: {}\n", name)
            },
            Instruction::Br(target) => {
                format!("    // goto {}\n", target)
            },
            Instruction::CondBr { cond, true_bb, false_bb } => {
                format!("    if {} {{ /* goto {} */ }} else {{ /* goto {} */ }}\n", 
                    cond, true_bb, false_bb)
            },
            _ => format!("    // {:?}\n", inst),
        }
    }
}

impl CodegenBackend for SwiftBackend {
    fn name(&self) -> &str {
        "Swift"
    }

    fn generate(&self, module: &Module) -> Result<String, Box<dyn Error>> {
        let mut code = String::new();
        
        // 文件头注释
        code.push_str("// Generated by Chim Compiler - Swift Backend\n");
        code.push_str("// Target: Swift 5.0+\n");
        code.push_str("// Features: strong types, optionals, value semantics\n\n");
        
        code.push_str("import Foundation\n\n");
        
        // 生成函数
        for func in &module.functions {
            code.push_str(&self.generate_function(func));
            code.push_str("\n");
        }
        
        // 生成程序入口（如果没有 main 函数）
        if module.functions.iter().any(|f| f.name == "main") {
            code.push_str("// Entry point\n");
            code.push_str("main()\n");
        }
        
        Ok(code)
    }

    fn file_extension(&self) -> &str {
        "swift"
    }
}
