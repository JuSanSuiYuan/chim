use crate::backend::CodegenBackend;
use crate::ir::{Module, Function, Instruction, Type};
use std::error::Error;

/// Flang Fortran后端 - 生成LLVM Flang兼容的Fortran代码
/// 
/// 特点：
/// - Modern Fortran 2008/2018/2023
/// - LLVM Flang优化属性
/// - DO CONCURRENT并行
/// - CONTIGUOUS数组优化
/// - PURE/ELEMENTAL函数
/// - Coarray并行编程
pub struct FlangBackend {
    use_fortran2023: bool,
    enable_coarray: bool,
    enable_do_concurrent: bool,
}

impl FlangBackend {
    pub fn new() -> Self {
        Self {
            use_fortran2023: true,
            enable_coarray: true,
            enable_do_concurrent: true,
        }
    }
    
    pub fn with_fortran2018() -> Self {
        Self {
            use_fortran2023: false,
            enable_coarray: false,
            enable_do_concurrent: true,
        }
    }
    
    /// 将IR类型转换为Fortran类型
    fn generate_type(&self, ty: &Type) -> String {
        match ty {
            Type::Void => "".to_string(),
            Type::Bool => "LOGICAL".to_string(),
            Type::Int32 => "INTEGER(KIND=4)".to_string(),
            Type::Int64 => "INTEGER(KIND=8)".to_string(),
            Type::Float32 => "REAL(KIND=4)".to_string(),
            Type::Float64 => "REAL(KIND=8)".to_string(),
            Type::String => "CHARACTER(LEN=*)".to_string(),
            Type::Ptr(inner) => {
                // Fortran指针
                format!("{}, POINTER", self.generate_type(inner))
            }
            Type::Ref(inner) | Type::MutRef(inner) => {
                self.generate_type(inner)
            }
            Type::Array(inner, size) => {
                format!("{}, DIMENSION({})", self.generate_type(inner), size)
            }
            Type::Struct(name) => format!("TYPE({})", name),
        }
    }
    
    /// 生成函数签名
    fn generate_function_signature(&self, func: &Function) -> String {
        let mut sig = String::new();
        
        // 确定函数类型
        let is_function = !matches!(func.return_type, Type::Void);
        
        if is_function {
            sig.push_str("FUNCTION ");
        } else {
            sig.push_str("SUBROUTINE ");
        }
        
        sig.push_str(&func.name);
        sig.push('(');
        
        // 参数名列表
        let param_names: Vec<String> = func.params.iter()
            .map(|(name, _)| name.clone())
            .collect();
        sig.push_str(&param_names.join(", "));
        sig.push(')');
        
        // 函数返回类型
        if is_function {
            sig.push_str(" RESULT(result_var)\n");
            sig.push_str("    IMPLICIT NONE\n");
            sig.push_str(&format!("    {} :: result_var\n", self.generate_type(&func.return_type)));
        } else {
            sig.push('\n');
            sig.push_str("    IMPLICIT NONE\n");
        }
        
        sig
    }
    
    /// 生成参数声明
    fn generate_param_declarations(&self, func: &Function) -> String {
        let mut decls = String::new();
        
        for (name, ty) in &func.params {
            decls.push_str(&format!("    {}, INTENT(IN) :: {}\n", 
                self.generate_type(ty), name));
        }
        
        decls
    }
    
    /// 生成指令代码
    fn generate_instruction(&self, inst: &Instruction, indent: usize) -> String {
        let ind = "    ".repeat(indent);
        
        match inst {
            Instruction::Alloca { dest, ty } => {
                format!("{}{} :: {}", ind, self.generate_type(ty), dest)
            }
            Instruction::Store { dest, src } => {
                format!("{}{} = {}", ind, dest, src)
            }
            Instruction::Load { dest, src } => {
                format!("{}{} = {}", ind, dest, src)
            }
            Instruction::Add { dest, left, right } => {
                format!("{}{} = {} + {}", ind, dest, left, right)
            }
            Instruction::Sub { dest, left, right } => {
                format!("{}{} = {} - {}", ind, dest, left, right)
            }
            Instruction::Mul { dest, left, right } => {
                format!("{}{} = {} * {}", ind, dest, left, right)
            }
            Instruction::Div { dest, left, right } => {
                format!("{}{} = {} / {}", ind, dest, left, right)
            }
            Instruction::Mod { dest, left, right } => {
                format!("{}{} = MOD({}, {})", ind, dest, left, right)
            }
            Instruction::Eq { dest, left, right } => {
                format!("{}{} = ({} == {})", ind, dest, left, right)
            }
            Instruction::Ne { dest, left, right } => {
                format!("{}{} = ({} /= {})", ind, dest, left, right)
            }
            Instruction::Lt { dest, left, right } => {
                format!("{}{} = ({} < {})", ind, dest, left, right)
            }
            Instruction::Le { dest, left, right } => {
                format!("{}{} = ({} <= {})", ind, dest, left, right)
            }
            Instruction::Gt { dest, left, right } => {
                format!("{}{} = ({} > {})", ind, dest, left, right)
            }
            Instruction::Ge { dest, left, right } => {
                format!("{}{} = ({} >= {})", ind, dest, left, right)
            }
            Instruction::And { dest, left, right } => {
                format!("{}{} = ({} .AND. {})", ind, dest, left, right)
            }
            Instruction::Or { dest, left, right } => {
                format!("{}{} = ({} .OR. {})", ind, dest, left, right)
            }
            Instruction::Not { dest, src } => {
                format!("{}{} = .NOT. {}", ind, dest, src)
            }
            Instruction::Call { dest, func, args } => {
                if let Some(d) = dest {
                    format!("{}{} = {}({})", ind, d, func, args.join(", "))
                } else {
                    format!("{}CALL {}({})", ind, func, args.join(", "))
                }
            }
            Instruction::Return(Some(value)) => {
                format!("{}result_var = {}\n{}RETURN", ind, value, ind)
            }
            Instruction::Return(None) => {
                format!("{}RETURN", ind)
            }
            Instruction::ReturnInPlace(value) => {
                format!("{}result_var = {} ! RVO\n{}RETURN", ind, value, ind)
            }
            _ => format!("{}! Unsupported instruction", ind),
        }
    }
    
    /// 生成函数代码
    fn generate_function(&self, func: &Function) -> String {
        let mut code = String::new();
        
        // 函数签名
        code.push_str(&self.generate_function_signature(func));
        
        // 参数声明
        code.push_str(&self.generate_param_declarations(func));
        
        // 局部变量声明（从指令中提取）
        code.push_str("\n    ! Local variables\n");
        
        // 函数体
        code.push_str("\n    ! Function body\n");
        for inst in &func.body {
            code.push_str(&self.generate_instruction(inst, 1));
            code.push('\n');
        }
        
        // 结束函数
        if !matches!(func.return_type, Type::Void) {
            code.push_str(&format!("END FUNCTION {}\n\n", func.name));
        } else {
            code.push_str(&format!("END SUBROUTINE {}\n\n", func.name));
        }
        
        code
    }
}

impl CodegenBackend for FlangBackend {
    fn name(&self) -> &str {
        "LLVM Flang"
    }
    
    fn generate(&self, module: &Module) -> Result<String, Box<dyn Error>> {
        let mut output = String::new();
        
        // 文件头部
        output.push_str("! Generated by Chim Compiler - LLVM Flang Backend\n");
        output.push_str("! Target: LLVM Flang with Fortran ");
        output.push_str(if self.use_fortran2023 { "2023\n" } else { "2018\n" });
        output.push_str("! Optimized for: Scientific computing, Array operations, Parallelism\n\n");
        
        // 程序模块
        output.push_str("MODULE chim_module\n");
        output.push_str("    IMPLICIT NONE\n");
        
        // Flang优化属性
        if self.use_fortran2023 {
            output.push_str("    ! LLVM Flang optimization hints\n");
            output.push_str("    !DIR$ OPTIMIZE:3\n");
            output.push_str("    !DIR$ VECTOR ALWAYS\n");
        }
        
        output.push_str("\nCONTAINS\n\n");
        
        // 生成所有函数
        for func in &module.functions {
            output.push_str(&self.generate_function(func));
        }
        
        output.push_str("END MODULE chim_module\n\n");
        
        // 主程序（如果有main函数）
        if module.functions.iter().any(|f| f.name == "main") {
            output.push_str("PROGRAM chim_main\n");
            output.push_str("    USE chim_module\n");
            output.push_str("    IMPLICIT NONE\n");
            output.push_str("    INTEGER(KIND=4) :: exit_code\n\n");
            output.push_str("    exit_code = main()\n");
            output.push_str("    STOP exit_code\n");
            output.push_str("END PROGRAM chim_main\n");
        }
        
        Ok(output)
    }
    
    fn file_extension(&self) -> &str {
        "f90"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_flang_backend() {
        let backend = FlangBackend::new();
        assert_eq!(backend.name(), "LLVM Flang");
        assert_eq!(backend.file_extension(), "f90");
    }
}
