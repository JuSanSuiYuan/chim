use crate::backend::CodegenBackend;
use crate::ir::{Module, Function, Instruction, Type};
use std::error::Error;

/// C#后端 - 生成C# 9.0+代码
/// 
/// 特点：
/// - Modern C# 9.0+ 特性
/// - 顶层语句支持
/// - 记录类型(record)
/// - 模式匹配
pub struct CSharpBackend {
    use_top_level: bool,    // 使用顶层语句
    use_var: bool,          // 使用var关键字
    use_nullable: bool,     // 使用可空引用类型
}

impl CSharpBackend {
    pub fn new() -> Self {
        Self {
            use_top_level: true,
            use_var: true,
            use_nullable: true,
        }
    }
    
    /// 将IR类型转换为C#类型
    fn generate_type(&self, ty: &Type) -> String {
        match ty {
            Type::Void => "void".to_string(),
            Type::Bool => "bool".to_string(),
            Type::Int32 => "int".to_string(),
            Type::Int64 => "long".to_string(),
            Type::Float32 => "float".to_string(),
            Type::Float64 => "double".to_string(),
            Type::String => "string".to_string(),
            Type::Ptr(inner) | Type::Ref(inner) | Type::MutRef(inner) => {
                format!("{}?", self.generate_type(inner))
            }
            Type::Array(inner, _) => {
                format!("{}[]", self.generate_type(inner))
            }
            Type::Struct(name) => name.clone(),
        }
    }
    
    /// 生成函数签名
    fn generate_function_signature(&self, func: &Function) -> String {
        let mut sig = String::new();
        
        // 访问修饰符
        sig.push_str("    static ");
        
        // 返回类型
        let return_type = self.generate_type(&func.return_type);
        sig.push_str(&return_type);
        sig.push(' ');
        
        // 函数名（首字母大写，遵循C#命名规范）
        let func_name = Self::to_pascal_case(&func.name);
        sig.push_str(&func_name);
        sig.push('(');
        
        // 参数列表
        let params: Vec<String> = func.params.iter()
            .map(|(name, ty)| format!("{} {}", self.generate_type(ty), name))
            .collect();
        sig.push_str(&params.join(", "));
        sig.push(')');
        
        sig
    }
    
    /// 转换为PascalCase命名
    fn to_pascal_case(s: &str) -> String {
        if s.is_empty() {
            return s.to_string();
        }
        let mut result = String::new();
        let mut capitalize_next = true;
        
        for ch in s.chars() {
            if ch == '_' {
                capitalize_next = true;
            } else if capitalize_next {
                result.push(ch.to_uppercase().next().unwrap());
                capitalize_next = false;
            } else {
                result.push(ch);
            }
        }
        result
    }
    
    /// 生成指令代码
    fn generate_instruction(&self, inst: &Instruction, indent: usize) -> String {
        let ind = "    ".repeat(indent);
        
        match inst {
            Instruction::Alloca { dest, ty } => {
                if self.use_var {
                    format!("{}var {};", ind, dest)
                } else {
                    format!("{}{} {};", ind, self.generate_type(ty), dest)
                }
            }
            Instruction::Store { dest, src } => {
                format!("{}{} = {};", ind, dest, src)
            }
            Instruction::Load { dest, src } => {
                if self.use_var {
                    format!("{}var {} = {};", ind, dest, src)
                } else {
                    format!("{}var {} = {};", ind, dest, src)
                }
            }
            Instruction::Add { dest, left, right } => {
                if self.use_var {
                    format!("{}var {} = {} + {};", ind, dest, left, right)
                } else {
                    format!("{}int {} = {} + {};", ind, dest, left, right)
                }
            }
            Instruction::Sub { dest, left, right } => {
                if self.use_var {
                    format!("{}var {} = {} - {};", ind, dest, left, right)
                } else {
                    format!("{}int {} = {} - {};", ind, dest, left, right)
                }
            }
            Instruction::Mul { dest, left, right } => {
                if self.use_var {
                    format!("{}var {} = {} * {};", ind, dest, left, right)
                } else {
                    format!("{}int {} = {} * {};", ind, dest, left, right)
                }
            }
            Instruction::Div { dest, left, right } => {
                if self.use_var {
                    format!("{}var {} = {} / {};", ind, dest, left, right)
                } else {
                    format!("{}int {} = {} / {};", ind, dest, left, right)
                }
            }
            Instruction::Mod { dest, left, right } => {
                if self.use_var {
                    format!("{}var {} = {} % {};", ind, dest, left, right)
                } else {
                    format!("{}int {} = {} % {};", ind, dest, left, right)
                }
            }
            Instruction::Eq { dest, left, right } => {
                if self.use_var {
                    format!("{}var {} = ({} == {});", ind, dest, left, right)
                } else {
                    format!("{}bool {} = ({} == {});", ind, dest, left, right)
                }
            }
            Instruction::Ne { dest, left, right } => {
                if self.use_var {
                    format!("{}var {} = ({} != {});", ind, dest, left, right)
                } else {
                    format!("{}bool {} = ({} != {});", ind, dest, left, right)
                }
            }
            Instruction::Lt { dest, left, right } => {
                if self.use_var {
                    format!("{}var {} = ({} < {});", ind, dest, left, right)
                } else {
                    format!("{}bool {} = ({} < {});", ind, dest, left, right)
                }
            }
            Instruction::Le { dest, left, right } => {
                if self.use_var {
                    format!("{}var {} = ({} <= {});", ind, dest, left, right)
                } else {
                    format!("{}bool {} = ({} <= {});", ind, dest, left, right)
                }
            }
            Instruction::Gt { dest, left, right } => {
                if self.use_var {
                    format!("{}var {} = ({} > {});", ind, dest, left, right)
                } else {
                    format!("{}bool {} = ({} > {});", ind, dest, left, right)
                }
            }
            Instruction::Ge { dest, left, right } => {
                if self.use_var {
                    format!("{}var {} = ({} >= {});", ind, dest, left, right)
                } else {
                    format!("{}bool {} = ({} >= {});", ind, dest, left, right)
                }
            }
            Instruction::And { dest, left, right } => {
                if self.use_var {
                    format!("{}var {} = ({} && {});", ind, dest, left, right)
                } else {
                    format!("{}bool {} = ({} && {});", ind, dest, left, right)
                }
            }
            Instruction::Or { dest, left, right } => {
                if self.use_var {
                    format!("{}var {} = ({} || {});", ind, dest, left, right)
                } else {
                    format!("{}bool {} = ({} || {});", ind, dest, left, right)
                }
            }
            Instruction::Not { dest, src } => {
                if self.use_var {
                    format!("{}var {} = !{};", ind, dest, src)
                } else {
                    format!("{}bool {} = !{};", ind, dest, src)
                }
            }
            Instruction::Call { dest, func, args } => {
                let func_name = Self::to_pascal_case(func);
                if let Some(d) = dest {
                    if self.use_var {
                        format!("{}var {} = {}({});", ind, d, func_name, args.join(", "))
                    } else {
                        format!("{}var {} = {}({});", ind, d, func_name, args.join(", "))
                    }
                } else {
                    format!("{}{}({});", ind, func_name, args.join(", "))
                }
            }
            Instruction::Return(Some(value)) => {
                format!("{}return {};", ind, value)
            }
            Instruction::Return(None) => {
                format!("{}return;", ind)
            }
            Instruction::ReturnInPlace(value) => {
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
        code.push_str("\n    {\n");
        
        // 函数体
        for inst in &func.body {
            code.push_str(&self.generate_instruction(inst, 2));
            code.push('\n');
        }
        
        code.push_str("    }\n\n");
        code
    }
}

impl CodegenBackend for CSharpBackend {
    fn name(&self) -> &str {
        "C#"
    }
    
    fn generate(&self, module: &Module) -> Result<String, Box<dyn Error>> {
        let mut output = String::new();
        
        // 文件头部
        output.push_str("// Generated by Chim Compiler - C# Backend\n");
        output.push_str("// Target: C# 9.0+ (.NET 5.0+)\n");
        output.push_str("// Features: top-level statements, var, nullable references\n\n");
        
        // 命名空间和using
        output.push_str("using System;\n");
        output.push_str("using System.Collections.Generic;\n");
        
        if self.use_nullable {
            output.push_str("#nullable enable\n");
        }
        
        output.push_str("\n");
        
        // 类定义
        output.push_str("public class ChimGenerated\n{\n");
        
        // 生成所有函数
        for func in &module.functions {
            output.push_str(&self.generate_function(func));
        }
        
        // 主方法
        if module.functions.iter().any(|f| f.name == "main") {
            output.push_str("    static void Main(string[] args)\n");
            output.push_str("    {\n");
            output.push_str("        Main();\n");
            output.push_str("    }\n");
        }
        
        output.push_str("}\n");
        
        Ok(output)
    }
    
    fn file_extension(&self) -> &str {
        "cs"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_csharp_backend() {
        let backend = CSharpBackend::new();
        assert_eq!(backend.name(), "C#");
        assert_eq!(backend.file_extension(), "cs");
    }
    
    #[test]
    fn test_pascal_case() {
        assert_eq!(CSharpBackend::to_pascal_case("test_function"), "TestFunction");
        assert_eq!(CSharpBackend::to_pascal_case("main"), "Main");
    }
}
