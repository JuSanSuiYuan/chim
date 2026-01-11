// ==================== 路径模块 ====================
// 提供跨平台的路径操作功能

pub mod path {
    use crate::stdlib::prelude::Option;
    use crate::stdlib::string::String;

    #[derive(Debug, Clone, PartialEq)]
    pub struct Path {
        inner: string,
    }

    impl Path {
        pub fn new(path: string) -> Path {
            let normalized = normalize_path(&path);
            Path { inner: normalized }
        }

        pub fn as_str(&self) -> string {
            self.inner.clone()
        }

        pub fn to_string(&self) -> string {
            self.inner.clone()
        }

        pub fn file_name(&self) -> Option<string> {
            let parts = split_path(&self.inner);
            if parts.is_empty() {
                None
            } else {
                Some(parts.last().unwrap().clone())
            }
        }

        pub fn parent(&self) -> Option<Path> {
            let parts = split_path(&self.inner);
            if parts.len() <= 1 {
                if is_absolute_path(&self.inner) {
                    None
                } else {
                    Some(Path::new(".".to_string()))
                }
            } else {
                let parent = parts[..parts.len() - 1].join("/");
                Some(Path::new(parent))
            }
        }

        pub fn extension(&self) -> Option<string> {
            let name = self.file_name()?;
            let pos = name.rfind('.');
            if pos.is_some() {
                let p = pos.unwrap();
                if p > 0 && p < name.len() - 1 {
                    return Some(name[p + 1..].to_string());
                }
            }
            None
        }

        pub fn stem(&self) -> Option<string> {
            let name = self.file_name()?;
            let pos = name.rfind('.');
            if pos.is_some() {
                let p = pos.unwrap();
                return Some(name[..p].to_string());
            }
            Some(name)
        }

        pub fn is_absolute(&self) -> bool {
            is_absolute_path(&self.inner)
        }

        pub fn is_relative(&self) -> bool {
            !self.is_absolute()
        }

        pub fn join(&self, other: &Path) -> Path {
            if self.inner.is_empty() {
                return other.clone();
            }
            if other.inner.is_empty() {
                return self.clone();
            }
            if self.inner.ends_with('/') {
                Path::new(self.inner.clone() + &other.inner)
            } else {
                Path::new(self.inner.clone() + "/" + &other.inner)
            }
        }

        pub fn with_extension(&self, ext: &str) -> Path {
            let stem = self.stem().unwrap_or("".to_string());
            Path::new(stem + "." + ext)
        }

        pub fn exists(&self) -> bool {
            self.metadata().is_ok()
        }

        pub fn is_file(&self) -> bool {
            match self.metadata() {
                Ok(m) => m.is_file,
                Err(_) => false,
            }
        }

        pub fn is_dir(&self) -> bool {
            match self.metadata() {
                Ok(m) => m.is_dir,
                Err(_) => false,
            }
        }

        pub fn metadata(&self) -> Result<Metadata, Error> {
            get_metadata(self)
        }

        pub fn canonicalize(&self) -> Result<Path, Error> {
            if !self.exists() {
                return Err(Error::new("path does not exist"));
            }
            Ok(self.clone())
        }
    }

    fn normalize_path(path: &string) -> string {
        let mut result = path.clone();
        result = result.replace("\\", "/");
        while result.contains("//") {
            result = result.replace("//", "/");
        }
        result
    }

    fn split_path(path: &string) -> Vec<string> {
        if path.is_empty() {
            return Vec::new();
        }
        path.split('/').filter(|s| !s.is_empty()).collect()
    }

    fn is_absolute_path(path: &string) -> bool {
        if path.starts_with('/') {
            return true;
        }
        if path.len() >= 2 && path.chars().nth(1).unwrap() == ':' {
            return true;
        }
        false
    }

    #[derive(Debug, Clone)]
    pub struct Metadata {
        pub len: int,
        pub is_dir: bool,
        pub is_file: bool,
        pub modified: i64,
    }

    #[derive(Debug, Clone)]
    pub struct Error {
        message: string,
    }

    impl Error {
        pub fn new(msg: string) -> Error {
            Error { message: msg }
        }
    }

    pub type Result<T> = std::result::Result<T, Error>;

    fn get_metadata(p: &Path) -> Result<Metadata> {
        let is_dir = p.inner.ends_with('/') || p.inner == ".";
        let is_file = !is_dir && p.inner.contains('.');
        Ok(Metadata {
            len: 0,
            is_dir,
            is_file,
            modified: 0,
        })
    }

    pub fn home_dir() -> Option<Path> {
        Some(Path::new(std::env::home_dir().unwrap_or(".".to_string())))
    }

    pub fn current_dir() -> Path {
        Path::new(".".to_string())
    }
}
