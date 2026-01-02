use crate::backend::CodegenBackend;
use crate::ir::{Module, Function, Instruction, Type};
use std::error::Error;

/// V语言后端 - 生成V语言代码
/// 
/// 特点：
/// - 简洁的语法
/// - 编译期内存安全
/// - 零依赖
/// - 快速编译
pub struct VBackend {
    use_autofree: bool,  // 使用自动内存管理
}

impl VBackend {
    pub fn new() -> Self {
        Self {
            use_autofree: true,
        }
    }
    
    /// 将IR类型转换为V类型
    fn generate_type(&self, ty: &Type) -> String {
        match ty {
            Type::Void => "".to_string(),
            Type::Bool => "bool".to_string(),
            Type::Int32 => "int".to_string(),
            Type::Int64 => "i64".to_string(),
            Type::Float32 => "f32".to_string(),
            Type::Float64 => "f64".to_string(),
            Type::String => "string".to_string(),
            Type::Ptr(inner) | Type::Ref(inner) | Type::MutRef(inner) => {
                format!("&{}", self.generate_type(inner))
            }
            Type::Array(inner, size) => {
                format!("[{}]{}", size, self.generate_type(inner))
            }
            Type::Struct(name) => name.clone(),
        }
    }
    
    /// 生成函数签名
    fn generate_function_signature(&self, func: &Function) -> String {
        let mut sig = String::new();
        
        sig.push_str("fn ");
        sig.push_str(&func.name);
        sig.push('(');
        
        // 参数列表
        let params: Vec<String> = func.params.iter()
            .map(|(name, ty)| format!("{} {}", name, self.generate_type(ty)))
            .collect();
        sig.push_str(&params.join(", "));
        sig.push(')');
        
        // 返回类型
        let ret_type = self.generate_type(&func.return_type);
        if !ret_type.is_empty() {
            sig.push(' ');
            sig.push_str(&ret_type);
        }
        
        sig
    }
    
    /// 生成指令代码
    fn generate_instruction(&self, inst: &Instruction, indent: usize) -> String {
        let ind = "\t".repeat(indent);
        
        match inst {
            Instruction::Alloca { dest, ty } => {
                let type_str = self.generate_type(ty);
                if type_str.is_empty() {
                    format!("{}mut {}", ind, dest)
                } else {
                    format!("{}mut {} := {}{{}}", ind, dest, type_str)
                }
            }
            Instruction::Store { dest, src } => {
                format!("{}{} = {}", ind, dest, src)
            }
            Instruction::Load { dest, src } => {
                format!("{}{} := {}", ind, dest, src)
            }
            Instruction::Add { dest, left, right } => {
                format!("{}{} := {} + {}", ind, dest, left, right)
            }
            Instruction::Sub { dest, left, right } => {
                format!("{}{} := {} - {}", ind, dest, left, right)
            }
            Instruction::Mul { dest, left, right } => {
                format!("{}{} := {} * {}", ind, dest, left, right)
            }
            Instruction::Div { dest, left, right } => {
                format!("{}{} := {} / {}", ind, dest, left, right)
            }
            Instruction::Mod { dest, left, right } => {
                format!("{}{} := {} % {}", ind, dest, left, right)
            }
            Instruction::Eq { dest, left, right } => {
                format!("{}{} := ({} == {})", ind, dest, left, right)
            }
            Instruction::Ne { dest, left, right } => {
                format!("{}{} := ({} != {})", ind, dest, left, right)
            }
            Instruction::Lt { dest, left, right } => {
                format!("{}{} := ({} < {})", ind, dest, left, right)
            }
            Instruction::Le { dest, left, right } => {
                format!("{}{} := ({} <= {})", ind, dest, left, right)
            }
            Instruction::Gt { dest, left, right } => {
                format!("{}{} := ({} > {})", ind, dest, left, right)
            }
            Instruction::Ge { dest, left, right } => {
                format!("{}{} := ({} >= {})", ind, dest, left, right)
            }
            Instruction::And { dest, left, right } => {
                format!("{}{} := ({} && {})", ind, dest, left, right)
            }
            Instruction::Or { dest, left, right } => {
                format!("{}{} := ({} || {})", ind, dest, left, right)
            }
            Instruction::Not { dest, src } => {
                format!("{}{} := !{}", ind, dest, src)
            }
            Instruction::Call { dest, func, args } => {
                if let Some(d) = dest {
                    format!("{}{} := {}({})", ind, d, func, args.join(", "))
                } else {
                    format!("{}{}({})", ind, func, args.join(", "))
                }
            }
            Instruction::Return(Some(value)) => {
                format!("{}return {}", ind, value)
            }
            Instruction::Return(None) => {
                format!("{}return", ind)
            }
            Instruction::ReturnInPlace(value) => {
                format!("{}return {} // RVO", ind, value)
            }
            _ => format!("{}// Unsupported instruction", ind),
        }
    }
    
    /// 生成函数代码
    fn generate_function(&self, func: &Function) -> String {
        let mut code = String::new();
        
        // 函数签名
        code.push_str(&self.generate_function_signature(func));
        code.push_str(" {\n");
        
        // 函数体
        for inst in &func.body {
            code.push_str(&self.generate_instruction(inst, 1));
            code.push('\n');
        }
        
        code.push_str("}\n\n");
        code
    }
}

impl CodegenBackend for VBackend {
    fn name(&self) -> &str {
        "V"
    }
    
    fn generate(&self, module: &Module) -> Result<String, Box<dyn Error>> {
        let mut output = String::new();
        
        // 文件头部
        output.push_str("// Generated by Chim Compiler - V Backend\n");
        output.push_str("// Target: V Language\n");
        output.push_str("// Features: autofree, compile-time safety, zero dependencies\n\n");
        
        // 模块名称
        output.push_str("module main\n\n");
        
        // 生成所有函数
        for func in &module.functions {
            output.push_str(&self.generate_function(func));
        }
        
        Ok(output)
    }
    
    fn file_extension(&self) -> &str {
        "v"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_v_backend() {
        let backend = VBackend::new();
        assert_eq!(backend.name(), "V");
        assert_eq!(backend.file_extension(), "v");
    }
}
