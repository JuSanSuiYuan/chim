use crate::backend::CodegenBackend;
use crate::ir::{Module, Function, Instruction, Type};
use std::error::Error;

pub struct LLVMBackend;

impl LLVMBackend {
    pub fn new() -> Self {
        Self
    }
    
    fn generate_type(&self, ty: &Type) -> String {
        match ty {
            Type::Void => "void".to_string(),
            Type::Int32 => "i32".to_string(),
            Type::Int64 => "i64".to_string(),
            Type::Float32 => "float".to_string(),
            Type::Float64 => "double".to_string(),
            Type::Bool => "i1".to_string(),
            Type::String => "i8*".to_string(),
            Type::Ptr(inner) => format!("{}*", self.generate_type(inner)),
            _ => "i8*".to_string(),
        }
    }
    
    fn generate_function(&self, func: &Function) -> String {
        let mut output = String::new();
        
        // 函数签名
        let ret_type = self.generate_type(&func.return_type);
        output.push_str(&format!("define {} @{}(", ret_type, func.name));
        
        // 参数
        let params: Vec<String> = func.params.iter()
            .map(|(name, ty)| format!("{} %{}", self.generate_type(ty), name))
            .collect();
        output.push_str(&params.join(", "));
        
        output.push_str(") {\n");
        output.push_str("entry:\n");
        
        // 函数体
        for inst in &func.body {
            output.push_str(&self.generate_instruction(inst));
        }
        
        // 如果没有return，添加默认return
        if !func.body.iter().any(|i| matches!(i, Instruction::Return(_) | Instruction::ReturnInPlace(_))) {
            if func.return_type == Type::Void {
                output.push_str("  ret void\n");
            }
        }
        
        output.push_str("}\n\n");
        output
    }
    
    fn generate_instruction(&self, inst: &Instruction) -> String {
        match inst {
            Instruction::Add { dest, left, right } => {
                format!("  %{} = add i32 %{}, %{}\n", dest, left, right)
            },
            Instruction::Sub { dest, left, right } => {
                format!("  %{} = sub i32 %{}, %{}\n", dest, left, right)
            },
            Instruction::Mul { dest, left, right } => {
                format!("  %{} = mul i32 %{}, %{}\n", dest, left, right)
            },
            Instruction::Div { dest, left, right } => {
                format!("  %{} = sdiv i32 %{}, %{}\n", dest, left, right)
            },
            Instruction::Store { dest, src } => {
                format!("  store i32 %{}, i32* %{}\n", src, dest)
            },
            Instruction::Load { dest, src } => {
                format!("  %{} = load i32, i32* %{}\n", dest, src)
            },
            Instruction::Alloca { dest, ty } => {
                format!("  %{} = alloca {}\n", dest, self.generate_type(ty))
            },
            Instruction::Return(Some(value)) => {
                format!("  ret i32 %{}\n", value)
            },
            Instruction::Return(None) => {
                "  ret void\n".to_string()
            },
            Instruction::ReturnInPlace(value) => {
                // RVO: LLVM会自动优化
                format!("  ret i32 %{} ; RVO\n", value)
            },
            Instruction::Call { dest, func, args } => {
                let args_str: Vec<String> = args.iter()
                    .map(|a| format!("i32 %{}", a))
                    .collect();
                if let Some(d) = dest {
                    format!("  %{} = call i32 @{}({})\n", d, func, args_str.join(", "))
                } else {
                    format!("  call void @{}({})\n", func, args_str.join(", "))
                }
            },
            Instruction::Br(label) => {
                format!("  br label %{}\n", label)
            },
            Instruction::CondBr { cond, true_bb, false_bb } => {
                format!("  br i1 %{}, label %{}, label %{}\n", cond, true_bb, false_bb)
            },
            Instruction::Label(name) => {
                format!("{}:\n", name)
            },
            _ => format!("  ; {:?}\n", inst),
        }
    }
}

impl CodegenBackend for LLVMBackend {
    fn name(&self) -> &str {
        "LLVM IR"
    }
    
    fn generate(&self, module: &Module) -> Result<String, Box<dyn Error>> {
        let mut output = String::new();
        
        // 模块声明
        output.push_str("; ModuleID = 'chim_module'\n");
        output.push_str("target triple = \"x86_64-pc-linux-gnu\"\n\n");
        
        // 外部函数声明
        output.push_str("declare void @println(i8*)\n");
        output.push_str("declare void @print(i8*)\n\n");
        
        // 生成函数
        for func in &module.functions {
            output.push_str(&self.generate_function(func));
        }
        
        Ok(output)
    }
    
    fn file_extension(&self) -> &str {
        "ll"
    }
}
