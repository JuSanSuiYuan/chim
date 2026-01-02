use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// package.chim配置文件结构（TOML格式）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageConfig {
    /// 包元信息
    pub package: PackageInfo,
    /// 依赖列表
    #[serde(default)]
    pub dependencies: HashMap<String, String>,
    /// 开发依赖列表
    #[serde(default, rename = "dev-dependencies")]
    pub dev_dependencies: HashMap<String, String>,
    /// 构建脚本
    #[serde(default)]
    pub scripts: HashMap<String, String>,
    /// 仓库配置
    #[serde(default)]
    pub repository: Option<RepositoryInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageInfo {
    /// 包名
    pub name: String,
    /// 版本
    pub version: String,
    /// 描述
    #[serde(default)]
    pub description: Option<String>,
    /// 作者列表
    #[serde(default)]
    pub authors: Vec<String>,
    /// 许可证
    #[serde(default)]
    pub license: Option<String>,
    /// 主页
    #[serde(default)]
    pub homepage: Option<String>,
    /// 关键词
    #[serde(default)]
    pub keywords: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryInfo {
    /// 仓库类型（git, http等）
    #[serde(rename = "type")]
    pub repo_type: String,
    /// 仓库URL
    pub url: String,
}

/// cy-lock.toml锁文件结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockFile {
    /// 锁文件版本
    pub version: String,
    /// 锁定的包信息
    pub packages: HashMap<String, LockedPackage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockedPackage {
    /// 包版本
    pub version: String,
    /// 包的内容哈希（用于content-addressable存储）
    pub integrity: String,
    /// 依赖列表
    #[serde(default)]
    pub dependencies: HashMap<String, String>,
    /// 包的下载地址
    #[serde(default)]
    pub resolved: Option<String>,
}

impl PackageConfig {
    /// 从文件加载配置
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = fs::read_to_string(&path)
            .context(format!("无法读取配置文件: {}", path.as_ref().display()))?;
        
        let config: PackageConfig = toml::from_str(&content)
            .context("无法解析配置文件")?;
        
        Ok(config)
    }

    /// 从当前目录加载配置
    pub fn load() -> Result<Self> {
        Self::from_file("package.chim")
    }

    /// 保存配置到文件
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let content = toml::to_string_pretty(self)
            .context("无法序列化配置")?;
        
        fs::write(&path, content)
            .context(format!("无法写入配置文件: {}", path.as_ref().display()))?;
        
        Ok(())
    }

    /// 添加依赖
    pub fn add_dependency(&mut self, name: String, version: String, dev: bool) {
        if dev {
            self.dev_dependencies.insert(name, version);
        } else {
            self.dependencies.insert(name, version);
        }
    }

    /// 移除依赖
    pub fn remove_dependency(&mut self, name: &str) -> bool {
        self.dependencies.remove(name).is_some() || 
        self.dev_dependencies.remove(name).is_some()
    }
}

impl LockFile {
    /// 从文件加载锁文件
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = fs::read_to_string(&path)
            .context(format!("无法读取锁文件: {}", path.as_ref().display()))?;
        
        let lock: LockFile = toml::from_str(&content)
            .context("无法解析锁文件")?;
        
        Ok(lock)
    }

    /// 从当前目录加载锁文件
    pub fn load() -> Result<Self> {
        Self::from_file("cy-lock.toml")
    }

    /// 保存锁文件
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let content = toml::to_string_pretty(self)
            .context("无法序列化锁文件")?;
        
        fs::write(&path, content)
            .context(format!("无法写入锁文件: {}", path.as_ref().display()))?;
        
        Ok(())
    }

    /// 创建新的空锁文件
    pub fn new() -> Self {
        Self {
            version: "1.0".to_string(),
            packages: HashMap::new(),
        }
    }
}

impl Default for LockFile {
    fn default() -> Self {
        Self::new()
    }
}

/// 获取cy的全局配置目录
pub fn get_cy_home() -> Result<PathBuf> {
    let home = dirs::home_dir()
        .context("无法获取用户主目录")?;
    
    Ok(home.join(".cy"))
}

/// 获取cy的全局存储目录（类似pnpm的store）
pub fn get_store_dir() -> Result<PathBuf> {
    Ok(get_cy_home()?.join("store"))
}

/// 获取cy的全局缓存目录
pub fn get_cache_dir() -> Result<PathBuf> {
    Ok(get_cy_home()?.join("cache"))
}
