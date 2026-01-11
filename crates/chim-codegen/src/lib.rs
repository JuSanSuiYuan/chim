pub mod native;
pub mod x86_64;
pub mod ir_backends;
pub mod transpilers;
pub mod modern_transpilers;
pub mod swift_ffi;
pub mod mojo_ffi;
pub mod moonbit_ffi;
pub mod dotnet10_ffi;
pub mod agda_ffi;
pub mod unison_ffi;
pub mod c_transpiler;

use chim_ir::{IRModule, IRFunction};
use chim_semantic::AnalyzedProgram;
use chim_span::Span;
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CodegenTarget {
    Native,
    Wasm,
    LLVM,
    Cranelift,
    QBE,
    C,
    Cpp,
    Rust,
    Go,
    Python,
    JavaScript,
    TypeScript,
    Java,
    CSharp,
    Kotlin,
    Swift,
    GoLang,
    Ruby,
    PHP,
    Rustc,
    GCC,
    Clang,
    LLVMIR,
    MLIR,
    GLSL,
    HLSL,
    CUDA,
    Metal,
    Vulkan,
    OpenCL,
    SPIRV,
    OpenGL,
    WebGPU,
    DotNet,
    D,
    Nim,
    V,
    Zig,
    Julia,
    R,
    MATLAB,
    Octave,
    Fortran,
    Ada,
    Pascal,
    OCaml,
    Haskell,
    Scala,
    Clojure,
    FSharp,
    Erlang,
    Elixir,
    Lua,
    Perl,
    Shell,
    PowerShell,
    Batch,
    Makefile,
    CMake,
    Meson,
    Ninja,
    Bazel,
    X86_64,
    AArch64,
    ARM32,
    RISC-V,
    WASM,
    BPF,
    SPARC,
    PowerPC,
    MIPS,
    Mojo,
    MoonBit,
}

impl CodegenTarget {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "native" | "asm" => Some(CodegenTarget::Native),
            "wasm" | "webassembly" => Some(CodegenTarget::Wasm),
            "llvm" => Some(CodegenTarget::LLVM),
            "cranelift" => Some(CodegenTarget::Cranelift),
            "qbe" => Some(CodegenTarget::QBE),
            "c" => Some(CodegenTarget::C),
            "cpp" | "c++" => Some(CodegenTarget::Cpp),
            "rust" | "rs" => Some(CodegenTarget::Rust),
            "go" | "golang" => Some(CodegenTarget::Go),
            "py" | "python" => Some(CodegenTarget::Python),
            "js" | "javascript" => Some(CodegenTarget::JavaScript),
            "ts" | "typescript" => Some(CodegenTarget::TypeScript),
            "java" => Some(CodegenTarget::Java),
            "cs" | "csharp" | "dotnet" => Some(CodegenTarget::CSharp),
            "kt" | "kotlin" => Some(CodegenTarget::Kotlin),
            "swift" => Some(CodegenTarget::Swift),
            "ruby" | "rb" => Some(CodegenTarget::Ruby),
            "php" => Some(CodegenTarget::PHP),
            "gcc" => Some(CodegenTarget::GCC),
            "clang" => Some(CodegenTarget::Clang),
            "llvm-ir" => Some(CodegenTarget::LLVMIR),
            "mlir" => Some(CodegenTarget::MLIR),
            "glsl" => Some(CodegenTarget::GLSL),
            "hlsl" => Some(CodegenTarget::HLSL),
            "cuda" | "cu" => Some(CodegenTarget::CUDA),
            "metal" | "mtl" => Some(CodegenTarget::Metal),
            "vulkan" | "glsl" => Some(CodegenTarget::Vulkan),
            "opencl" | "cl" => Some(CodegenTarget::OpenCL),
            "spirv" => Some(CodegenTarget::SPIRV),
            "opengl" => Some(CodegenTarget::OpenGL),
            "webgpu" => Some(CodegenTarget::WebGPU),
            "d" => Some(CodegenTarget::D),
            "nim" => Some(CodegenTarget::Nim),
            "v" => Some(CodegenTarget::V),
            "zig" => Some(CodegenTarget::Zig),
            "jl" | "julia" => Some(CodegenTarget::Julia),
            "r" => Some(CodegenTarget::R),
            "matlab" | "m" => Some(CodegenTarget::MATLAB),
            "octave" => Some(CodegenTarget::Octave),
            "fortran" | "f" | "f90" => Some(CodegenTarget::Fortran),
            "ada" => Some(CodegenTarget::Ada),
            "pascal" => Some(CodegenTarget::Pascal),
            "ocaml" => Some(CodegenTarget::OCaml),
            "haskell" | "hs" => Some(CodegenTarget::Haskell),
            "scala" => Some(CodegenTarget::Scala),
            "clojure" | "clj" => Some(CodegenTarget::Clojure),
            "fsharp" | "fs" => Some(CodegenTarget::FSharp),
            "erlang" | "erl" => Some(CodegenTarget::Erlang),
            "elixir" | "ex" => Some(CodegenTarget::Elixir),
            "lua" => Some(CodegenTarget::Lua),
            "perl" | "pl" => Some(CodegenTarget::Perl),
            "shell" | "sh" => Some(CodegenTarget::Shell),
            "powershell" | "ps1" => Some(CodegenTarget::PowerShell),
            "batch" | "bat" => Some(CodegenTarget::Batch),
            "makefile" => Some(CodegenTarget::Makefile),
            "cmake" => Some(CodegenTarget::CMake),
            "meson" => Some(CodegenTarget::Meson),
            "ninja" => Some(CodegenTarget::Ninja),
            "bazel" => Some(CodegenTarget::Bazel),
            "x86_64" | "x64" => Some(CodegenTarget::X86_64),
            "aarch64" | "arm64" => Some(CodegenTarget::AArch64),
            "arm32" | "arm" => Some(CodegenTarget::ARM32),
            "riscv" | "rv64gc" => Some(CodegenTarget::RISC-V),
            "wasm" => Some(CodegenTarget::WASM),
            "bpf" => Some(CodegenTarget::BPF),
            "sparc" => Some(CodegenTarget::SPARC),
            "powerpc" | "ppc64" => Some(CodegenTarget::PowerPC),
            "mips" => Some(CodegenTarget::MIPS),
            "mojo" => Some(CodegenTarget::Mojo),
            "moonbit" | "mbt" => Some(CodegenTarget::MoonBit),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GeneratedCode {
    pub source: String,
    pub extension: String,
    pub language: String,
    pub is_executable: bool,
}

pub trait CodeGenerator {
    fn generate(&self, module: &IRModule, program: &AnalyzedProgram) -> Result<GeneratedCode, CodegenError>;
    fn name(&self) -> &str;
    fn file_extension(&self) -> &str;
    fn target(&self) -> CodegenTarget;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CodegenError {
    pub message: String,
    pub span: Option<Span>,
    pub suggestion: Option<String>,
}

impl std::fmt::Display for CodegenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "code generation error: {}", self.message)
    }
}

impl std::error::Error for CodegenError {}

pub struct CodeGen {
    generators: Vec<Box<dyn CodeGenerator>>,
}

impl CodeGen {
    pub fn new() -> Self {
        let mut generators: Vec<Box<dyn CodeGenerator>> = Vec::new();

        generators.push(Box::new(native::NativeCodeGenerator::new()));
        generators.push(Box::new(ir_backends::LLVMCodeGenerator::new()));
        generators.push(Box::new(ir_backends::CraneliftCodeGenerator::new()));
        generators.push(Box::new(ir_backends::QBECodeGenerator::new()));
        generators.push(Box::new(c_transpiler::CTranspiler::new()));
        generators.push(Box::new(transpilers::CppTranspiler::new()));
        generators.push(Box::new(transpilers::RustTranspiler::new()));
        generators.push(Box::new(transpilers::GoTranspiler::new()));
        generators.push(Box::new(transpilers::PythonTranspiler::new()));
        generators.push(Box::new(transpilers::JavaScriptTranspiler::new()));
        generators.push(Box::new(transpilers::TypeScriptTranspiler::new()));
        generators.push(Box::new(transpilers::JavaTranspiler::new()));
        generators.push(Box::new(transpilers::CSharpTranspiler::new()));
        generators.push(Box::new(transpilers::WasmTranspiler::new()));
        generators.push(Box::new(modern_transpilers::SwiftTranspiler::new()));
        generators.push(Box::new(modern_transpilers::KotlinTranspiler::new()));
        generators.push(Box::new(modern_transpilers::MojoTranspiler::new()));
        generators.push(Box::new(modern_transpilers::MoonBitTranspiler::new()));

        CodeGen { generators }
    }

    pub fn generate(&self, module: &IRModule, program: &AnalyzedProgram, target: CodegenTarget) -> Result<GeneratedCode, CodegenError> {
        for generator in &self.generators {
            if generator.target() == target {
                return generator.generate(module, program);
            }
        }
        Err(CodegenError {
            message: format!("no code generator for target {:?}", target),
            span: None,
            suggestion: None,
        })
    }

    pub fn generate_with_name(&self, module: &IRModule, program: &AnalyzedProgram, target_name: &str) -> Result<GeneratedCode, CodegenError> {
        if let Some(target) = CodegenTarget::from_str(target_name) {
            self.generate(module, program, target)
        } else {
            Err(CodegenError {
                message: format!("unknown target: {}", target_name),
                span: None,
                suggestion: Some(format!("Supported targets: {:?}", self.supported_targets())),
            })
        }
    }

    pub fn supported_targets(&self) -> Vec<CodegenTarget> {
        self.generators.iter().map(|g| g.target()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_target_from_str() {
        assert_eq!(CodegenTarget::from_str("c"), Some(CodegenTarget::C));
        assert_eq!(CodegenTarget::from_str("cpp"), Some(CodegenTarget::Cpp));
        assert_eq!(CodegenTarget::from_str("rust"), Some(CodegenTarget::Rust));
        assert_eq!(CodegenTarget::from_str("unknown"), None);
    }

    #[test]
    fn test_code_gen_creation() {
        let codegen = CodeGen::new();
        assert!(!codegen.supported_targets().is_empty());
    }
}
