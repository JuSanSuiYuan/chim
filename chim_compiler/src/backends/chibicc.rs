use crate::backend::CodegenBackend;
use crate::ir::{Module, Function, Instruction, Type};
use std::error::Error;

/// chibicc 后端代码生成器
/// 
/// chibicc 是 Rui Ueyama 开发的小型 C 编译器（8cc 的后继）
/// 特性：
/// - C11 标准支持
/// - 清晰的代码结构
/// - 适合学习编译器原理
/// - 支持更多现代C特性
pub struct ChibiccBackend {
    /// 使用 C11 标准
    pub use_c11: bool,
}

impl ChibiccBackend {
    pub fn new() -> Self {
        Self {
            use_c11: true,
        }
    }

    fn generate_type(&self, ty: &Type) -> String {
        match ty {
            Type::Void => "void".to_string(),
            Type::Int32 => "int".to_string(),
            Type::Int64 => "long".to_string(),
            Type::Float32 => "float".to_string(),
            Type::Float64 => "double".to_string(),
            Type::Bool => "_Bool".to_string(),  // C11 _Bool
            Type::String => "char*".to_string(),
            Type::Ptr(inner) | Type::Ref(inner) | Type::MutRef(inner) => {
                format!("{}*", self.generate_type(inner))
            },
            Type::Array(inner, size) => {
                if *size == 0 {
                    format!("{}*", self.generate_type(inner))
                } else {
                    format!("{}[{}]", self.generate_type(inner), size)
                }
            },
            Type::Struct(name) => format!("struct {}", name),
        }
    }

    fn generate_function(&self, func: &Function) -> String {
        let mut code = String::new();
        
        let return_type = self.generate_type(&func.return_type);
        let params = func.params
            .iter()
            .map(|(name, ty)| format!("{} {}", self.generate_type(ty), name))
            .collect::<Vec<_>>()
            .join(", ");
        
        let params_str = if params.is_empty() {
            "void".to_string()
        } else {
            params
        };
        
        code.push_str(&format!("{} {}({}) {{\n", return_type, func.name, params_str));
        
        for inst in &func.body {
            code.push_str(&self.generate_instruction(inst));
        }
        
        code.push_str("}\n");
        code
    }

    fn generate_instruction(&self, inst: &Instruction) -> String {
        match inst {
            Instruction::Alloca { dest, ty } => {
                format!("    {} {};\n", self.generate_type(ty), dest)
            },
            Instruction::Store { dest, src } => {
                format!("    {} = {};\n", dest, src)
            },
            Instruction::Load { dest, src } => {
                format!("    int {} = {};\n", dest, src)
            },
            Instruction::Add { dest, left, right } => {
                format!("    int {} = {} + {};\n", dest, left, right)
            },
            Instruction::Sub { dest, left, right } => {
                format!("    int {} = {} - {};\n", dest, left, right)
            },
            Instruction::Mul { dest, left, right } => {
                format!("    int {} = {} * {};\n", dest, left, right)
            },
            Instruction::Div { dest, left, right } => {
                format!("    int {} = {} / {};\n", dest, left, right)
            },
            Instruction::Mod { dest, left, right } => {
                format!("    int {} = {} % {};\n", dest, left, right)
            },
            Instruction::Eq { dest, left, right } => {
                format!("    _Bool {} = {} == {};\n", dest, left, right)
            },
            Instruction::Ne { dest, left, right } => {
                format!("    _Bool {} = {} != {};\n", dest, left, right)
            },
            Instruction::Lt { dest, left, right } => {
                format!("    _Bool {} = {} < {};\n", dest, left, right)
            },
            Instruction::Le { dest, left, right } => {
                format!("    _Bool {} = {} <= {};\n", dest, left, right)
            },
            Instruction::Gt { dest, left, right } => {
                format!("    _Bool {} = {} > {};\n", dest, left, right)
            },
            Instruction::Ge { dest, left, right } => {
                format!("    _Bool {} = {} >= {};\n", dest, left, right)
            },
            Instruction::And { dest, left, right } => {
                format!("    _Bool {} = {} && {};\n", dest, left, right)
            },
            Instruction::Or { dest, left, right } => {
                format!("    _Bool {} = {} || {};\n", dest, left, right)
            },
            Instruction::Not { dest, src } => {
                format!("    _Bool {} = !{};\n", dest, src)
            },
            Instruction::Call { dest, func, args } => {
                if let Some(dest) = dest {
                    format!("    int {} = {}({});\n", dest, func, args.join(", "))
                } else {
                    format!("    {}({});\n", func, args.join(", "))
                }
            },
            Instruction::Return(value) => {
                if let Some(val) = value {
                    format!("    return {};\n", val)
                } else {
                    "    return;\n".to_string()
                }
            },
            Instruction::ReturnInPlace(value) => {
                format!("    return {}; /* RVO */\n", value)
            },
            Instruction::Label(name) => {
                format!("{}:\n", name)
            },
            Instruction::Br(target) => {
                format!("    goto {};\n", target)
            },
            Instruction::CondBr { cond, true_bb, false_bb } => {
                format!("    if ({}) goto {}; else goto {};\n", cond, true_bb, false_bb)
            },
            _ => format!("    /* {:?} */\n", inst),
        }
    }
}

impl CodegenBackend for ChibiccBackend {
    fn name(&self) -> &str {
        "chibicc"
    }

    fn generate(&self, module: &Module) -> Result<String, Box<dyn Error>> {
        let mut code = String::new();
        
        code.push_str("/* Generated by Chim Compiler - chibicc Backend */\n");
        code.push_str("/* Target: chibicc (Rui Ueyama's Small C Compiler) */\n");
        code.push_str("/* C11 Standard */\n\n");
        
        code.push_str("#include <stdio.h>\n");
        code.push_str("#include <stdlib.h>\n");
        code.push_str("#include <stdbool.h>\n\n");
        
        // 生成函数声明
        for func in &module.functions {
            let return_type = self.generate_type(&func.return_type);
            let params = func.params
                .iter()
                .map(|(name, ty)| format!("{} {}", self.generate_type(ty), name))
                .collect::<Vec<_>>()
                .join(", ");
            let params_str = if params.is_empty() {
                "void".to_string()
            } else {
                params
            };
            code.push_str(&format!("{} {}({});\n", return_type, func.name, params_str));
        }
        code.push_str("\n");
        
        // 生成函数定义
        for func in &module.functions {
            code.push_str(&self.generate_function(func));
            code.push_str("\n");
        }
        
        Ok(code)
    }

    fn file_extension(&self) -> &str {
        "c"
    }
}
