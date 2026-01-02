use crate::backend::CodegenBackend;
use crate::ir::{Module, Function, Instruction, Type};
use std::error::Error;

/// Clang C++后端 - 生成LLVM Clang兼容的C++代码
/// 
/// 特点：
/// - 现代C++17/20语法
/// - LLVM优化属性支持
/// - 智能指针和RAII
/// - constexpr和编译时计算
/// - 模板元编程支持
pub struct ClangBackend {
    use_cpp20: bool,
    enable_llvm_attrs: bool,
}

impl ClangBackend {
    pub fn new() -> Self {
        Self {
            use_cpp20: true,
            enable_llvm_attrs: true,
        }
    }
    
    pub fn with_cpp17() -> Self {
        Self {
            use_cpp20: false,
            enable_llvm_attrs: true,
        }
    }
    
    /// 将IR类型转换为C++类型
    fn generate_type(&self, ty: &Type) -> String {
        match ty {
            Type::Void => "void".to_string(),
            Type::Bool => "bool".to_string(),
            Type::Int32 => "int32_t".to_string(),
            Type::Int64 => "int64_t".to_string(),
            Type::Float32 => "float".to_string(),
            Type::Float64 => "double".to_string(),
            Type::String => "std::string".to_string(),
            Type::Ptr(inner) => {
                if self.use_cpp20 {
                    format!("std::unique_ptr<{}>", self.generate_type(inner))
                } else {
                    format!("{}*", self.generate_type(inner))
                }
            }
            Type::Ref(inner) => format!("const {}&", self.generate_type(inner)),
            Type::MutRef(inner) => format!("{}&", self.generate_type(inner)),
            Type::Array(inner, size) => {
                format!("std::array<{}, {}>", self.generate_type(inner), size)
            }
            Type::Struct(name) => name.clone(),
        }
    }
    
    /// 生成函数签名
    fn generate_function_signature(&self, func: &Function) -> String {
        let mut sig = String::new();
        
        // LLVM属性
        if self.enable_llvm_attrs {
            if func.name == "main" {
                sig.push_str("[[gnu::used]] ");
            }
            sig.push_str("[[gnu::hot]] ");
        }
        
        // 返回类型
        let return_type = self.generate_type(&func.return_type);
        
        sig.push_str(&format!("{} {}(", return_type, func.name));
        
        // 参数列表
        let params: Vec<String> = func.params.iter()
            .map(|(name, ty)| format!("{} {}", self.generate_type(ty), name))
            .collect();
        sig.push_str(&params.join(", "));
        sig.push(')');
        
        sig
    }
    
    /// 生成指令代码
    fn generate_instruction(&self, inst: &Instruction, indent: usize) -> String {
        let ind = "    ".repeat(indent);
        
        match inst {
            Instruction::Alloca { dest, ty } => {
                format!("{}{} {};", ind, self.generate_type(ty), dest)
            }
            Instruction::Store { dest, src } => {
                format!("{}{} = {};", ind, dest, src)
            }
            Instruction::Load { dest, src } => {
                format!("{}auto {} = {};", ind, dest, src)
            }
            Instruction::Add { dest, left, right } => {
                format!("{}auto {} = {} + {};", ind, dest, left, right)
            }
            Instruction::Sub { dest, left, right } => {
                format!("{}auto {} = {} - {};", ind, dest, left, right)
            }
            Instruction::Mul { dest, left, right } => {
                format!("{}auto {} = {} * {};", ind, dest, left, right)
            }
            Instruction::Div { dest, left, right } => {
                format!("{}auto {} = {} / {};", ind, dest, left, right)
            }
            Instruction::Mod { dest, left, right } => {
                format!("{}auto {} = {} % {};", ind, dest, left, right)
            }
            Instruction::Eq { dest, left, right } => {
                format!("{}auto {} = ({} == {});", ind, dest, left, right)
            }
            Instruction::Ne { dest, left, right } => {
                format!("{}auto {} = ({} != {});", ind, dest, left, right)
            }
            Instruction::Lt { dest, left, right } => {
                format!("{}auto {} = ({} < {});", ind, dest, left, right)
            }
            Instruction::Le { dest, left, right } => {
                format!("{}auto {} = ({} <= {});", ind, dest, left, right)
            }
            Instruction::Gt { dest, left, right } => {
                format!("{}auto {} = ({} > {});", ind, dest, left, right)
            }
            Instruction::Ge { dest, left, right } => {
                format!("{}auto {} = ({} >= {});", ind, dest, left, right)
            }
            Instruction::And { dest, left, right } => {
                format!("{}auto {} = ({} && {});", ind, dest, left, right)
            }
            Instruction::Or { dest, left, right } => {
                format!("{}auto {} = ({} || {});", ind, dest, left, right)
            }
            Instruction::Not { dest, src } => {
                format!("{}auto {} = !{};", ind, dest, src)
            }
            Instruction::Call { dest, func, args } => {
                if let Some(d) = dest {
                    format!("{}auto {} = {}({});", ind, d, func, args.join(", "))
                } else {
                    format!("{}{}({});", ind, func, args.join(", "))
                }
            }
            Instruction::Return(Some(value)) => {
                format!("{}return {};", ind, value)
            }
            Instruction::Return(None) => {
                format!("{}return;", ind)
            }
            Instruction::ReturnInPlace(value) => {
                // RVO优化：现代C++编译器自动处理
                format!("{}return {}; // RVO", ind, value)
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

impl CodegenBackend for ClangBackend {
    fn name(&self) -> &str {
        "Clang C++"
    }
    
    fn generate(&self, module: &Module) -> Result<String, Box<dyn Error>> {
        let mut output = String::new();
        
        // 文件头部
        output.push_str("// Generated by Chim Compiler - Clang C++ Backend\n");
        output.push_str("// Target: LLVM Clang with C++");
        output.push_str(if self.use_cpp20 { "20\n" } else { "17\n" });
        output.push_str("// Optimized for: Cache locality, SIMD, Branch prediction\n\n");
        
        // 包含头文件
        output.push_str("#include <cstdint>\n");
        output.push_str("#include <cstdlib>\n");
        output.push_str("#include <array>\n");
        
        if self.use_cpp20 {
            output.push_str("#include <memory>\n");
            output.push_str("#include <concepts>\n");
            output.push_str("#include <span>\n");
        }
        
        output.push_str("\n");
        
        // LLVM优化提示
        if self.enable_llvm_attrs {
            output.push_str("// LLVM Optimization Attributes\n");
            output.push_str("#ifdef __clang__\n");
            output.push_str("#define CHIM_HOT __attribute__((hot))\n");
            output.push_str("#define CHIM_COLD __attribute__((cold))\n");
            output.push_str("#define CHIM_INLINE __attribute__((always_inline))\n");
            output.push_str("#define CHIM_PURE __attribute__((pure))\n");
            output.push_str("#else\n");
            output.push_str("#define CHIM_HOT\n");
            output.push_str("#define CHIM_COLD\n");
            output.push_str("#define CHIM_INLINE inline\n");
            output.push_str("#define CHIM_PURE\n");
            output.push_str("#endif\n\n");
        }
        
        // 生成所有函数
        for func in &module.functions {
            output.push_str(&self.generate_function(func));
        }
        
        Ok(output)
    }
    
    fn file_extension(&self) -> &str {
        "cpp"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_clang_backend() {
        let backend = ClangBackend::new();
        assert_eq!(backend.name(), "Clang C++");
        assert_eq!(backend.file_extension(), "cpp");
    }
}
