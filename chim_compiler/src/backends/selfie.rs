use crate::backend::CodegenBackend;
use crate::ir::{Module, Function, Instruction, Type as IRType, Global, Struct};
use std::error::Error;
use std::process::{Command, Stdio};
use std::io::Write;

#[derive(Clone, Copy, Debug)]
pub enum SelfieArch {
    X86_64,
    RISCV64,
}

pub struct SelfieBackend {
    pub arch: SelfieArch,
    pub optimize: bool,
    pub emulate: bool,
}

impl SelfieBackend {
    pub fn new() -> Self {
        Self {
            arch: SelfieArch::X86_64,
            optimize: true,
            emulate: true,
        }
    }
    
    pub fn with_arch(mut self, arch: SelfieArch) -> Self {
        self.arch = arch;
        self
    }
    
    pub fn with_optimization(mut self, enabled: bool) -> Self {
        self.optimize = enabled;
        self
    }
    
    pub fn with_emulation(mut self, enabled: bool) -> Self {
        self.emulate = enabled;
        self
    }
    
    fn convert_type(&self, ty: &IRType) -> String {
        match ty {
            IRType::Void => "void".to_string(),
            IRType::Int32 => "int32_t".to_string(),
            IRType::Int64 => "int64_t".to_string(),
            IRType::Float32 => "float".to_string(),
            IRType::Float64 => "double".to_string(),
            IRType::Bool => "uint8_t".to_string(),
            IRType::String => "char*".to_string(),
            IRType::Ptr(inner) => format!("{}*", self.convert_type(inner)),
            IRType::Ref(inner) => format!("{}*", self.convert_type(inner)),
            IRType::MutRef(inner) => format!("{}*", self.convert_type(inner)),
            IRType::Array(inner, size) => format!("{}*", self.convert_type(inner)),
            IRType::Struct(name) => format!("struct_{}", name),
        }
    }
    
    fn generate_function(&self, func: &Function) -> String {
        let mut output = String::new();
        
        let ret_type = self.convert_type(&func.return_type);
        
        output.push_str(&format!("{} ", ret_type));
        
        output.push_str(&format!("{}(", func.name));
        
        let params: Vec<String> = func.params.iter()
            .map(|(name, ty)| format!("{} {}", self.convert_type(ty), name))
            .collect();
        output.push_str(&params.join(", "));
        output.push_str(") {\n");
        
        let mut temp_counter = 0;
        let mut label_counter = 0;
        let mut temp_vars = std::collections::HashSet::new();
        let mut labels: std::collections::HashMap<String, String> = std::collections::HashMap::new();
        
        for inst in &func.body {
            match inst {
                Instruction::Alloca { dest, ty } => {
                    temp_vars.insert(dest.clone());
                    output.push_str(&format!("  {} {};\n", self.convert_type(ty), dest));
                }
                Instruction::Load { dest, src } => {
                    if !temp_vars.contains(dest) {
                        temp_vars.insert(dest.clone());
                    }
                    output.push_str(&format!("  {} = *({});\n", dest, src));
                }
                Instruction::Store { dest, src } => {
                    output.push_str(&format!("  *({}) = {};\n", dest, src));
                }
                Instruction::Add { dest, left, right } => {
                    output.push_str(&format!("  {} = {} + {};\n", dest, left, right));
                }
                Instruction::Sub { dest, left, right } => {
                    output.push_str(&format!("  {} = {} - {};\n", dest, left, right));
                }
                Instruction::Mul { dest, left, right } => {
                    output.push_str(&format!("  {} = {} * {};\n", dest, left, right));
                }
                Instruction::Div { dest, left, right } => {
                    output.push_str(&format!("  {} = {} / {};\n", dest, left, right));
                }
                Instruction::Mod { dest, left, right } => {
                    output.push_str(&format!("  {} = {} % {};\n", dest, left, right));
                }
                Instruction::Eq { dest, left, right } => {
                    output.push_str(&format!("  {} = ({} == {});\n", dest, left, right));
                }
                Instruction::Ne { dest, left, right } => {
                    output.push_str(&format!("  {} = ({} != {});\n", dest, left, right));
                }
                Instruction::Lt { dest, left, right } => {
                    output.push_str(&format!("  {} = ({} < {});\n", dest, left, right));
                }
                Instruction::Le { dest, left, right } => {
                    output.push_str(&format!("  {} = ({} <= {});\n", dest, left, right));
                }
                Instruction::Gt { dest, left, right } => {
                    output.push_str(&format!("  {} = ({} > {});\n", dest, left, right));
                }
                Instruction::Ge { dest, left, right } => {
                    output.push_str(&format!("  {} = ({} >= {});\n", dest, left, right));
                }
                Instruction::And { dest, left, right } => {
                    output.push_str(&format!("  {} = ({} && {});\n", dest, left, right));
                }
                Instruction::Or { dest, left, right } => {
                    output.push_str(&format!("  {} = ({} || {});\n", dest, left, right));
                }
                Instruction::Not { dest, src } => {
                    output.push_str(&format!("  {} = !{};\n", dest, src));
                }
                Instruction::Call { dest, func: func_name, args } => {
                    let args_str: Vec<String> = args.iter()
                        .map(|a| a.clone())
                        .collect();
                    if let Some(d) = dest {
                        output.push_str(&format!("  {} = {}({});\n", d, func_name, args_str.join(", ")));
                    } else {
                        output.push_str(&format!("  {}({});\n", func_name, args_str.join(", ")));
                    }
                }
                Instruction::Br(label) => {
                    output.push_str(&format!("  goto {};\n", label));
                }
                Instruction::CondBr { cond, true_bb, false_bb } => {
                    output.push_str(&format!("  if ({}) goto {}; else goto {};\n", cond, true_bb, false_bb));
                }
                Instruction::Label(name) => {
                    output.push_str(&format!("{}:\n", name));
                }
                Instruction::Return(Some(value)) => {
                    output.push_str(&format!("  return {};\n", value));
                }
                Instruction::Return(None) => {
                    output.push_str("  return;\n");
                }
                Instruction::GetPointer { dest, src, offset } => {
                    output.push_str(&format!("  {} = (void*)((char*){} + {});\n", dest, src, offset));
                }
                Instruction::Borrow { dest, src, mutable: _ } => {
                    output.push_str(&format!("  {} = &{};\n", dest, src));
                }
                Instruction::Deref { dest, src } => {
                    output.push_str(&format!("  {} = *{};\n", dest, src));
                }
                _ => {
                    output.push_str(&format!("  /* {:?} */\n", inst));
                }
            }
        }
        
        output.push_str("}\n\n");
        output
    }
    
    fn generate_struct(&self, struct_: &Struct) -> String {
        let mut output = String::new();
        output.push_str(&format!("struct {} {{\n", struct_.name));
        for (field_name, field_type) in &struct_.fields {
            output.push_str(&format!("  {} {};\n", self.convert_type(field_type), field_name));
        }
        output.push_str("};\n\n");
        output
    }
    
    fn generate_global(&self, global: &Global) -> String {
        let ty = self.convert_type(&global.ty);
        format!("{} {};\n", ty, global.name)
    }
    
    fn generate_header(&self) -> String {
        let mut code = String::new();
        
        code.push_str("/* Chim -> Selfie (Educational Self-Hosted Compiler) */\n");
        code.push_str("/* Target: ");
        match self.arch {
            SelfieArch::X86_64 => code.push_str("x86-64"),
            SelfieArch::RISCV64 => code.push_str("RISC-V 64-bit"),
        };
        code.push_str(" */\n\n");
        
        code.push_str("#include <stdint.h>\n");
        code.push_str("#include <stdio.h>\n");
        code.push_str("#include <stdlib.h>\n\n");
        
        code.push_str("/* Selfie Runtime */\n");
        code.push_str("void* palloc(uint64_t n) { return malloc(n); }\n");
        code.push_str("void pfree(void* p) { free(p); }\n\n");
        
        code
    }
}

impl CodegenBackend for SelfieBackend {
    fn name(&self) -> &str {
        match self.arch {
            SelfieArch::X86_64 => "Selfie (x86-64)",
            SelfieArch::RISCV64 => "Selfie (RISC-V)",
        }
    }
    
    fn generate(&self, module: &Module) -> Result<String, Box<dyn Error>> {
        let mut code = self.generate_header();
        
        for struct_ in &module.structs {
            code.push_str(&self.generate_struct(struct_));
        }
        
        for global in &module.globals {
            code.push_str(&self.generate_global(global));
        }
        
        code.push_str("/* Function Definitions */\n");
        for func in &module.functions {
            code.push_str(&self.generate_function(func));
        }
        
        code.push_str("/* Selfie Entry Point */\n");
        code.push_str("int64_t main() {\n");
        code.push_str("  return 0;\n");
        code.push_str("}\n");
        
        Ok(code)
    }
    
    fn file_extension(&self) -> &str {
        "c"
    }
}

impl Default for SelfieBackend {
    fn default() -> Self {
        Self::new()
    }
}

pub struct SelfieCompiler {
    backend: SelfieBackend,
}

impl SelfieCompiler {
    pub fn new() -> Self {
        Self {
            backend: SelfieBackend::new(),
        }
    }
    
    pub fn with_arch(mut self, arch: SelfieArch) -> Self {
        self.backend = self.backend.with_arch(arch);
        self
    }
    
    pub fn compile(&self, module: &Module, output: &str) -> Result<(), Box<dyn Error>> {
        let selfie_code = self.backend.generate(module)?;
        
        let selfie_path = std::env::var("SELFIE_PATH")
            .unwrap_or_else(|_| "selfie".to_string());
        
        let arch_flag = match self.backend.arch {
            SelfieArch::X86_64 => "-x86-64",
            SelfieArch::RISCV64 => "-riscv64",
        };
        
        let optimize_flag = if self.backend.optimize { "-O2" } else { "-O0" };
        
        let mut selfie = Command::new(&selfie_path);
        selfie.arg("-c");
        selfie.arg(arch_flag);
        selfie.arg(optimize_flag);
        
        selfie.arg("-o").arg(output);
        
        selfie.stdin(Stdio::piped());
        selfie.stdout(Stdio::piped());
        selfie.stderr(Stdio::piped());
        
        let mut child = selfie.spawn().map_err(|e| {
            format!("Failed to run selfie: {}. Make sure selfie is installed and in PATH.", e)
        })?;
        
        {
            let stdin = child.stdin.as_mut().expect("Failed to get stdin");
            stdin.write_all(selfie_code.as_bytes())?;
        }
        
        let output = child.wait_with_output().map_err(|e| {
            format!("Failed to read selfie output: {}", e)
        })?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Selfie compilation failed: {}", stderr).into());
        }
        
        Ok(())
    }
    
    pub fn compile_and_link(&self, module: &Module, output: &str) -> Result<(), Box<dyn Error>> {
        let selfie_path = std::env::var("SELFIE_PATH")
            .unwrap_or_else(|_| "selfie".to_string());
        
        let arch_flag = match self.backend.arch {
            SelfieArch::X86_64 => "-x86-64",
            SelfieArch::RISCV64 => "-riscv64",
        };
        
        let selfie_code = self.backend.generate(module)?;
        
        let mut selfie = Command::new(&selfie_path);
        selfie.arg(arch_flag);
        
        if self.backend.emulate {
            selfie.arg("-emulate");
        }
        
        selfie.arg("-o").arg(output);
        
        selfie.stdin(Stdio::piped());
        selfie.stdout(Stdio::piped());
        selfie.stderr(Stdio::piped());
        
        let mut child = selfie.spawn().map_err(|e| {
            format!("Failed to run selfie: {}", e)
        })?;
        
        {
            let stdin = child.stdin.as_mut().expect("Failed to get stdin");
            stdin.write_all(selfie_code.as_bytes())?;
        }
        
        let result = child.wait_with_output()?;
        
        if !result.status.success() {
            let stderr = String::from_utf8_lossy(&result.stderr);
            return Err(format!("Selfie failed: {}", stderr).into());
        }
        
        Ok(())
    }
}

impl Default for SelfieCompiler {
    fn default() -> Self {
        Self::new()
    }
}
