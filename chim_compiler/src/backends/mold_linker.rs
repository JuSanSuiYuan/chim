use std::error::Error;
use std::process::Command;

#[derive(Clone)]
pub struct MOLDLinker {
    pub target_triple: String,
    pub output_name: String,
    pub library_paths: Vec<String>,
    pub libraries: Vec<String>,
    pub linker_flags: Vec<String>,
    pub threads: usize,
}

impl MOLDLinker {
    pub fn new() -> Self {
        Self {
            target_triple: "x86_64-unknown-linux-gnu".to_string(),
            output_name: "a.out".to_string(),
            library_paths: Vec::new(),
            libraries: Vec::new(),
            linker_flags: Vec::new(),
            threads: num_cpus::get(),
        }
    }
    
    pub fn with_target(mut self, triple: &str) -> Self {
        self.target_triple = triple.to_string();
        self
    }
    
    pub fn with_output(mut self, name: &str) -> Self {
        self.output_name = name.to_string();
        self
    }
    
    pub fn add_library_path<P: Into<String>>(mut self, path: P) -> Self {
        self.library_paths.push(path.into());
        self
    }
    
    pub fn add_library<L: Into<String>>(mut self, lib: L) -> Self {
        self.libraries.push(lib.into());
        self
    }
    
    pub fn add_flag<F: Into<String>>(mut self, flag: F) -> Self {
        self.linker_flags.push(flag.into());
        self
    }
    
    pub fn with_threads(mut self, n: usize) -> Self {
        self.threads = n;
        self
    }
    
    fn find_mold_path() -> String {
        std::env::var("MOLD_PATH").unwrap_or_else(|_| {
            std::env::var("MOLD").unwrap_or_else(|_| "mold".to_string())
        })
    }
    
    fn get_arch_from_triple(triple: &str) -> &str {
        if triple.starts_with("x86_64") {
            "x86-64"
        } else if triple.starts_with("aarch64") || triple.starts_with("arm64") {
            "aarch64"
        } else if triple.starts_with("i686") || triple.starts_with("i386") {
            "i386"
        } else if triple.starts_with("riscv") {
            if triple.contains("64") {
                "rv64"
            } else {
                "rv32"
            }
        } else {
            "unknown"
        }
    }
    
    fn get_os_from_triple(triple: &str) -> &str {
        if triple.contains("linux") {
            "linux"
        } else if triple.contains("darwin") || triple.contains("macos") {
            "macos"
        } else if triple.contains("windows") {
            "windows"
        } else {
            "linux"
        }
    }
    
    pub fn link(&self, objects: &[&str]) -> Result<(), Box<dyn Error>> {
        let mold_path = Self::find_mold_path();
        
        let mut cmd = Command::new(&mold_path);
        
        cmd.arg("-o").arg(&self.output_name);
        cmd.arg("--threads").arg(self.threads.to_string());
        
        let arch = Self::get_arch_from_triple(&self.target_triple);
        let os = Self::get_os_from_triple(&self.target_triple);
        
        cmd.arg("-m").arg(format!("{}_{}", arch, os));
        
        for obj in objects {
            cmd.arg(obj);
        }
        
        for path in &self.library_paths {
            cmd.arg("-L").arg(path);
        }
        
        for lib in &self.libraries {
            cmd.arg(format!("-l{}", lib));
        }
        
        cmd.arg("--start-group");
        for lib in &self.libraries {
            cmd.arg(format!("-l{}", lib));
        }
        cmd.arg("--end-group");
        
        for flag in &self.linker_flags {
            cmd.arg(flag);
        }
        
        let output = cmd.output().map_err(|e| {
            format!("Failed to run mold: {}", e)
        })?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("MOLD failed: {}", stderr).into());
        }
        
        Ok(())
    }
    
    pub fn link_static(&self, objects: &[&str]) -> Result<(), Box<dyn Error>> {
        let mut cmd = Command::new(Self::find_mold_path());
        cmd.arg("-o").arg(&self.output_name);
        cmd.arg("--threads").arg(self.threads.to_string());
        cmd.arg("-static");
        for obj in objects {
            cmd.arg(obj);
        }
        for path in &self.library_paths {
            cmd.arg("-L").arg(path);
        }
        for lib in &self.libraries {
            cmd.arg(format!("-l{}", lib));
        }
        let output = cmd.output()?;
        if !output.status.success() {
            return Err(String::from_utf8_lossy(&output.stderr).into());
        }
        Ok(())
    }
    
    pub fn link_shared(&self, objects: &[&str]) -> Result<(), Box<dyn Error>> {
        let mut cmd = Command::new(Self::find_mold_path());
        cmd.arg("-o").arg(&self.output_name);
        cmd.arg("--threads").arg(self.threads.to_string());
        cmd.arg("-shared");
        for obj in objects {
            cmd.arg(obj);
        }
        for path in &self.library_paths {
            cmd.arg("-L").arg(path);
        }
        for lib in &self.libraries {
            cmd.arg(format!("-l{}", lib));
        }
        let output = cmd.output()?;
        if !output.status.success() {
            return Err(String::from_utf8_lossy(&output.stderr).into());
        }
        Ok(())
    }
}

impl Default for MOLDLinker {
    fn default() -> Self {
        Self::new()
    }
}
