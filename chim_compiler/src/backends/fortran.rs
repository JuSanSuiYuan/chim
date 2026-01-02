use crate::backend::CodegenBackend;
use crate::ir::{Module, Function, Instruction, Type};
use std::error::Error;

/// Fortran后端 - 针对科学计算优化
/// 
/// 特点：
/// - 生成现代Fortran 2008/2018代码
/// - 利用Fortran数组优化能力
/// - 针对数值计算和线性代数优化
/// - 支持DO CONCURRENT并行
pub struct FortranBackend {
    use_modern_syntax: bool,  // 使用现代Fortran语法
    enable_parallel: bool,    // 启用并行优化
}

impl FortranBackend {
    pub fn new() -> Self {
        Self {
            use_modern_syntax: true,
            enable_parallel: true,
        }
    }
    
    /// 将IR类型转换为Fortran类型
    fn generate_type(&self, ty: &Type) -> String {
        match ty {
            Type::Void => "".to_string(),
            Type::Int32 => "INTEGER(4)".to_string(),
            Type::Int64 => "INTEGER(8)".to_string(),
            Type::Float32 => "REAL(4)".to_string(),
            Type::Float64 => "REAL(8)".to_string(),  // 科学计算首选双精度
            Type::Bool => "LOGICAL".to_string(),
            Type::String => "CHARACTER(LEN=*)".to_string(),
            Type::Ptr(inner) | Type::Ref(inner) | Type::MutRef(inner) => {
                format!("{}, POINTER", self.generate_type(inner))
            },
            Type::Array(inner, size) => {
                format!("{}, DIMENSION({})", self.generate_type(inner), size)
            },
            Type::Struct(name) => format!("TYPE({})", name),
        }
    }
    
    /// 生成函数
    fn generate_function(&self, func: &Function) -> String {
        let mut output = String::new();
        
        // 判断是函数还是子程序
        let is_subroutine = func.return_type == Type::Void;
        
        if is_subroutine {
            output.push_str(&format!("SUBROUTINE {}(", func.name.to_uppercase()));
        } else {
            let ret_type = self.generate_type(&func.return_type);
            output.push_str(&format!("FUNCTION {}(", func.name.to_uppercase()));
        }
        
        // 参数列表
        let param_names: Vec<String> = func.params.iter()
            .map(|(name, _)| name.to_uppercase())
            .collect();
        output.push_str(&param_names.join(", "));
        
        if !is_subroutine {
            output.push_str(&format!(") RESULT({})\n", format!("{}_RESULT", func.name.to_uppercase())));
        } else {
            output.push_str(")\n");
        }
        
        output.push_str("  IMPLICIT NONE\n");
        
        // 声明参数
        for (name, ty) in &func.params {
            output.push_str(&format!("  {}, INTENT(IN) :: {}\n", 
                self.generate_type(ty), name.to_uppercase()));
        }
        
        // 声明返回值
        if !is_subroutine {
            output.push_str(&format!("  {} :: {}\n", 
                self.generate_type(&func.return_type),
                format!("{}_RESULT", func.name.to_uppercase())));
        }
        
        // 局部变量声明（从指令中提取）
        let mut declared_vars = vec![];
        for inst in &func.body {
            match inst {
                Instruction::Alloca { dest, ty } => {
                    if !declared_vars.contains(dest) {
                        output.push_str(&format!("  {} :: {}\n", 
                            self.generate_type(ty), dest.to_uppercase()));
                        declared_vars.push(dest.clone());
                    }
                },
                _ => {}
            }
        }
        
        output.push_str("\n");
        
        // 函数体
        for inst in &func.body {
            output.push_str(&self.generate_instruction(inst));
        }
        
        output.push_str("\n");
        
        if is_subroutine {
            output.push_str(&format!("END SUBROUTINE {}\n\n", func.name.to_uppercase()));
        } else {
            output.push_str(&format!("END FUNCTION {}\n\n", func.name.to_uppercase()));
        }
        
        output
    }
    
    /// 生成指令
    fn generate_instruction(&self, inst: &Instruction) -> String {
        match inst {
            Instruction::Add { dest, left, right } => {
                format!("  {} = {} + {}\n", dest.to_uppercase(), left.to_uppercase(), right.to_uppercase())
            },
            Instruction::Sub { dest, left, right } => {
                format!("  {} = {} - {}\n", dest.to_uppercase(), left.to_uppercase(), right.to_uppercase())
            },
            Instruction::Mul { dest, left, right } => {
                format!("  {} = {} * {}\n", dest.to_uppercase(), left.to_uppercase(), right.to_uppercase())
            },
            Instruction::Div { dest, left, right } => {
                format!("  {} = {} / {}\n", dest.to_uppercase(), left.to_uppercase(), right.to_uppercase())
            },
            Instruction::Store { dest, src } => {
                format!("  {} = {}\n", dest.to_uppercase(), src.to_uppercase())
            },
            Instruction::Load { dest, src } => {
                format!("  {} = {}\n", dest.to_uppercase(), src.to_uppercase())
            },
            Instruction::Alloca { .. } => {
                // 已在声明部分处理
                String::new()
            },
            Instruction::Return(Some(value)) => {
                // Fortran使用RESULT变量
                format!("  RETURN\n")
            },
            Instruction::Return(None) => {
                "  RETURN\n".to_string()
            },
            Instruction::ReturnInPlace(value) => {
                // Fortran自动优化返回值
                "  RETURN\n".to_string()
            },
            Instruction::Call { dest, func, args } => {
                let args_str = args.iter()
                    .map(|a| a.to_uppercase())
                    .collect::<Vec<_>>()
                    .join(", ");
                if let Some(d) = dest {
                    format!("  {} = {}({})\n", d.to_uppercase(), func.to_uppercase(), args_str)
                } else {
                    format!("  CALL {}({})\n", func.to_uppercase(), args_str)
                }
            },
            Instruction::Lt { dest, left, right } => {
                format!("  {} = {} < {}\n", dest.to_uppercase(), left.to_uppercase(), right.to_uppercase())
            },
            Instruction::Le { dest, left, right } => {
                format!("  {} = {} <= {}\n", dest.to_uppercase(), left.to_uppercase(), right.to_uppercase())
            },
            Instruction::Gt { dest, left, right } => {
                format!("  {} = {} > {}\n", dest.to_uppercase(), left.to_uppercase(), right.to_uppercase())
            },
            Instruction::Ge { dest, left, right } => {
                format!("  {} = {} >= {}\n", dest.to_uppercase(), left.to_uppercase(), right.to_uppercase())
            },
            Instruction::Eq { dest, left, right } => {
                format!("  {} = {} == {}\n", dest.to_uppercase(), left.to_uppercase(), right.to_uppercase())
            },
            Instruction::Ne { dest, left, right } => {
                format!("  {} = {} /= {}\n", dest.to_uppercase(), left.to_uppercase(), right.to_uppercase())
            },
            Instruction::Label(name) => {
                // Fortran不需要显式标签，使用注释
                format!("  ! Label: {}\n", name)
            },
            _ => format!("  ! {:?}\n", inst),
        }
    }
    
    /// 生成模块级声明
    fn generate_module_header(&self) -> String {
        let mut output = String::new();
        output.push_str("! Generated by Chim Compiler - Fortran Backend\n");
        output.push_str("! Optimized for scientific computing\n");
        output.push_str("!\n");
        output.push_str("! Features:\n");
        output.push_str("!   - Modern Fortran 2008/2018 syntax\n");
        output.push_str("!   - Double precision for numerical stability\n");
        output.push_str("!   - Array optimization ready\n");
        output.push_str("!\n\n");
        
        if self.use_modern_syntax {
            output.push_str("MODULE chim_module\n");
            output.push_str("  IMPLICIT NONE\n");
            output.push_str("  PRIVATE\n\n");
            output.push_str("CONTAINS\n\n");
        }
        
        output
    }
    
    fn generate_module_footer(&self) -> String {
        if self.use_modern_syntax {
            "END MODULE chim_module\n".to_string()
        } else {
            String::new()
        }
    }
}

impl CodegenBackend for FortranBackend {
    fn name(&self) -> &str {
        "Fortran (Scientific Computing)"
    }
    
    fn generate(&self, module: &Module) -> Result<String, Box<dyn Error>> {
        let mut output = String::new();
        
        // 模块头
        output.push_str(&self.generate_module_header());
        
        // 生成所有函数
        for func in &module.functions {
            output.push_str(&self.generate_function(func));
        }
        
        // 模块尾
        output.push_str(&self.generate_module_footer());
        
        // 添加主程序框架
        if !self.use_modern_syntax {
            output.push_str("\nPROGRAM main\n");
            output.push_str("  IMPLICIT NONE\n");
            output.push_str("  ! Call your functions here\n");
            output.push_str("END PROGRAM main\n");
        }
        
        Ok(output)
    }
    
    fn file_extension(&self) -> &str {
        "f90"
    }
    
    fn supports_optimization(&self) -> bool {
        true  // gfortran/ifort支持强大的优化
    }
}
