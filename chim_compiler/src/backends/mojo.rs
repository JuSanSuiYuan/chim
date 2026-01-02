use crate::backend::CodegenBackend;
use crate::ir::{Module, Function, Instruction, Type};
use std::error::Error;

/// Mojo后端
/// 生成Mojo代码，AI原生语言，统一CPU/GPU执行
pub struct MojoBackend {
    pub use_gpu: bool,
    pub simd_width: usize,
    pub use_mlir: bool,
}

impl MojoBackend {
    pub fn new() -> Self {
        Self {
            use_gpu: true,
            simd_width: 8,
            use_mlir: true,
        }
    }
    
    fn mojo_type(&self, ty: &Type) -> String {
        match ty {
            Type::Int32 | Type::Int64 => "Int".to_string(),
            Type::Float32 => "Float32".to_string(),
            Type::Float64 => "Float64".to_string(),
            Type::Bool => "Bool".to_string(),
            Type::Void => "None".to_string(),
            Type::String => "String".to_string(),
            Type::Array(elem_ty, size) => {
                format!("StaticTuple[{}, {}]", self.mojo_type(elem_ty), size)
            },
            Type::Ptr(inner) | Type::Ref(inner) | Type::MutRef(inner) => {
                format!("Pointer[{}]", self.mojo_type(inner))
            },
            _ => "Float32".to_string(),
        }
    }
    
    fn generate_function(&self, func: &Function) -> String {
        let mut code = String::new();
        
        // Function decorator for GPU
        if self.use_gpu {
            code.push_str("@parameter\n");
            code.push_str("@always_inline\n");
        }
        
        // Function definition
        code.push_str("fn ");
        code.push_str(&func.name);
        code.push_str("(");
        
        // Parameters with Mojo type annotations
        let params: Vec<String> = func.params.iter()
            .map(|(name, ty)| format!("{}: {}", name, self.mojo_type(ty)))
            .collect();
        code.push_str(&params.join(", "));
        
        // Return type
        code.push_str(") -> ");
        code.push_str(&self.mojo_type(&func.return_type));
        code.push_str(":\n");
        
        // SIMD vectorization
        if self.simd_width > 1 {
            code.push_str(&format!("    # SIMD vectorization (width={})\n", self.simd_width));
            code.push_str(&format!("    alias simd_width = {}\n", self.simd_width));
            code.push_str("    var vec_result: SIMD[DType.float32, simd_width]\n\n");
        }
        
        // Function body
        for inst in &func.body {
            code.push_str("    ");
            code.push_str(&self.generate_instruction(inst));
            code.push_str("\n");
        }
        
        code.push_str("\n");
        code
    }
    
    fn generate_instruction(&self, inst: &Instruction) -> String {
        match inst {
            Instruction::Alloca { dest, ty } => {
                format!("var {}: {}", dest, self.mojo_type(ty))
            },
            Instruction::Store { dest, src } => {
                format!("{} = {}", dest, src)
            },
            Instruction::Load { dest, src } => {
                format!("let {} = {}", dest, src)
            },
            Instruction::Add { dest, left, right } => {
                format!("let {} = {} + {}", dest, left, right)
            },
            Instruction::Sub { dest, left, right } => {
                format!("let {} = {} - {}", dest, left, right)
            },
            Instruction::Mul { dest, left, right } => {
                format!("let {} = {} * {}", dest, left, right)
            },
            Instruction::Div { dest, left, right } => {
                format!("let {} = {} / {}", dest, left, right)
            },
            Instruction::Call { dest, func, args } => {
                // Mojo math functions
                let mojo_func = match func.as_str() {
                    "sqrt" => "math.sqrt",
                    "sin" => "math.sin",
                    "cos" => "math.cos",
                    "exp" => "math.exp",
                    "log" => "math.log",
                    _ => func.as_str(),
                };
                
                if let Some(d) = dest {
                    format!("let {} = {}({})", d, mojo_func, args.join(", "))
                } else {
                    format!("{}({})", mojo_func, args.join(", "))
                }
            },
            Instruction::Return(val) => {
                if let Some(v) = val {
                    format!("return {}", v)
                } else {
                    "return".to_string()
                }
            },
            _ => "# Unsupported instruction".to_string(),
        }
    }
    
    fn generate_gpu_kernel(&self, func: &Function) -> String {
        let mut code = String::new();
        
        code.push_str("# GPU Kernel (Mojo GPU extension)\n");
        code.push_str("@register_passable(\"trivial\")\n");
        code.push_str("struct GPUKernel:\n");
        code.push_str(&format!("    @staticmethod\n"));
        code.push_str(&format!("    fn {}(", func.name));
        
        let params: Vec<String> = func.params.iter()
            .map(|(name, ty)| format!("{}: {}", name, self.mojo_type(ty)))
            .collect();
        code.push_str(&params.join(", "));
        code.push_str(") -> ");
        code.push_str(&self.mojo_type(&func.return_type));
        code.push_str(":\n");
        
        code.push_str("        # GPU thread indexing\n");
        code.push_str("        let tid = gpu.thread_id()\n");
        code.push_str("        let block_id = gpu.block_id()\n");
        code.push_str("        let block_size = gpu.block_size()\n");
        code.push_str("        let gid = block_id * block_size + tid\n\n");
        
        for inst in &func.body {
            code.push_str("        ");
            code.push_str(&self.generate_instruction(inst));
            code.push_str("\n");
        }
        
        code.push_str("\n");
        code
    }
    
    fn generate_main(&self, module: &Module) -> String {
        let mut code = String::new();
        
        code.push_str("# Main execution\n");
        code.push_str("fn main():\n");
        code.push_str("    print(\"Chim Compiler - Mojo Backend\")\n");
        code.push_str("    print(\"AI-native unified CPU/GPU execution\")\n\n");
        
        if self.use_gpu {
            code.push_str("    # GPU execution\n");
            for func in &module.functions {
                code.push_str(&format!("    # Launch {} on GPU\n", func.name));
                code.push_str(&format!("    let result = GPUKernel.{}(/* args */)\n", func.name));
                code.push_str("    print(result)\n\n");
            }
        } else {
            code.push_str("    # CPU execution\n");
            for func in &module.functions {
                code.push_str(&format!("    let result = {}(/* args */)\n", func.name));
                code.push_str("    print(result)\n\n");
            }
        }
        
        code
    }
}

impl CodegenBackend for MojoBackend {
    fn name(&self) -> &str {
        "mojo"
    }
    
    fn generate(&self, module: &Module) -> Result<String, Box<dyn Error>> {
        let mut code = String::new();
        
        // Header
        code.push_str("# Generated by Chim Compiler - Mojo Backend\n");
        code.push_str("# Target: Mojo (AI-native language)\n");
        code.push_str("# Unified CPU/GPU execution with zero-cost abstractions\n");
        code.push_str("# https://docs.modular.com/mojo/\n\n");
        
        // Imports
        code.push_str("from memory import UnsafePointer\n");
        code.push_str("from math import sqrt, sin, cos, exp, log\n");
        code.push_str("from algorithm import vectorize, parallelize\n");
        code.push_str("from tensor import Tensor\n");
        
        if self.use_gpu {
            code.push_str("from gpu import *\n");
        }
        code.push_str("\n");
        
        // CPU functions
        code.push_str("# CPU Functions\n");
        for func in &module.functions {
            code.push_str(&self.generate_function(func));
        }
        
        // GPU kernels
        if self.use_gpu {
            code.push_str("\n# GPU Kernels\n");
            for func in &module.functions {
                code.push_str(&self.generate_gpu_kernel(func));
            }
        }
        
        // Main function
        code.push_str(&self.generate_main(module));
        
        Ok(code)
    }
    
    fn file_extension(&self) -> &str {
        ".mojo"
    }
    
    fn supports_optimization(&self) -> bool {
        true
    }
}
