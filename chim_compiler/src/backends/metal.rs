use crate::backend::CodegenBackend;
use crate::ir::{Module, Function, Instruction, Type};
use std::error::Error;

/// Metal GPU后端
/// 生成Apple Metal Shading Language代码，适用于macOS/iOS
pub struct MetalBackend {
    pub use_simd: bool,
    pub threads_per_threadgroup: usize,
}

impl MetalBackend {
    pub fn new() -> Self {
        Self {
            use_simd: true,
            threads_per_threadgroup: 256,
        }
    }
    
    fn metal_type(&self, ty: &Type) -> String {
        match ty {
            Type::Int32 | Type::Int64 => "int".to_string(),
            Type::Float32 => "float".to_string(),
            Type::Float64 => "double".to_string(),
            Type::Bool => "bool".to_string(),
            Type::Void => "void".to_string(),
            Type::String => "constant char*".to_string(),
            Type::Array(elem_ty, _) => format!("device {}*", self.metal_type(elem_ty)),
            Type::Ptr(inner) | Type::Ref(inner) | Type::MutRef(inner) => {
                format!("device {}*", self.metal_type(inner))
            },
            _ => "float4".to_string(),
        }
    }
    
    fn generate_kernel(&self, func: &Function) -> String {
        let mut code = String::new();
        
        // Metal kernel function
        code.push_str("kernel void ");
        code.push_str(&func.name);
        code.push_str("(\n");
        
        // Device buffers
        code.push_str("    device float* input [[buffer(0)]],\n");
        code.push_str("    device float* output [[buffer(1)]],\n");
        code.push_str("    constant uint& dataSize [[buffer(2)]],\n");
        
        // Thread position
        code.push_str("    uint gid [[thread_position_in_grid]],\n");
        code.push_str("    uint tid [[thread_position_in_threadgroup]],\n");
        code.push_str("    uint tgid [[threadgroup_position_in_grid]]\n");
        code.push_str(") {\n");
        
        // Thread indexing
        code.push_str("    // Thread indexing\n");
        code.push_str("    uint index = gid;\n");
        code.push_str("    if (index >= dataSize) return;\n\n");
        
        // Threadgroup memory
        code.push_str("    // Threadgroup (shared) memory\n");
        code.push_str(&format!("    threadgroup float sharedData[{}];\n\n", 
            self.threads_per_threadgroup));
        
        // SIMD operations
        if self.use_simd {
            code.push_str("    // SIMD operations\n");
            code.push_str("    simd_float4 vecData;\n\n");
        }
        
        // Function body
        for inst in &func.body {
            code.push_str("    ");
            code.push_str(&self.generate_instruction(inst));
            code.push_str("\n");
        }
        
        // Synchronization
        code.push_str("\n    // Synchronization\n");
        code.push_str("    threadgroup_barrier(mem_flags::mem_threadgroup);\n");
        
        code.push_str("}\n\n");
        code
    }
    
    fn generate_instruction(&self, inst: &Instruction) -> String {
        match inst {
            Instruction::Alloca { dest, ty } => {
                format!("{} {};", self.metal_type(ty), dest)
            },
            Instruction::Store { dest, src } => {
                format!("{} = {};", dest, src)
            },
            Instruction::Load { dest, src } => {
                format!("auto {} = {};", dest, src)
            },
            Instruction::Add { dest, left, right } => {
                format!("float {} = {} + {};", dest, left, right)
            },
            Instruction::Sub { dest, left, right } => {
                format!("float {} = {} - {};", dest, left, right)
            },
            Instruction::Mul { dest, left, right } => {
                format!("float {} = {} * {};", dest, left, right)
            },
            Instruction::Div { dest, left, right } => {
                format!("float {} = {} / {};", dest, left, right)
            },
            Instruction::Call { dest, func, args } => {
                // Metal math functions
                let metal_func = match func.as_str() {
                    "sqrt" => "sqrt",
                    "sin" => "sin",
                    "cos" => "cos",
                    "exp" => "exp",
                    "log" => "log",
                    _ => func.as_str(),
                };
                
                if let Some(d) = dest {
                    format!("float {} = {}({});", d, metal_func, args.join(", "))
                } else {
                    format!("{}({});", metal_func, args.join(", "))
                }
            },
            Instruction::Return(_) => {
                "return;".to_string()
            },
            _ => "// Unsupported instruction".to_string(),
        }
    }
    
    fn generate_host_code(&self, module: &Module) -> String {
        let mut code = String::new();
        
        code.push_str("// Metal C++ Host Code (Objective-C++)\n");
        code.push_str("#import <Metal/Metal.h>\n\n");
        code.push_str("void executeKernels() {\n");
        code.push_str("    // Get Metal device\n");
        code.push_str("    id<MTLDevice> device = MTLCreateSystemDefaultDevice();\n");
        code.push_str("    id<MTLCommandQueue> commandQueue = [device newCommandQueue];\n\n");
        
        for func in &module.functions {
            code.push_str(&format!("    // Execute {} kernel\n", func.name));
            code.push_str("    id<MTLCommandBuffer> commandBuffer = [commandQueue commandBuffer];\n");
            code.push_str("    id<MTLComputeCommandEncoder> encoder = [commandBuffer computeCommandEncoder];\n");
            code.push_str(&format!("    [encoder setComputePipelineState:{}Pipeline];\n", func.name));
            code.push_str("    [encoder setBuffer:inputBuffer offset:0 atIndex:0];\n");
            code.push_str("    [encoder setBuffer:outputBuffer offset:0 atIndex:1];\n\n");
            
            code.push_str(&format!("    MTLSize threadgroupSize = MTLSizeMake({}, 1, 1);\n", 
                self.threads_per_threadgroup));
            code.push_str("    MTLSize gridSize = MTLSizeMake(dataSize, 1, 1);\n");
            code.push_str("    [encoder dispatchThreads:gridSize threadsPerThreadgroup:threadgroupSize];\n");
            code.push_str("    [encoder endEncoding];\n");
            code.push_str("    [commandBuffer commit];\n");
            code.push_str("    [commandBuffer waitUntilCompleted];\n\n");
        }
        
        code.push_str("}\n");
        code
    }
}

impl CodegenBackend for MetalBackend {
    fn name(&self) -> &str {
        "metal"
    }
    
    fn generate(&self, module: &Module) -> Result<String, Box<dyn Error>> {
        let mut code = String::new();
        
        // Header
        code.push_str("/* Generated by Chim Compiler - Metal Backend */\n");
        code.push_str("/* Target: Apple Metal Shading Language */\n");
        code.push_str("/* Platform: macOS/iOS/iPadOS */\n\n");
        
        // Metal includes
        code.push_str("#include <metal_stdlib>\n");
        code.push_str("using namespace metal;\n\n");
        
        // Kernel functions
        code.push_str("// Metal Compute Kernels\n");
        for func in &module.functions {
            code.push_str(&self.generate_kernel(func));
        }
        
        // Host code
        code.push_str(&self.generate_host_code(module));
        
        Ok(code)
    }
    
    fn file_extension(&self) -> &str {
        ".metal"
    }
    
    fn supports_optimization(&self) -> bool {
        true
    }
}
