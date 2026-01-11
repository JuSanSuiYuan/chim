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
    Clang,     // Clang C++（LLVM优化）
    Flang,     // LLVM Flang Fortran
    Java,      // Java 11+
    JavaScript,// JavaScript ES6+
    TypeScript,// TypeScript
    CSharp,    // C# 9.0+
    V,         // V语言
    Nim,       // Nim语言
    Kotlin,    // Kotlin 1.8+
    Swift,     // Swift 5.0+
    ObjectiveC,// Objective-C 2.0+
    Cc8,       // 8cc (教育型)
    GCC,       // GNU Compiler Collection
    Rustc,     // Rust Compiler
    ZigCC,     // Zig C Compiler
    UCC,       // Universal C Compiler
    Selfie,    // 自托管教育编译器
    Cc9,       // 9cc
    PGI,       // NVIDIA HPC
    MSVC,      // Microsoft Visual C++
    CompCert,  // 经验证的C编译器
    LCC,       // 可重定向C编译器
    Chibicc,   // chibicc (小型C编译器)
    
    // GPU后端
    CUDA,      // NVIDIA CUDA
    Vulkan,    // Vulkan Compute
    Metal,     // Apple Metal
    OpenCL,    // OpenCL (跨平台)
    Mojo,      // Mojo (AI原生)
    TileLang,  // Chim TileLang (AI/ML优化)
    
    // 现代语言后端
    MoonBit,   // MoonBit
    Cone,      // Cone
    Pony,      // Pony
    FSharp,    // F#
    Gleam,     // Gleam
    Go,        // Go
    Python,    // Python
    Crystal,   // Crystal
    Reason,    // Reason/ReScript
    Julia,     // Julia
    R,         // R
    Ruby,      // Ruby
    D,         // D
    Delphi,    // Delphi/Pascal
    Cpp,       // C++
    Erlang,    // Erlang
    Matlab,    // MATLAB
    Php,       // PHP
    June,      // June
    Agda,      // Agda
    Unison,    // Unison
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
            "clang" | "cpp" | "c++" | "cxx" => Some(Self::Clang),
            "flang" | "fortran-llvm" => Some(Self::Flang),
            "java" => Some(Self::Java),
            "js" | "javascript" => Some(Self::JavaScript),
            "ts" | "typescript" => Some(Self::TypeScript),
            "cs" | "csharp" | "c#" => Some(Self::CSharp),
            "v" | "vlang" => Some(Self::V),
            "nim" => Some(Self::Nim),
            "kotlin" | "kt" => Some(Self::Kotlin),
            "swift" => Some(Self::Swift),
            "objc" | "objective-c" | "objectivec" => Some(Self::ObjectiveC),
            "8cc" => Some(Self::Cc8),
            "gcc" => Some(Self::GCC),
            "rustc" | "rust" => Some(Self::Rustc),
            "zig" | "zigcc" | "zig-cc" => Some(Self::ZigCC),
            "ucc" => Some(Self::UCC),
            "selfie" => Some(Self::Selfie),
            "9cc" => Some(Self::Cc9),
            "pgi" => Some(Self::PGI),
            "msvc" => Some(Self::MSVC),
            "compcert" => Some(Self::CompCert),
            "lcc" => Some(Self::LCC),
            "chibicc" => Some(Self::Chibicc),
            
            // GPU后端
            "cuda" => Some(Self::CUDA),
            "vulkan" | "vulkan-compute" | "comp" => Some(Self::Vulkan),
            "metal" => Some(Self::Metal),
            "opencl" | "cl" => Some(Self::OpenCL),
            "mojo" => Some(Self::Mojo),
            "tilelang" | "tile" | "国产" | "北大" | "deepseek" => Some(Self::TileLang),
            
            // 现代语言后端
            "moonbit" | "mbt" => Some(Self::MoonBit),
            "cone" => Some(Self::Cone),
            "pony" => Some(Self::Pony),
            "fsharp" | "f#" | "fs" => Some(Self::FSharp),
            "gleam" => Some(Self::Gleam),
            "go" | "golang" => Some(Self::Go),
            "python" | "py" => Some(Self::Python),
            "crystal" | "cr" => Some(Self::Crystal),
            "reason" | "reasonml" | "re" => Some(Self::Reason),
            "julia" | "jl" => Some(Self::Julia),
            "r" | "rlang" => Some(Self::R),
            "ruby" | "rb" => Some(Self::Ruby),
            "d" | "dlang" => Some(Self::D),
            "delphi" | "pascal" | "pas" => Some(Self::Delphi),
            "cpp" | "c++" | "cxx" => Some(Self::Cpp),
            "erlang" | "erl" => Some(Self::Erlang),
            "matlab" | "mat" | "m" => Some(Self::Matlab),
            "php" => Some(Self::Php),
            "june" => Some(Self::June),
            "agda" => Some(Self::Agda),
            "unison" | "u" => Some(Self::Unison),
            
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
            Self::Clang,
            Self::Flang,
            Self::Java,
            Self::JavaScript,
            Self::TypeScript,
            Self::CSharp,
            Self::V,
            Self::Nim,
            Self::Kotlin,
            Self::Swift,
            Self::ObjectiveC,
            Self::Cc8,
            Self::GCC,
            Self::Rustc,
            Self::ZigCC,
            Self::UCC,
            Self::Selfie,
            Self::Cc9,
            Self::PGI,
            Self::MSVC,
            Self::CompCert,
            Self::LCC,
            Self::Chibicc,
            
            // GPU后端
            Self::CUDA,
            Self::Vulkan,
            Self::Metal,
            Self::OpenCL,
            Self::Mojo,
            Self::TileLang,
            
            // 现代语言后端
            Self::MoonBit,
            Self::Cone,
            Self::Pony,
            Self::FSharp,
            Self::Gleam,
            Self::Go,
            Self::Python,
            Self::Crystal,
            Self::Reason,
            Self::Julia,
            Self::R,
            Self::Ruby,
            Self::D,
            Self::Delphi,
            Self::Cpp,
            Self::Erlang,
            Self::Matlab,
            Self::Php,
            Self::June,
            Self::Agda,
            Self::Unison,
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
        BackendType::Clang => Box::new(crate::backends::clang::ClangBackend::new()),
        BackendType::Flang => Box::new(crate::backends::flang::FlangBackend::new()),
        BackendType::Java => Box::new(crate::backends::java::JavaBackend::new()),
        BackendType::JavaScript => Box::new(crate::backends::javascript::JavaScriptBackend::new()),
        BackendType::TypeScript => Box::new(crate::backends::typescript::TypeScriptBackend::new()),
        BackendType::CSharp => Box::new(crate::backends::csharp::CSharpBackend::new()),
        BackendType::V => Box::new(crate::backends::v::VBackend::new()),
        BackendType::Nim => Box::new(crate::backends::nim::NimBackend::new()),
        BackendType::Kotlin => Box::new(crate::backends::kotlin::KotlinBackend::new()),
        BackendType::Swift => Box::new(crate::backends::swift::SwiftBackend::new()),
        BackendType::ObjectiveC => Box::new(crate::backends::objc::ObjectiveCBackend::new()),
        BackendType::Cc8 => Box::new(crate::backends::cc8::Cc8Backend::new()),
        BackendType::GCC => Box::new(crate::backends::gcc::GCCBackend::new()),
        BackendType::Rustc => Box::new(crate::backends::rustc::RustcBackend::new()),
        BackendType::ZigCC => Box::new(crate::backends::zigcc::ZigCCBackend::new()),
        BackendType::UCC => Box::new(crate::backends::ucc::UCCBackend::new()),
        BackendType::Selfie => Box::new(crate::backends::selfie::SelfieBackend::new()),
        BackendType::Cc9 => Box::new(crate::backends::cc9::Cc9Backend::new()),
        BackendType::PGI => Box::new(crate::backends::pgi::PGIBackend::new()),
        BackendType::MSVC => Box::new(crate::backends::msvc::MSVCBackend::new()),
        BackendType::CompCert => Box::new(crate::backends::compcert::CompCertBackend::new()),
        BackendType::LCC => Box::new(crate::backends::lcc::LCCBackend::new()),
        BackendType::Chibicc => Box::new(crate::backends::chibicc::ChibiccBackend::new()),
        
        // GPU后端
        BackendType::CUDA => Box::new(crate::backends::cuda::CUDABackend::new()),
        BackendType::Vulkan => Box::new(crate::backends::vulkan::VulkanBackend::new()),
        BackendType::Metal => Box::new(crate::backends::metal::MetalBackend::new()),
        BackendType::OpenCL => Box::new(crate::backends::opencl::OpenCLBackend::new()),
        BackendType::Mojo => Box::new(crate::backends::mojo::MojoBackend::new()),
        BackendType::TileLang => Box::new(crate::backends::tilelang::TileLangBackend::new()),
        
        // 现代语言后端
        BackendType::MoonBit => Box::new(crate::backends::moonbit::MoonBitBackend::new()),
        BackendType::Cone => Box::new(crate::backends::cone::ConeBackend::new()),
        BackendType::Pony => Box::new(crate::backends::pony::PonyBackend::new()),
        BackendType::FSharp => Box::new(crate::backends::fsharp::FSharpBackend::new()),
        BackendType::Gleam => Box::new(crate::backends::gleam::GleamBackend::new()),
        BackendType::Go => Box::new(crate::backends::go::GoBackend::new()),
        BackendType::Python => Box::new(crate::backends::python::PythonBackend::new()),
        BackendType::Crystal => Box::new(crate::backends::crystal::CrystalBackend::new()),
        BackendType::Reason => Box::new(crate::backends::reason::ReasonBackend::new()),
        BackendType::Julia => Box::new(crate::backends::julia::JuliaBackend::new()),
        BackendType::R => Box::new(crate::backends::r::RBackend::new()),
        BackendType::Ruby => Box::new(crate::backends::ruby::RubyBackend::new()),
        BackendType::D => Box::new(crate::backends::d::DBackend::new()),
        BackendType::Delphi => Box::new(crate::backends::delphi::DelphiBackend::new()),
        BackendType::Cpp => Box::new(crate::backends::cpp::CppBackend::new()),
        BackendType::Erlang => Box::new(crate::backends::erlang::ErlangBackend::new()),
        BackendType::Matlab => Box::new(crate::backends::matlab::MatlabBackend::new()),
        BackendType::Php => Box::new(crate::backends::php::PhpBackend::new()),
        BackendType::June => Box::new(crate::backends::june::JuneBackend::new()),
        BackendType::Agda => Box::new(crate::backends::agda::AgdaBackend::new()),
        BackendType::Unison => Box::new(crate::backends::unison::UnisonBackend::new()),
    }
}
