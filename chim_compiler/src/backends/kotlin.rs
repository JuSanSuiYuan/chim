use crate::backend::CodegenBackend;
use crate::ir::{Module, Function, Instruction, Type};
use std::error::Error;

/// Kotlin 后端代码生成器
/// 
/// 生成 Kotlin 1.8+ 代码
/// 特性：
/// - 数据类（data class）
/// - 协程支持
/// - 空安全类型系统
/// - 函数式编程特性
pub struct KotlinBackend {
    /// 使用 data class 用于结构体
    pub use_data_classes: bool,
    /// 生成空安全类型注解
    pub null_safety: bool,
}

impl KotlinBackend {
    pub fn new() -> Self {
        Self {
            use_data_classes: true,
            null_safety: true,
        }
    }

    fn generate_type(&self, ty: &Type) -> String {
        match ty {
            Type::Void => "Unit".to_string(),
            Type::Int32 => "Int".to_string(),
            Type::Int64 => "Long".to_string(),
            Type::Float32 => "Float".to_string(),
            Type::Float64 => "Double".to_string(),
            Type::Bool => "Boolean".to_string(),
            Type::String => "String".to_string(),
            Type::Ptr(inner) | Type::Ref(inner) | Type::MutRef(inner) => {
                format!("{}?", self.generate_type(inner))
            },
            Type::Array(inner, size) => {
                if *size == 0 {
                    format!("Array<{}>", self.generate_type(inner))
                } else {
                    format!("Array<{}> /* size: {} */", self.generate_type(inner), size)
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
            format!(": {}", self.generate_type(&func.return_type))
        };
        
        let params = func.params
            .iter()
            .map(|(name, ty)| format!("{}: {}", name, self.generate_type(ty)))
            .collect::<Vec<_>>()
            .join(", ");
        
        code.push_str(&format!("fun {}({}){} {{\n", func.name, params, return_type));
        
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
                format!("    val {} = {}\n", dest, src)
            },
            Instruction::Add { dest, left, right } => {
                format!("    val {} = {} + {}\n", dest, left, right)
            },
            Instruction::Sub { dest, left, right } => {
                format!("    val {} = {} - {}\n", dest, left, right)
            },
            Instruction::Mul { dest, left, right } => {
                format!("    val {} = {} * {}\n", dest, left, right)
            },
            Instruction::Div { dest, left, right } => {
                format!("    val {} = {} / {}\n", dest, left, right)
            },
            Instruction::Mod { dest, left, right } => {
                format!("    val {} = {} % {}\n", dest, left, right)
            },
            Instruction::Eq { dest, left, right } => {
                format!("    val {} = {} == {}\n", dest, left, right)
            },
            Instruction::Ne { dest, left, right } => {
                format!("    val {} = {} != {}\n", dest, left, right)
            },
            Instruction::Lt { dest, left, right } => {
                format!("    val {} = {} < {}\n", dest, left, right)
            },
            Instruction::Le { dest, left, right } => {
                format!("    val {} = {} <= {}\n", dest, left, right)
            },
            Instruction::Gt { dest, left, right } => {
                format!("    val {} = {} > {}\n", dest, left, right)
            },
            Instruction::Ge { dest, left, right } => {
                format!("    val {} = {} >= {}\n", dest, left, right)
            },
            Instruction::And { dest, left, right } => {
                format!("    val {} = {} && {}\n", dest, left, right)
            },
            Instruction::Or { dest, left, right } => {
                format!("    val {} = {} || {}\n", dest, left, right)
            },
            Instruction::Not { dest, src } => {
                format!("    val {} = !{}\n", dest, src)
            },
            Instruction::Call { dest, func, args } => {
                if let Some(dest) = dest {
                    format!("    val {} = {}({})\n", dest, func, args.join(", "))
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
                format!("    if ({}) {{ /* goto {} */ }} else {{ /* goto {} */ }}\n", 
                    cond, true_bb, false_bb)
            },
            _ => format!("    // {:?}\n", inst),
        }
    }
}

impl CodegenBackend for KotlinBackend {
    fn name(&self) -> &str {
        "Kotlin"
    }

    fn generate(&self, module: &Module) -> Result<String, Box<dyn Error>> {
        let mut code = String::new();
        
        // 文件头注释
        code.push_str("// Generated by Chim Compiler - Kotlin Backend\n");
        code.push_str("// Target: Kotlin 1.8+\n");
        code.push_str("// Features: data classes, coroutines, null-safety\n\n");
        
        // 导入语句
        code.push_str("package chim.generated\n\n");
        
        // 生成函数
        for func in &module.functions {
            code.push_str(&self.generate_function(func));
            code.push_str("\n");
        }
        
        // 生成 main 函数包装（如果有 main 函数）
        if module.functions.iter().any(|f| f.name == "main") {
            code.push_str("// Entry point\n");
            code.push_str("fun main(args: Array<String>) {\n");
            code.push_str("    main()\n");
            code.push_str("}\n");
        }
        
        Ok(code)
    }

    fn file_extension(&self) -> &str {
        "kt"
    }
}
