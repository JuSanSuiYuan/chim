use crate::backend::CodegenBackend;
use crate::ir::{Module, Function, Instruction, Type as IRType};
use std::error::Error;
use std::process::{Command, Stdio};
use std::io::Write;

#[derive(Clone, Copy)]
pub enum OptimizationLevel {
    O0,
    O1,
    O2,
    O3,
    Oz,
}

impl std::fmt::Debug for OptimizationLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OptimizationLevel::O0 => write!(f, "O0"),
            OptimizationLevel::O1 => write!(f, "O1"),
            OptimizationLevel::O2 => write!(f, "O2"),
            OptimizationLevel::O3 => write!(f, "O3"),
            OptimizationLevel::Oz => write!(f, "Oz"),
        }
    }
}

pub struct LLVMMachineCodeBackend {
    opt_level: OptimizationLevel,
    use_lto: bool,
    target_triple: String,
}

impl LLVMMachineCodeBackend {
    pub fn new() -> Self {
        Self {
            opt_level: OptimizationLevel::O3,
            use_lto: true,
            target_triple: "x86_64-unknown-linux-gnu".to_string(),
        }
    }
    
    pub fn with_optimization(mut self, level: OptimizationLevel) -> Self {
        self.opt_level = level;
        self
    }
    
    pub fn with_lto(mut self, enabled: bool) -> Self {
        self.use_lto = enabled;
        self
    }
    
    pub fn with_target(mut self, triple: &str) -> Self {
        self.target_triple = triple.to_string();
        self
    }
    
    fn generate_llvm_ir(&self, module: &Module) -> String {
        let mut output = String::new();
        
        output.push_str("; Chim Compiler - LLVM IR Output\n");
        output.push_str(&format!("; Target: {}\n", self.target_triple));
        output.push_str(&format!("; Optimization: {:?}\n", self.opt_level));
        if self.use_lto {
            output.push_str("; LTO: Enabled\n");
        }
        output.push_str("\n");
        
        output.push_str("target triple = \"");
        output.push_str(&self.target_triple);
        output.push_str("\"\n\n");
        
        output.push_str("; External declarations\n");
        output.push_str("declare void @println(i8*)\n");
        output.push_str("declare void @print(i8*)\n");
        output.push_str("declare i32 @malloc(i64)\n");
        output.push_str("declare void @free(i8*)\n\n");
        
        output.push_str("; Module functions\n");
        for func in &module.functions {
            output.push_str(&self.generate_function(func));
        }
        
        output
    }
    
    fn generate_function(&self, func: &Function) -> String {
        let mut output = String::new();
        
        let ret_type = self.convert_type(&func.return_type);
        
        let params: Vec<String> = func.params.iter()
            .map(|(name, ty)| format!("{} %{}", self.convert_type(ty), name))
            .collect();
        
        output.push_str(&format!("define {} @{}({}) {{\n", ret_type, func.name, params.join(", ")));
        output.push_str("entry:\n");
        
        for inst in &func.body {
            output.push_str(&self.generate_instruction(inst));
        }
        
        if !func.body.iter().any(|i| matches!(i, Instruction::Return(_) | Instruction::ReturnInPlace(_))) {
            if func.return_type == IRType::Void {
                output.push_str("  ret void\n");
            }
        }
        
        output.push_str("}\n\n");
        output
    }
    
    fn generate_instruction(&self, inst: &Instruction) -> String {
        match inst {
            Instruction::Add { dest, left, right } => {
                format!("  %{} = add i32 %{}, %{}\n", dest, left, right)
            }
            Instruction::Sub { dest, left, right } => {
                format!("  %{} = sub i32 %{}, %{}\n", dest, left, right)
            }
            Instruction::Mul { dest, left, right } => {
                format!("  %{} = mul i32 %{}, %{}\n", dest, left, right)
            }
            Instruction::Div { dest, left, right } => {
                format!("  %{} = sdiv i32 %{}, %{}\n", dest, left, right)
            }
            Instruction::Mod { dest, left, right } => {
                format!("  %{} = srem i32 %{}, %{}\n", dest, left, right)
            }
            Instruction::And { dest, left, right } => {
                format!("  %{} = and i1 %{}, %{}\n", dest, left, right)
            }
            Instruction::Or { dest, left, right } => {
                format!("  %{} = or i1 %{}, %{}\n", dest, left, right)
            }
            Instruction::Not { dest, src } => {
                format!("  %{} = xor i1 %{}, true\n", dest, src)
            }
            Instruction::Eq { dest, left, right } => {
                format!("  %{} = icmp eq i32 %{}, %{}\n", dest, left, right)
            }
            Instruction::Ne { dest, left, right } => {
                format!("  %{} = icmp ne i32 %{}, %{}\n", dest, left, right)
            }
            Instruction::Lt { dest, left, right } => {
                format!("  %{} = icmp slt i32 %{}, %{}\n", dest, left, right)
            }
            Instruction::Le { dest, left, right } => {
                format!("  %{} = icmp sle i32 %{}, %{}\n", dest, left, right)
            }
            Instruction::Gt { dest, left, right } => {
                format!("  %{} = icmp sgt i32 %{}, %{}\n", dest, left, right)
            }
            Instruction::Ge { dest, left, right } => {
                format!("  %{} = icmp sge i32 %{}, %{}\n", dest, left, right)
            }
            Instruction::Alloca { dest, ty } => {
                let ty_str = self.convert_type(ty);
                format!("  %{} = alloca {}\n", dest, ty_str)
            }
            Instruction::Load { dest, src } => {
                format!("  %{} = load i32, i32* %{}\n", dest, src)
            }
            Instruction::Store { dest, src } => {
                format!("  store i32 %{}, i32* %{}\n", src, dest)
            }
            Instruction::GetPointer { dest, src, offset } => {
                format!("  %{} = getelementptr i32, i32* %{}, i32 {}\n", dest, src, offset)
            }
            Instruction::Call { dest, func, args } => {
                let args_str: Vec<String> = args.iter()
                    .map(|a| format!("i32 %{}", a))
                    .collect();
                if let Some(d) = dest {
                    format!("  %{} = call i32 @{}({})\n", d, func, args_str.join(", "))
                } else {
                    format!("  call void @{}({})\n", func, args_str.join(", "))
                }
            }
            Instruction::Br(label) => {
                format!("  br label %{}\n", label)
            }
            Instruction::CondBr { cond, true_bb, false_bb } => {
                format!("  br i1 %{}, label %{}, label %{}\n", cond, true_bb, false_bb)
            }
            Instruction::Label(name) => {
                format!("{}:\n", name)
            }
            Instruction::Return(Some(value)) => {
                format!("  ret i32 %{}\n", value)
            }
            Instruction::Return(None) => {
                "  ret void\n".to_string()
            }
            Instruction::ReturnInPlace(value) => {
                format!("  ret i32 %{} ; RVO\n", value)
            }
            Instruction::Borrow { dest, src, mutable: _ } => {
                format!("  %{} = bitcast i32* %{} to i8*\n", dest, src)
            }
            Instruction::Deref { dest, src } => {
                format!("  %{} = load i32, i32* %{}\n", dest, src)
            }
            Instruction::Phi { dest, incoming } => {
                let pairs: Vec<String> = incoming.iter()
                    .map(|(val, _)| format!("[{} -> %{}]", val, "bb"))
                    .collect();
                format!("  ; phi: %{} = {}\n", dest, pairs.join(", "))
            }
            Instruction::ExtractValue { dest, src, index } => {
                format!("  ; extract %{} from %{} index {}\n", dest, src, index)
            }
            Instruction::InsertValue { dest, src, value, index } => {
                format!("  ; insert {} into %{} index {}\n", value, src, index)
            }
            Instruction::GetElementPtr { dest, src, indices } => {
                let indices_str: Vec<String> = indices.iter()
                    .map(|i| format!("i32 {}", i))
                    .collect();
                format!("  %{} = getelementptr i32, i32* %{}, {}\n", dest, src, indices_str.join(", "))
            }
            Instruction::Nop => {
                "; nop\n".to_string()
            }
            Instruction::Unreachable => {
                "  unreachable\n".to_string()
            }
        }
    }
    
    fn convert_type(&self, ty: &IRType) -> String {
        match ty {
            IRType::Void => "void".to_string(),
            IRType::Int32 => "i32".to_string(),
            IRType::Int64 => "i64".to_string(),
            IRType::Float32 => "float".to_string(),
            IRType::Float64 => "double".to_string(),
            IRType::Bool => "i1".to_string(),
            IRType::String => "i8*".to_string(),
            IRType::Ptr(inner) => format!("{}*", self.convert_type(inner)),
            IRType::Ref(inner) => format!("{}*", self.convert_type(inner)),
            IRType::MutRef(inner) => format!("{}*", self.convert_type(inner)),
            IRType::Array(inner, size) => format!("[{} x {}]", self.convert_type(inner), size),
            IRType::Struct(name) => format!("%struct.{}", name),
        }
    }
    
    fn get_opt_flag(&self) -> String {
        match self.opt_level {
            OptimizationLevel::O0 => "0",
            OptimizationLevel::O1 => "1",
            OptimizationLevel::O2 => "2",
            OptimizationLevel::O3 => "3",
            OptimizationLevel::Oz => "z",
        }.to_string()
    }
}

impl CodegenBackend for LLVMMachineCodeBackend {
    fn name(&self) -> &str {
        if self.use_lto {
            "LLVM (O3 + LTO)"
        } else {
            "LLVM (O3)"
        }
    }
    
    fn generate(&self, module: &Module) -> Result<String, Box<dyn Error>> {
        let llvm_ir = self.generate_llvm_ir(module);
        
        let llc_path = std::env::var("LLC_PATH").unwrap_or_else(|_| "llc".to_string());
        
        let mut llc = Command::new(&llc_path);
        llc.arg(format!("-O{}", self.get_opt_flag()));
        llc.arg("--relocation-model=pic");
        llc.arg("--position-independent");
        
        if self.use_lto {
            llc.arg("-filetype=obj");
            llc.arg("-O3");
        }
        
        llc.arg("-o").arg("-");
        llc.stdin(Stdio::piped());
        llc.stdout(Stdio::piped());
        
        let mut child = llc.spawn().map_err(|e| {
            format!("Failed to spawn llc: {}. Make sure LLVM is installed and llc is in PATH.", e)
        })?;
        
        {
            let stdin = child.stdin.as_mut().expect("Failed to get stdin");
            stdin.write_all(llvm_ir.as_bytes())?;
        }
        let output = child.wait_with_output().map_err(|e| {
            format!("Failed to read llc output: {}", e)
        })?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("llc failed: {}", stderr).into());
        }
        
        let machine_code = output.stdout;
        let hex_string = machine_code.iter()
            .map(|b| format!("{:02x}", b))
            .collect::<Vec<String>>()
            .join("");
        
        Ok(format!("; Machine code ({} bytes)\n; {}\n", machine_code.len(), hex_string))
    }
    
    fn file_extension(&self) -> &str {
        "o"
    }
}

impl Default for LLVMMachineCodeBackend {
    fn default() -> Self {
        Self::new()
    }
}
