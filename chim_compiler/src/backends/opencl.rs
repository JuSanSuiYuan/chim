use crate::backend::CodegenBackend;
use crate::ir::{Module, Function, Instruction, Type};
use std::error::Error;

/// OpenCL后端
/// 生成OpenCL C代码，跨平台GPU计算（传统方案）
pub struct OpenCLBackend {
    pub local_work_size: usize,
    pub use_local_memory: bool,
}

impl OpenCLBackend {
    pub fn new() -> Self {
        Self {
            local_work_size: 256,
            use_local_memory: true,
        }
    }
    
    fn opencl_type(&self, ty: &Type) -> String {
        match ty {
            Type::Int32 | Type::Int64 => "int".to_string(),
            Type::Float32 => "float".to_string(),
            Type::Float64 => "double".to_string(),
            Type::Bool => "bool".to_string(),
            Type::Void => "void".to_string(),
            Type::Array(elem_ty, _) => format!("__global {}*", self.opencl_type(elem_ty)),
            Type::Ptr(inner) | Type::Ref(inner) | Type::MutRef(inner) => {
                format!("__global {}*", self.opencl_type(inner))
            },
            _ => "float4".to_string(),
        }
    }
    
    fn generate_kernel(&self, func: &Function) -> String {
        let mut code = String::new();
        
        // OpenCL kernel function
        code.push_str("__kernel void ");
        code.push_str(&func.name);
        code.push_str("(\n");
        
        // Global memory buffers
        code.push_str("    __global float* input,\n");
        code.push_str("    __global float* output,\n");
        code.push_str("    const unsigned int dataSize\n");
        code.push_str(") {\n");
        
        // Thread indexing
        code.push_str("    // Work-item indexing\n");
        code.push_str("    int gid = get_global_id(0);\n");
        code.push_str("    int lid = get_local_id(0);\n");
        code.push_str("    int group_id = get_group_id(0);\n");
        code.push_str("    int group_size = get_local_size(0);\n\n");
        
        // Bounds check
        code.push_str("    if (gid >= dataSize) return;\n\n");
        
        // Local memory
        if self.use_local_memory {
            code.push_str("    // Local (shared) memory\n");
            code.push_str(&format!("    __local float localData[{}];\n\n", self.local_work_size));
        }
        
        // Function body
        for inst in &func.body {
            code.push_str("    ");
            code.push_str(&self.generate_instruction(inst));
            code.push_str("\n");
        }
        
        // Synchronization
        if self.use_local_memory {
            code.push_str("\n    // Work-group barrier\n");
            code.push_str("    barrier(CLK_LOCAL_MEM_FENCE);\n");
        }
        
        code.push_str("}\n\n");
        code
    }
    
    fn generate_instruction(&self, inst: &Instruction) -> String {
        match inst {
            Instruction::Alloca { dest, ty } => {
                format!("{} {};", self.opencl_type(ty), dest)
            },
            Instruction::Store { dest, src } => {
                format!("{} = {};", dest, src)
            },
            Instruction::Load { dest, src } => {
                format!("float {} = {};", dest, src)
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
                // OpenCL built-in functions
                let opencl_func = match func.as_str() {
                    "sqrt" => "sqrt",
                    "sin" => "sin",
                    "cos" => "cos",
                    "exp" => "exp",
                    "log" => "log",
                    "abs" => "fabs",
                    _ => func.as_str(),
                };
                
                if let Some(d) = dest {
                    format!("float {} = {}({});", d, opencl_func, args.join(", "))
                } else {
                    format!("{}({});", opencl_func, args.join(", "))
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
        
        code.push_str("// OpenCL Host Code (C)\n");
        code.push_str("#include <CL/cl.h>\n");
        code.push_str("#include <stdio.h>\n\n");
        
        code.push_str("void executeKernels() {\n");
        code.push_str("    // OpenCL setup\n");
        code.push_str("    cl_platform_id platform;\n");
        code.push_str("    cl_device_id device;\n");
        code.push_str("    cl_context context;\n");
        code.push_str("    cl_command_queue queue;\n");
        code.push_str("    cl_program program;\n\n");
        
        code.push_str("    // Get platform and device\n");
        code.push_str("    clGetPlatformIDs(1, &platform, NULL);\n");
        code.push_str("    clGetDeviceIDs(platform, CL_DEVICE_TYPE_GPU, 1, &device, NULL);\n\n");
        
        code.push_str("    // Create context and command queue\n");
        code.push_str("    context = clCreateContext(NULL, 1, &device, NULL, NULL, NULL);\n");
        code.push_str("    queue = clCreateCommandQueue(context, device, 0, NULL);\n\n");
        
        for func in &module.functions {
            code.push_str(&format!("    // Execute {} kernel\n", func.name));
            code.push_str(&format!("    cl_kernel kernel_{} = clCreateKernel(program, \"{}\", NULL);\n", 
                func.name, func.name));
            code.push_str(&format!("    clSetKernelArg(kernel_{}, 0, sizeof(cl_mem), &inputBuffer);\n", 
                func.name));
            code.push_str(&format!("    clSetKernelArg(kernel_{}, 1, sizeof(cl_mem), &outputBuffer);\n", 
                func.name));
            code.push_str(&format!("    clSetKernelArg(kernel_{}, 2, sizeof(unsigned int), &dataSize);\n\n", 
                func.name));
            
            code.push_str(&format!("    size_t globalWorkSize = dataSize;\n"));
            code.push_str(&format!("    size_t localWorkSize = {};\n", self.local_work_size));
            code.push_str(&format!("    clEnqueueNDRangeKernel(queue, kernel_{}, 1, NULL, &globalWorkSize, &localWorkSize, 0, NULL, NULL);\n", 
                func.name));
            code.push_str("    clFinish(queue);\n\n");
        }
        
        code.push_str("    // Cleanup\n");
        code.push_str("    clReleaseCommandQueue(queue);\n");
        code.push_str("    clReleaseContext(context);\n");
        code.push_str("}\n");
        code
    }
}

impl CodegenBackend for OpenCLBackend {
    fn name(&self) -> &str {
        "opencl"
    }
    
    fn generate(&self, module: &Module) -> Result<String, Box<dyn Error>> {
        let mut code = String::new();
        
        // Header
        code.push_str("/* Generated by Chim Compiler - OpenCL Backend */\n");
        code.push_str("/* Target: OpenCL C (Cross-platform GPU) */\n");
        code.push_str("/* Note: OpenCL is deprecated on macOS, use Metal instead */\n\n");
        
        // Kernel functions
        code.push_str("// OpenCL Kernels\n");
        for func in &module.functions {
            code.push_str(&self.generate_kernel(func));
        }
        
        // Host code
        code.push_str(&self.generate_host_code(module));
        
        Ok(code)
    }
    
    fn file_extension(&self) -> &str {
        ".cl"
    }
    
    fn supports_optimization(&self) -> bool {
        true
    }
}
