use crate::backend::CodegenBackend;
use crate::ir::{Module, Function, Instruction, Type};
use std::error::Error;

/// CUDA GPU后端
/// 生成NVIDIA CUDA C代码，适用于AI/科学计算
pub struct CUDABackend {
    pub compute_capability: String, // e.g., "sm_80" for A100
    pub use_shared_memory: bool,
    pub block_size: usize,
}

impl CUDABackend {
    pub fn new() -> Self {
        Self {
            compute_capability: "sm_70".to_string(), // Default: Volta/Turing
            use_shared_memory: true,
            block_size: 256,
        }
    }
    
    fn cuda_type(&self, ty: &Type) -> String {
        match ty {
            Type::Int32 | Type::Int64 => "int".to_string(),
            Type::Float32 => "float".to_string(),
            Type::Float64 => "double".to_string(),
            Type::Bool => "bool".to_string(),
            Type::Void => "void".to_string(),
            Type::String => "char*".to_string(),
            Type::Array(elem_ty, _) => format!("{}*", self.cuda_type(elem_ty)),
            Type::Ptr(inner) | Type::Ref(inner) | Type::MutRef(inner) => {
                format!("{}*", self.cuda_type(inner))
            },
            _ => "void*".to_string(),
        }
    }
    
    fn generate_kernel(&self, func: &Function) -> String {
        let mut code = String::new();
        
        // CUDA kernel declaration
        code.push_str("__global__ void ");
        code.push_str(&func.name);
        code.push_str("(");
        
        // Parameters
        let params: Vec<String> = func.params.iter()
            .map(|(name, ty)| format!("{} {}", self.cuda_type(ty), name))
            .collect();
        code.push_str(&params.join(", "));
        code.push_str(") {\n");
        
        // Thread indexing
        code.push_str("    // Thread indexing\n");
        code.push_str("    int tid = blockIdx.x * blockDim.x + threadIdx.x;\n");
        code.push_str("    int stride = blockDim.x * gridDim.x;\n\n");
        
        // Shared memory if enabled
        if self.use_shared_memory {
            code.push_str("    // Shared memory\n");
            code.push_str(&format!("    __shared__ float shared_data[{}];\n\n", self.block_size));
        }
        
        // Function body
        for inst in &func.body {
            code.push_str("    ");
            code.push_str(&self.generate_instruction(inst));
            code.push_str("\n");
        }
        
        code.push_str("}\n\n");
        code
    }
    
    fn generate_instruction(&self, inst: &Instruction) -> String {
        match inst {
            Instruction::Alloca { dest, ty } => {
                format!("{} {};", self.cuda_type(ty), dest)
            },
            Instruction::Store { dest, src } => {
                format!("{} = {};", dest, src)
            },
            Instruction::Load { dest, src } => {
                format!("auto {} = {};", dest, src)
            },
            Instruction::Add { dest, left, right } => {
                format!("int {} = {} + {};", dest, left, right)
            },
            Instruction::Sub { dest, left, right } => {
                format!("int {} = {} - {};", dest, left, right)
            },
            Instruction::Mul { dest, left, right } => {
                format!("int {} = {} * {};", dest, left, right)
            },
            Instruction::Div { dest, left, right } => {
                format!("int {} = {} / {};", dest, left, right)
            },
            Instruction::Call { dest, func, args } => {
                if let Some(d) = dest {
                    format!("{} = {}({});", d, func, args.join(", "))
                } else {
                    format!("{}({});", func, args.join(", "))
                }
            },
            Instruction::Return(val) => {
                if let Some(v) = val {
                    format!("return {};", v)
                } else {
                    "return;".to_string()
                }
            },
            Instruction::Label(name) => {
                format!("{}:", name)
            },
            Instruction::Br(target) => {
                format!("goto {};", target)
            },
            Instruction::CondBr { cond, true_bb, false_bb } => {
                format!("if ({}) goto {}; else goto {};", cond, true_bb, false_bb)
            },
            _ => "// Unsupported instruction".to_string(),
        }
    }
    
    fn generate_host_code(&self, module: &Module) -> String {
        let mut code = String::new();
        
        code.push_str("// Host code for kernel launch\n");
        code.push_str("void launch_kernels() {\n");
        
        for func in &module.functions {
            code.push_str(&format!("    // Launch {} kernel\n", func.name));
            code.push_str(&format!("    int numBlocks = (dataSize + {} - 1) / {};\n", 
                self.block_size, self.block_size));
            code.push_str(&format!("    {}<<<numBlocks, {}>>>(/* args */);\n", 
                func.name, self.block_size));
            code.push_str("    cudaDeviceSynchronize();\n\n");
        }
        
        code.push_str("}\n");
        code
    }
}

impl CodegenBackend for CUDABackend {
    fn name(&self) -> &str {
        "cuda"
    }
    
    fn generate(&self, module: &Module) -> Result<String, Box<dyn Error>> {
        let mut code = String::new();
        
        // Header
        code.push_str("/* Generated by Chim Compiler - CUDA Backend */\n");
        code.push_str("/* Target: NVIDIA CUDA GPU */\n");
        code.push_str(&format!("/* Compute Capability: {} */\n\n", self.compute_capability));
        
        // Includes
        code.push_str("#include <cuda_runtime.h>\n");
        code.push_str("#include <device_launch_parameters.h>\n");
        code.push_str("#include <stdio.h>\n\n");
        
        // Kernel functions
        code.push_str("// CUDA Kernels\n");
        for func in &module.functions {
            code.push_str(&self.generate_kernel(func));
        }
        
        // Host code
        code.push_str(&self.generate_host_code(module));
        
        Ok(code)
    }
    
    fn file_extension(&self) -> &str {
        ".cu"
    }
    
    fn supports_optimization(&self) -> bool {
        true
    }
}
