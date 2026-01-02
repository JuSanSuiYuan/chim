use crate::ir::Module;
use std::error::Error;

/// 统一的代码生成后端接口
pub trait CodegenBackend {
    /// 后端名称
    fn name(&self) -> &str;
    
    /// 生成代码
    fn generate(&self, module: &Module) -> Result<String, Box<dyn Error>>;
    
    /// 获取输出文件扩展名
    fn file_extension(&self) -> &str;
    
    /// 是否支持优化级别
    fn supports_optimization(&self) -> bool {
        true
    }
}

/// 后端类型枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackendType {
    WASM,      // WebAssembly
    Native,    // C语言（原有的）
    LLVM,      // LLVM IR
    QBE,       // QBE中间语言
    TinyCC,    // TinyCC C代码
    Cranelift, // Cranelift IR
    Fortran,   // Fortran（科学计算优化）
    Asm,       // x86-64汇编
}

impl BackendType {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "wasm" => Some(Self::WASM),
            "native" | "c" => Some(Self::Native),
            "llvm" => Some(Self::LLVM),
            "qbe" => Some(Self::QBE),
            "tinycc" | "tcc" => Some(Self::TinyCC),
            "cranelift" | "clif" => Some(Self::Cranelift),
            "fortran" | "f90" | "f95" => Some(Self::Fortran),
            "asm" | "assembly" | "x86" | "x86-64" => Some(Self::Asm),
            _ => None,
        }
    }
    
    pub fn all() -> &'static [BackendType] {
        &[
            Self::WASM,
            Self::Native,
            Self::LLVM,
            Self::QBE,
            Self::TinyCC,
            Self::Cranelift,
            Self::Fortran,
            Self::Asm,
        ]
    }
}

/// 创建后端实例
pub fn create_backend(backend_type: BackendType) -> Box<dyn CodegenBackend> {
    match backend_type {
        BackendType::WASM => Box::new(crate::backends::wasm::WASMBackend::new()),
        BackendType::Native => Box::new(crate::backends::native::NativeBackend::new()),
        BackendType::LLVM => Box::new(crate::backends::llvm::LLVMBackend::new()),
        BackendType::QBE => Box::new(crate::backends::qbe::QBEBackend::new()),
        BackendType::TinyCC => Box::new(crate::backends::tinycc::TinyCCBackend::new()),
        BackendType::Cranelift => Box::new(crate::backends::cranelift::CraneliftBackend::new()),
        BackendType::Fortran => Box::new(crate::backends::fortran::FortranBackend::new()),
        BackendType::Asm => Box::new(crate::backends::asm::AsmBackend::new()),
    }
}
