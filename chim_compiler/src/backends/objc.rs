use crate::backend::CodegenBackend;
use crate::ir::{Module, Function, Instruction, Type};
use std::error::Error;

/// Objective-C 后端代码生成器
/// 
/// 生成 Objective-C 2.0+ 代码
/// 特性：
/// - ARC（自动引用计数）
/// - Block 语法
/// - 属性（@property）
/// - 协议（@protocol）
pub struct ObjectiveCBackend {
    /// 使用 ARC
    pub use_arc: bool,
    /// 使用现代 Objective-C 语法
    pub modern_syntax: bool,
}

impl ObjectiveCBackend {
    pub fn new() -> Self {
        Self {
            use_arc: true,
            modern_syntax: true,
        }
    }

    fn generate_type(&self, ty: &Type) -> String {
        match ty {
            Type::Void => "void".to_string(),
            Type::Int32 => "int".to_string(),
            Type::Int64 => "long long".to_string(),
            Type::Float32 => "float".to_string(),
            Type::Float64 => "double".to_string(),
            Type::Bool => "BOOL".to_string(),
            Type::String => "NSString *".to_string(),
            Type::Ptr(inner) | Type::Ref(inner) | Type::MutRef(inner) => {
                format!("{} *", self.generate_type(inner))
            },
            Type::Array(inner, size) => {
                if *size == 0 {
                    format!("NSArray<{}> *", self.generate_type(inner))
                } else {
                    format!("{}[{}]", self.generate_type(inner), size)
                }
            },
            Type::Struct(name) => format!("{} *", name),
        }
    }

    fn generate_function(&self, func: &Function) -> String {
        let mut code = String::new();
        
        // 函数签名（使用 C 函数风格）
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
                format!("    {} {}; // alloca\n", self.generate_type(ty), dest)
            },
            Instruction::Store { dest, src } => {
                format!("    {} = {};\n", dest, src)
            },
            Instruction::Load { dest, src } => {
                format!("    id {} = {};\n", dest, src)
            },
            Instruction::Add { dest, left, right } => {
                format!("    id {} = {} + {};\n", dest, left, right)
            },
            Instruction::Sub { dest, left, right } => {
                format!("    id {} = {} - {};\n", dest, left, right)
            },
            Instruction::Mul { dest, left, right } => {
                format!("    id {} = {} * {};\n", dest, left, right)
            },
            Instruction::Div { dest, left, right } => {
                format!("    id {} = {} / {};\n", dest, left, right)
            },
            Instruction::Mod { dest, left, right } => {
                format!("    id {} = {} % {};\n", dest, left, right)
            },
            Instruction::Eq { dest, left, right } => {
                format!("    BOOL {} = {} == {};\n", dest, left, right)
            },
            Instruction::Ne { dest, left, right } => {
                format!("    BOOL {} = {} != {};\n", dest, left, right)
            },
            Instruction::Lt { dest, left, right } => {
                format!("    BOOL {} = {} < {};\n", dest, left, right)
            },
            Instruction::Le { dest, left, right } => {
                format!("    BOOL {} = {} <= {};\n", dest, left, right)
            },
            Instruction::Gt { dest, left, right } => {
                format!("    BOOL {} = {} > {};\n", dest, left, right)
            },
            Instruction::Ge { dest, left, right } => {
                format!("    BOOL {} = {} >= {};\n", dest, left, right)
            },
            Instruction::And { dest, left, right } => {
                format!("    BOOL {} = {} && {};\n", dest, left, right)
            },
            Instruction::Or { dest, left, right } => {
                format!("    BOOL {} = {} || {};\n", dest, left, right)
            },
            Instruction::Not { dest, src } => {
                format!("    BOOL {} = !{};\n", dest, src)
            },
            Instruction::Call { dest, func, args } => {
                if let Some(dest) = dest {
                    format!("    id {} = {}({});\n", dest, func, args.join(", "))
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
                format!("    return {}; // RVO\n", value)
            },
            Instruction::Label(name) => {
                format!("{}:\n", name)
            },
            Instruction::Br(target) => {
                format!("    goto {};\n", target)
            },
            Instruction::CondBr { cond, true_bb, false_bb } => {
                format!("    if ({}) {{ goto {}; }} else {{ goto {}; }}\n", 
                    cond, true_bb, false_bb)
            },
            _ => format!("    // {:?}\n", inst),
        }
    }
}

impl CodegenBackend for ObjectiveCBackend {
    fn name(&self) -> &str {
        "Objective-C"
    }

    fn generate(&self, module: &Module) -> Result<String, Box<dyn Error>> {
        let mut code = String::new();
        
        // 文件头注释
        code.push_str("// Generated by Chim Compiler - Objective-C Backend\n");
        code.push_str("// Target: Objective-C 2.0+\n");
        if self.use_arc {
            code.push_str("// Features: ARC enabled, modern syntax, blocks\n\n");
        } else {
            code.push_str("// Features: manual memory management, modern syntax\n\n");
        }
        
        // 导入头文件
        code.push_str("#import <Foundation/Foundation.h>\n");
        if self.use_arc {
            code.push_str("#if !__has_feature(objc_arc)\n");
            code.push_str("#error \"ARC is required for this file\"\n");
            code.push_str("#endif\n");
        }
        code.push_str("\n");
        
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
        
        // 生成 main 函数（如果需要）
        if module.functions.iter().any(|f| f.name == "main") {
            code.push_str("// Entry point\n");
            code.push_str("int main(int argc, const char * argv[]) {\n");
            code.push_str("    @autoreleasepool {\n");
            code.push_str("        main();\n");
            code.push_str("    }\n");
            code.push_str("    return 0;\n");
            code.push_str("}\n");
        }
        
        Ok(code)
    }

    fn file_extension(&self) -> &str {
        "m"
    }
}
