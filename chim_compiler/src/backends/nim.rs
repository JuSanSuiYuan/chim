use crate::backend::CodegenBackend;
use crate::ir::{Module, Function, Instruction, Type};
use std::error::Error;

/// Nim后端 - 生成Nim代码
/// 
/// 特点：
/// - Python风格语法
/// - 强大的元编程
/// - 编译到C/C++
/// - 高性能
pub struct NimBackend {
    use_strict: bool,
}

impl NimBackend {
    pub fn new() -> Self {
        Self {
            use_strict: true,
        }
    }
    
    /// 将IR类型转换为Nim类型
    fn generate_type(&self, ty: &Type) -> String {
        match ty {
            Type::Void => "void".to_string(),
            Type::Bool => "bool".to_string(),
            Type::Int32 => "int32".to_string(),
            Type::Int64 => "int64".to_string(),
            Type::Float32 => "float32".to_string(),
            Type::Float64 => "float64".to_string(),
            Type::String => "string".to_string(),
            Type::Ptr(inner) | Type::Ref(inner) | Type::MutRef(inner) => {
                format!("ref {}", self.generate_type(inner))
            }
            Type::Array(inner, size) => {
                format!("array[{}, {}]", size, self.generate_type(inner))
            }
            Type::Struct(name) => name.clone(),
        }
    }
    
    /// 生成函数签名
    fn generate_function_signature(&self, func: &Function) -> String {
        let mut sig = String::new();
        
        sig.push_str("proc ");
        sig.push_str(&func.name);
        sig.push('(');
        
        // 参数列表
        let params: Vec<String> = func.params.iter()
            .map(|(name, ty)| format!("{}: {}", name, self.generate_type(ty)))
            .collect();
        sig.push_str(&params.join("; "));
        sig.push(')');
        
        // 返回类型
        let ret_type = self.generate_type(&func.return_type);
        if ret_type != "void" {
            sig.push_str(": ");
            sig.push_str(&ret_type);
        }
        
        sig
    }
    
    /// 生成指令代码
    fn generate_instruction(&self, inst: &Instruction, indent: usize) -> String {
        let ind = "  ".repeat(indent);
        
        match inst {
            Instruction::Alloca { dest, ty } => {
                let type_str = self.generate_type(ty);
                format!("{}var {}: {}", ind, dest, type_str)
            }
            Instruction::Store { dest, src } => {
                format!("{}{} = {}", ind, dest, src)
            }
            Instruction::Load { dest, src } => {
                format!("{}var {} = {}", ind, dest, src)
            }
            Instruction::Add { dest, left, right } => {
                format!("{}var {} = {} + {}", ind, dest, left, right)
            }
            Instruction::Sub { dest, left, right } => {
                format!("{}var {} = {} - {}", ind, dest, left, right)
            }
            Instruction::Mul { dest, left, right } => {
                format!("{}var {} = {} * {}", ind, dest, left, right)
            }
            Instruction::Div { dest, left, right } => {
                format!("{}var {} = {} div {}", ind, dest, left, right)
            }
            Instruction::Mod { dest, left, right } => {
                format!("{}var {} = {} mod {}", ind, dest, left, right)
            }
            Instruction::Eq { dest, left, right } => {
                format!("{}var {} = ({} == {})", ind, dest, left, right)
            }
            Instruction::Ne { dest, left, right } => {
                format!("{}var {} = ({} != {})", ind, dest, left, right)
            }
            Instruction::Lt { dest, left, right } => {
                format!("{}var {} = ({} < {})", ind, dest, left, right)
            }
            Instruction::Le { dest, left, right } => {
                format!("{}var {} = ({} <= {})", ind, dest, left, right)
            }
            Instruction::Gt { dest, left, right } => {
                format!("{}var {} = ({} > {})", ind, dest, left, right)
            }
            Instruction::Ge { dest, left, right } => {
                format!("{}var {} = ({} >= {})", ind, dest, left, right)
            }
            Instruction::And { dest, left, right } => {
                format!("{}var {} = ({} and {})", ind, dest, left, right)
            }
            Instruction::Or { dest, left, right } => {
                format!("{}var {} = ({} or {})", ind, dest, left, right)
            }
            Instruction::Not { dest, src } => {
                format!("{}var {} = not {}", ind, dest, src)
            }
            Instruction::Call { dest, func, args } => {
                if let Some(d) = dest {
                    format!("{}var {} = {}({})", ind, d, func, args.join(", "))
                } else {
                    format!("{}{}({})", ind, func, args.join(", "))
                }
            }
            Instruction::Return(Some(value)) => {
                format!("{}result = {}", ind, value)
            }
            Instruction::Return(None) => {
                format!("{}discard", ind)
            }
            Instruction::ReturnInPlace(value) => {
                format!("{}result = {} # RVO", ind, value)
            }
            _ => format!("{}# Unsupported instruction", ind),
        }
    }
    
    /// 生成函数代码
    fn generate_function(&self, func: &Function) -> String {
        let mut code = String::new();
        
        // 函数签名
        code.push_str(&self.generate_function_signature(func));
        code.push_str(" =\n");
        
        // 函数体
        for inst in &func.body {
            code.push_str(&self.generate_instruction(inst, 1));
            code.push('\n');
        }
        
        code.push('\n');
        code
    }
}

impl CodegenBackend for NimBackend {
    fn name(&self) -> &str {
        "Nim"
    }
    
    fn generate(&self, module: &Module) -> Result<String, Box<dyn Error>> {
        let mut output = String::new();
        
        // 文件头部
        output.push_str("# Generated by Chim Compiler - Nim Backend\n");
        output.push_str("# Target: Nim Language\n");
        output.push_str("# Features: metaprogramming, compiles to C/C++, high performance\n\n");
        
        // 导入标准库
        output.push_str("import std/[os, strutils]\n\n");
        
        // 生成所有函数
        for func in &module.functions {
            output.push_str(&self.generate_function(func));
        }
        
        // 主程序入口（如果有main函数）
        if module.functions.iter().any(|f| f.name == "main") {
            output.push_str("# Main entry point\n");
            output.push_str("when isMainModule:\n");
            output.push_str("  discard main()\n");
        }
        
        Ok(output)
    }
    
    fn file_extension(&self) -> &str {
        "nim"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_nim_backend() {
        let backend = NimBackend::new();
        assert_eq!(backend.name(), "Nim");
        assert_eq!(backend.file_extension(), "nim");
    }
}
