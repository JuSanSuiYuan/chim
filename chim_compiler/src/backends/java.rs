use crate::backend::CodegenBackend;
use crate::ir::{Module, Function, Instruction, Type};
use std::error::Error;

/// Java后端 - 生成Java 11+代码
/// 
/// 特点：
/// - Modern Java 11+ 特性
/// - 类型安全和泛型支持
/// - 自动内存管理
/// - JIT编译优化
pub struct JavaBackend {
    use_var_syntax: bool,  // 使用var关键字（Java 10+）
    enable_records: bool,   // 使用record类型（Java 14+）
}

impl JavaBackend {
    pub fn new() -> Self {
        Self {
            use_var_syntax: true,
            enable_records: true,
        }
    }
    
    /// 将IR类型转换为Java类型
    fn generate_type(&self, ty: &Type) -> String {
        match ty {
            Type::Void => "void".to_string(),
            Type::Bool => "boolean".to_string(),
            Type::Int32 => "int".to_string(),
            Type::Int64 => "long".to_string(),
            Type::Float32 => "float".to_string(),
            Type::Float64 => "double".to_string(),
            Type::String => "String".to_string(),
            Type::Ptr(_) | Type::Ref(_) | Type::MutRef(_) => {
                // Java使用引用，不需要显式指针
                "Object".to_string()
            }
            Type::Array(inner, size) => {
                format!("{}[]", self.generate_type(inner))
            }
            Type::Struct(name) => name.clone(),
        }
    }
    
    /// 生成函数签名
    fn generate_function_signature(&self, func: &Function) -> String {
        let mut sig = String::new();
        
        // 访问修饰符
        sig.push_str("    public static ");
        
        // 返回类型
        let return_type = self.generate_type(&func.return_type);
        sig.push_str(&return_type);
        sig.push(' ');
        
        // 函数名
        sig.push_str(&func.name);
        sig.push('(');
        
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
                let type_str = self.generate_type(ty);
                if self.use_var_syntax {
                    format!("{}var {} = new {}();", ind, dest, type_str)
                } else {
                    format!("{}{} {};", ind, type_str, dest)
                }
            }
            Instruction::Store { dest, src } => {
                format!("{}{} = {};", ind, dest, src)
            }
            Instruction::Load { dest, src } => {
                if self.use_var_syntax {
                    format!("{}var {} = {};", ind, dest, src)
                } else {
                    format!("{}Object {} = {};", ind, dest, src)
                }
            }
            Instruction::Add { dest, left, right } => {
                if self.use_var_syntax {
                    format!("{}var {} = {} + {};", ind, dest, left, right)
                } else {
                    format!("{}int {} = {} + {};", ind, dest, left, right)
                }
            }
            Instruction::Sub { dest, left, right } => {
                if self.use_var_syntax {
                    format!("{}var {} = {} - {};", ind, dest, left, right)
                } else {
                    format!("{}int {} = {} - {};", ind, dest, left, right)
                }
            }
            Instruction::Mul { dest, left, right } => {
                if self.use_var_syntax {
                    format!("{}var {} = {} * {};", ind, dest, left, right)
                } else {
                    format!("{}int {} = {} * {};", ind, dest, left, right)
                }
            }
            Instruction::Div { dest, left, right } => {
                if self.use_var_syntax {
                    format!("{}var {} = {} / {};", ind, dest, left, right)
                } else {
                    format!("{}int {} = {} / {};", ind, dest, left, right)
                }
            }
            Instruction::Mod { dest, left, right } => {
                if self.use_var_syntax {
                    format!("{}var {} = {} % {};", ind, dest, left, right)
                } else {
                    format!("{}int {} = {} % {};", ind, dest, left, right)
                }
            }
            Instruction::Eq { dest, left, right } => {
                if self.use_var_syntax {
                    format!("{}var {} = ({} == {});", ind, dest, left, right)
                } else {
                    format!("{}boolean {} = ({} == {});", ind, dest, left, right)
                }
            }
            Instruction::Ne { dest, left, right } => {
                if self.use_var_syntax {
                    format!("{}var {} = ({} != {});", ind, dest, left, right)
                } else {
                    format!("{}boolean {} = ({} != {});", ind, dest, left, right)
                }
            }
            Instruction::Lt { dest, left, right } => {
                if self.use_var_syntax {
                    format!("{}var {} = ({} < {});", ind, dest, left, right)
                } else {
                    format!("{}boolean {} = ({} < {});", ind, dest, left, right)
                }
            }
            Instruction::Le { dest, left, right } => {
                if self.use_var_syntax {
                    format!("{}var {} = ({} <= {});", ind, dest, left, right)
                } else {
                    format!("{}boolean {} = ({} <= {});", ind, dest, left, right)
                }
            }
            Instruction::Gt { dest, left, right } => {
                if self.use_var_syntax {
                    format!("{}var {} = ({} > {});", ind, dest, left, right)
                } else {
                    format!("{}boolean {} = ({} > {});", ind, dest, left, right)
                }
            }
            Instruction::Ge { dest, left, right } => {
                if self.use_var_syntax {
                    format!("{}var {} = ({} >= {});", ind, dest, left, right)
                } else {
                    format!("{}boolean {} = ({} >= {});", ind, dest, left, right)
                }
            }
            Instruction::And { dest, left, right } => {
                if self.use_var_syntax {
                    format!("{}var {} = ({} && {});", ind, dest, left, right)
                } else {
                    format!("{}boolean {} = ({} && {});", ind, dest, left, right)
                }
            }
            Instruction::Or { dest, left, right } => {
                if self.use_var_syntax {
                    format!("{}var {} = ({} || {});", ind, dest, left, right)
                } else {
                    format!("{}boolean {} = ({} || {});", ind, dest, left, right)
                }
            }
            Instruction::Not { dest, src } => {
                if self.use_var_syntax {
                    format!("{}var {} = !{};", ind, dest, src)
                } else {
                    format!("{}boolean {} = !{};", ind, dest, src)
                }
            }
            Instruction::Call { dest, func, args } => {
                if let Some(d) = dest {
                    if self.use_var_syntax {
                        format!("{}var {} = {}({});", ind, d, func, args.join(", "))
                    } else {
                        format!("{}Object {} = {}({});", ind, d, func, args.join(", "))
                    }
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
            code.push_str(&self.generate_instruction(inst, 2));
            code.push('\n');
        }
        
        code.push_str("    }\n\n");
        code
    }
}

impl CodegenBackend for JavaBackend {
    fn name(&self) -> &str {
        "Java"
    }
    
    fn generate(&self, module: &Module) -> Result<String, Box<dyn Error>> {
        let mut output = String::new();
        
        // 文件头部注释
        output.push_str("// Generated by Chim Compiler - Java Backend\n");
        output.push_str("// Target: Java 11+\n");
        output.push_str("// Features: var syntax, type inference, modern APIs\n\n");
        
        // 类定义
        output.push_str("public class ChimGenerated {\n\n");
        
        // 生成所有函数
        for func in &module.functions {
            output.push_str(&self.generate_function(func));
        }
        
        // 主方法（如果有main函数）
        if module.functions.iter().any(|f| f.name == "main") {
            output.push_str("    public static void main(String[] args) {\n");
            output.push_str("        main();\n");
            output.push_str("    }\n");
        }
        
        output.push_str("}\n");
        
        Ok(output)
    }
    
    fn file_extension(&self) -> &str {
        "java"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_java_backend() {
        let backend = JavaBackend::new();
        assert_eq!(backend.name(), "Java");
        assert_eq!(backend.file_extension(), "java");
    }
}
