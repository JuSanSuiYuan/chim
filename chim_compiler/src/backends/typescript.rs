use crate::backend::CodegenBackend;
use crate::ir::{Module, Function, Instruction, Type};
use std::error::Error;

/// TypeScript后端 - 生成TypeScript代码
/// 
/// 特点：
/// - 完整的类型系统
/// - 接口和类型别名
/// - 严格的类型检查
/// - 现代ES特性
pub struct TypeScriptBackend {
    strict_mode: bool,
    use_interfaces: bool,
}

impl TypeScriptBackend {
    pub fn new() -> Self {
        Self {
            strict_mode: true,
            use_interfaces: true,
        }
    }
    
    /// 将IR类型转换为TypeScript类型
    fn generate_type(&self, ty: &Type) -> String {
        match ty {
            Type::Void => "void".to_string(),
            Type::Bool => "boolean".to_string(),
            Type::Int32 | Type::Int64 => "number".to_string(),
            Type::Float32 | Type::Float64 => "number".to_string(),
            Type::String => "string".to_string(),
            Type::Ptr(_) | Type::Ref(_) | Type::MutRef(_) => {
                "any".to_string()
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
        
        sig.push_str("function ");
        sig.push_str(&func.name);
        sig.push('(');
        
        // 参数列表（带类型）
        let params: Vec<String> = func.params.iter()
            .map(|(name, ty)| format!("{}: {}", name, self.generate_type(ty)))
            .collect();
        sig.push_str(&params.join(", "));
        sig.push(')');
        
        // 返回类型
        sig.push_str(": ");
        sig.push_str(&self.generate_type(&func.return_type));
        
        sig
    }
    
    /// 生成指令代码
    fn generate_instruction(&self, inst: &Instruction, indent: usize) -> String {
        let ind = "  ".repeat(indent);
        
        match inst {
            Instruction::Alloca { dest, ty } => {
                let type_str = self.generate_type(ty);
                format!("{}let {}: {};", ind, dest, type_str)
            }
            Instruction::Store { dest, src } => {
                format!("{}{} = {};", ind, dest, src)
            }
            Instruction::Load { dest, src } => {
                format!("{}const {} = {};", ind, dest, src)
            }
            Instruction::Add { dest, left, right } => {
                format!("{}const {}: number = {} + {};", ind, dest, left, right)
            }
            Instruction::Sub { dest, left, right } => {
                format!("{}const {}: number = {} - {};", ind, dest, left, right)
            }
            Instruction::Mul { dest, left, right } => {
                format!("{}const {}: number = {} * {};", ind, dest, left, right)
            }
            Instruction::Div { dest, left, right } => {
                format!("{}const {}: number = {} / {};", ind, dest, left, right)
            }
            Instruction::Mod { dest, left, right } => {
                format!("{}const {}: number = {} % {};", ind, dest, left, right)
            }
            Instruction::Eq { dest, left, right } => {
                format!("{}const {}: boolean = ({} === {});", ind, dest, left, right)
            }
            Instruction::Ne { dest, left, right } => {
                format!("{}const {}: boolean = ({} !== {});", ind, dest, left, right)
            }
            Instruction::Lt { dest, left, right } => {
                format!("{}const {}: boolean = ({} < {});", ind, dest, left, right)
            }
            Instruction::Le { dest, left, right } => {
                format!("{}const {}: boolean = ({} <= {});", ind, dest, left, right)
            }
            Instruction::Gt { dest, left, right } => {
                format!("{}const {}: boolean = ({} > {});", ind, dest, left, right)
            }
            Instruction::Ge { dest, left, right } => {
                format!("{}const {}: boolean = ({} >= {});", ind, dest, left, right)
            }
            Instruction::And { dest, left, right } => {
                format!("{}const {}: boolean = ({} && {});", ind, dest, left, right)
            }
            Instruction::Or { dest, left, right } => {
                format!("{}const {}: boolean = ({} || {});", ind, dest, left, right)
            }
            Instruction::Not { dest, src } => {
                format!("{}const {}: boolean = !{};", ind, dest, src)
            }
            Instruction::Call { dest, func, args } => {
                if let Some(d) = dest {
                    format!("{}const {} = {}({});", ind, d, func, args.join(", "))
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
            code.push_str(&self.generate_instruction(inst, 1));
            code.push('\n');
        }
        
        code.push_str("}\n\n");
        code
    }
}

impl CodegenBackend for TypeScriptBackend {
    fn name(&self) -> &str {
        "TypeScript"
    }
    
    fn generate(&self, module: &Module) -> Result<String, Box<dyn Error>> {
        let mut output = String::new();
        
        // 文件头部
        output.push_str("// Generated by Chim Compiler - TypeScript Backend\n");
        output.push_str("// Target: TypeScript 4.0+\n");
        output.push_str("// Features: strict types, interfaces, modern ES\n\n");
        
        // TypeScript配置注释
        if self.strict_mode {
            output.push_str("/* eslint-disable */\n");
            output.push_str("// @ts-check\n\n");
        }
        
        // 生成所有函数
        for func in &module.functions {
            output.push_str(&self.generate_function(func));
        }
        
        // 导出函数
        output.push_str("// Export functions\n");
        output.push_str("export {\n");
        for (i, func) in module.functions.iter().enumerate() {
            if i > 0 {
                output.push_str(",\n");
            }
            output.push_str(&format!("  {}", func.name));
        }
        output.push_str("\n};\n");
        
        Ok(output)
    }
    
    fn file_extension(&self) -> &str {
        "ts"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_typescript_backend() {
        let backend = TypeScriptBackend::new();
        assert_eq!(backend.name(), "TypeScript");
        assert_eq!(backend.file_extension(), "ts");
    }
}
