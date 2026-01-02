use crate::backend::CodegenBackend;
use crate::ir::{Module, Function, Instruction, Type};
use std::error::Error;

/// Vulkan Compute后端
/// 生成Vulkan GLSL Compute Shader代码，跨平台GPU计算
pub struct VulkanBackend {
    pub workgroup_size_x: u32,
    pub workgroup_size_y: u32,
    pub workgroup_size_z: u32,
    pub use_push_constants: bool,
}

impl VulkanBackend {
    pub fn new() -> Self {
        Self {
            workgroup_size_x: 16,
            workgroup_size_y: 16,
            workgroup_size_z: 1,
            use_push_constants: true,
        }
    }
    
    fn glsl_type(&self, ty: &Type) -> String {
        match ty {
            Type::Int32 | Type::Int64 => "int".to_string(),
            Type::Float32 => "float".to_string(),
            Type::Float64 => "double".to_string(),
            Type::Bool => "bool".to_string(),
            Type::Void => "void".to_string(),
            Type::Array(elem_ty, size) => {
                format!("{}[{}]", self.glsl_type(elem_ty), size)
            },
            _ => "vec4".to_string(),
        }
    }
    
    fn generate_compute_shader(&self, func: &Function) -> String {
        let mut code = String::new();
        
        // GLSL version
        code.push_str("#version 450\n\n");
        
        // Workgroup size
        code.push_str(&format!(
            "layout (local_size_x = {}, local_size_y = {}, local_size_z = {}) in;\n\n",
            self.workgroup_size_x, self.workgroup_size_y, self.workgroup_size_z
        ));
        
        // Storage buffers
        code.push_str("// Storage buffers\n");
        code.push_str("layout(set = 0, binding = 0) buffer InputBuffer {\n");
        code.push_str("    float data[];\n");
        code.push_str("} inputBuffer;\n\n");
        
        code.push_str("layout(set = 0, binding = 1) buffer OutputBuffer {\n");
        code.push_str("    float data[];\n");
        code.push_str("} outputBuffer;\n\n");
        
        // Push constants
        if self.use_push_constants {
            code.push_str("// Push constants\n");
            code.push_str("layout(push_constant) uniform PushConstants {\n");
            code.push_str("    uint dataSize;\n");
            code.push_str("    uint stride;\n");
            code.push_str("} pushConstants;\n\n");
        }
        
        // Main compute function
        code.push_str("void main() {\n");
        code.push_str("    // Global invocation ID\n");
        code.push_str("    uint gid = gl_GlobalInvocationID.x;\n");
        code.push_str("    uint lid = gl_LocalInvocationID.x;\n");
        code.push_str("    uint wid = gl_WorkGroupID.x;\n\n");
        
        // Shared memory
        code.push_str("    // Shared memory\n");
        code.push_str(&format!("    shared float sharedData[{}];\n\n", 
            self.workgroup_size_x * self.workgroup_size_y));
        
        // Function body
        for inst in &func.body {
            code.push_str("    ");
            code.push_str(&self.generate_instruction(inst));
            code.push_str("\n");
        }
        
        // Memory barrier
        code.push_str("\n    // Memory barrier\n");
        code.push_str("    memoryBarrierShared();\n");
        code.push_str("    barrier();\n");
        
        code.push_str("}\n");
        code
    }
    
    fn generate_instruction(&self, inst: &Instruction) -> String {
        match inst {
            Instruction::Alloca { dest, ty } => {
                format!("{} {};", self.glsl_type(ty), dest)
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
                if let Some(d) = dest {
                    format!("{} = {}({});", d, func, args.join(", "))
                } else {
                    format!("{}({});", func, args.join(", "))
                }
            },
            Instruction::Return(_) => {
                "return;".to_string()
            },
            _ => "// Unsupported instruction".to_string(),
        }
    }
}

impl CodegenBackend for VulkanBackend {
    fn name(&self) -> &str {
        "vulkan"
    }
    
    fn generate(&self, module: &Module) -> Result<String, Box<dyn Error>> {
        let mut code = String::new();
        
        // Header comment
        code.push_str("/* Generated by Chim Compiler - Vulkan Compute Backend */\n");
        code.push_str("/* Target: Vulkan Compute Shader (GLSL) */\n");
        code.push_str("/* Cross-platform GPU computing */\n\n");
        
        // Generate compute shaders
        for func in &module.functions {
            code.push_str(&self.generate_compute_shader(func));
            code.push_str("\n\n");
        }
        
        Ok(code)
    }
    
    fn file_extension(&self) -> &str {
        ".comp"
    }
    
    fn supports_optimization(&self) -> bool {
        true
    }
}
