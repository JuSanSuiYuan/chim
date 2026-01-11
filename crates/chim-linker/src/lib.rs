use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::io::Write;

#[derive(Debug, Clone, PartialEq)]
pub struct LinkerConfig {
    pub output: PathBuf,
    pub input_files: Vec<PathBuf>,
    pub library_paths: Vec<PathBuf>,
    pub libraries: Vec<String>,
    pub linker: String,
    pub is_static: bool,
    pub is_shared: bool,
    pub strip: bool,
    pub verbose: bool,
}

impl Default for LinkerConfig {
    fn default() -> Self {
        LinkerConfig {
            output: PathBuf::from("a.out"),
            input_files: Vec::new(),
            library_paths: Vec::new(),
            libraries: Vec::new(),
            linker: String::from("cc"),
            is_static: false,
            is_shared: false,
            strip: false,
            verbose: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LinkerError {
    pub message: String,
    pub command: Vec<String>,
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
}

impl std::fmt::Display for LinkerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "linker error: {}", self.message)
    }
}

impl std::error::Error for LinkerError {}

pub struct Linker {
    config: LinkerConfig,
}

impl Linker {
    pub fn new() -> Self {
        Linker {
            config: LinkerConfig::default(),
        }
    }

    pub fn config(&mut self) -> &mut LinkerConfig {
        &mut self.config
    }

    pub fn output(&mut self, path: PathBuf) -> &mut Self {
        self.config.output = path;
        self
    }

    pub fn input(&mut self, files: Vec<PathBuf>) -> &mut Self {
        self.config.input_files = files;
        self
    }

    pub fn library_path(&mut self, path: PathBuf) -> &mut Self {
        self.config.library_paths.push(path);
        self
    }

    pub fn library(&mut self, name: String) -> &mut Self {
        self.config.libraries.push(name);
        self
    }

    pub fn linker(&mut self, name: String) -> &mut Self {
        self.config.linker = name;
        self
    }

    pub fn static_linking(&mut self) -> &mut Self {
        self.config.is_static = true;
        self
    }

    pub fn shared_library(&mut self) -> &mut Self {
        self.config.is_shared = true;
        self
    }

    pub fn strip(&mut self) -> &mut Self {
        self.config.strip = true;
        self
    }

    pub fn link(&self) -> Result<(), LinkerError> {
        let mut cmd = Command::new(&self.config.linker);

        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        for file in &self.config.input_files {
            cmd.arg(file);
        }

        if self.config.is_shared {
            cmd.arg("-shared");
        }

        if self.config.is_static {
            cmd.arg("-static");
        }

        if self.config.strip {
            cmd.arg("-s");
        }

        for lib_path in &self.config.library_paths {
            cmd.arg(format!("-L{}", lib_path.display()));
        }

        for lib in &self.config.libraries {
            cmd.arg(format!("-l{}", lib));
        }

        cmd.arg("-o");
        cmd.arg(&self.config.output);

        if self.config.verbose {
            println!("Linker command: {:?}", cmd);
        }

        let output = cmd.output().map_err(|e| LinkerError {
            message: format!("failed to execute linker: {}", e),
            command: Vec::new(),
            exit_code: None,
            stdout: String::new(),
            stderr: String::new(),
        })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
            let stdout = String::from_utf8_lossy(&output.stdout).into_owned();

            return Err(LinkerError {
                message: format!("linker exited with code {:?}", output.status.code()),
                command: Vec::new(),
                exit_code: output.status.code(),
                stdout,
                stderr,
            });
        }

        Ok(())
    }

    pub fn link_executable(&self) -> Result<(), LinkerError> {
        let mut config = self.config.clone();
        config.is_shared = false;
        config.is_static = false;

        let linker = Self { config };
        linker.link()
    }

    pub fn link_shared_library(&self) -> Result<(), LinkerError> {
        let mut config = self.config.clone();
        config.is_shared = true;
        config.is_static = false;

        let linker = Self { config };
        linker.link()
    }

    pub fn link_static_library(&self) -> Result<(), LinkerError> {
        let mut config = self.config.clone();
        config.is_shared = false;
        config.is_static = true;

        let linker = Self { config };
        linker.link()
    }
}

pub fn create_executable(input: PathBuf, output: PathBuf) -> Result<(), LinkerError> {
    let mut linker = Linker::new();
    linker.input(vec![input]);
    linker.output(output);
    linker.link_executable()
}

pub fn create_shared_library(input: PathBuf, output: PathBuf) -> Result<(), LinkerError> {
    let mut linker = Linker::new();
    linker.input(vec![input]);
    linker.output(output);
    linker.link_shared_library()
}

#[cfg(target_os = "linux")]
pub fn get_default_linker() -> String {
    String::from("gcc")
}

#[cfg(target_os = "macos")]
pub fn get_default_linker() -> String {
    String::from("clang")
}

#[cfg(target_os = "windows")]
pub fn get_default_linker() -> String {
    String::from("clang")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linker_config() {
        let config = LinkerConfig::default();
        assert_eq!(config.output, PathBuf::from("a.out"));
        assert!(config.input_files.is_empty());
    }

    #[test]
    fn test_linker_creation() {
        let linker = Linker::new();
        assert_eq!(linker.config.linker, "cc");
    }

    #[test]
    fn test_linker_fluent_api() {
        let linker = Linker::new()
            .output(PathBuf::from("test"))
            .input(vec![PathBuf::from("test.o")])
            .library_path(PathBuf::from("/usr/lib"))
            .library(String::from("m"))
            .static_linking();

        assert_eq!(linker.config.output, PathBuf::from("test"));
        assert_eq!(linker.config.input_files.len(), 1);
        assert!(linker.config.is_static);
    }
}
