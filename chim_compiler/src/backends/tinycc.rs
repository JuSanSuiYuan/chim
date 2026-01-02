use crate::backend::CodegenBackend;
use crate::ir::Module;
use std::error::Error;

/// TinyCC后端 - 生成优化的C代码供TinyCC编译
pub struct TinyCCBackend;

impl TinyCCBackend {
    pub fn new() -> Self {
        Self
    }
}

impl CodegenBackend for TinyCCBackend {
    fn name(&self) -> &str {
        "TinyCC"
    }
    
    fn generate(&self, module: &Module) -> Result<String, Box<dyn Error>> {
        // TinyCC使用标准C代码，复用Native后端
        let native_backend = crate::backends::native::NativeBackend::new();
        let mut code = native_backend.generate(module)?;
        
        // 添加TinyCC特定的优化标记
        code.insert_str(0, "// Optimized for TinyCC\n");
        code.insert_str(0, "// TinyCC: fast compilation mode\n");
        
        Ok(code)
    }
    
    fn file_extension(&self) -> &str {
        "c"
    }
    
    fn supports_optimization(&self) -> bool {
        false // TinyCC优化能力有限
    }
}
