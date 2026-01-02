use anyhow::{Context, Result};
use sha2::{Sha256, Digest};
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Content-Addressable Store（内容寻址存储）
/// 类似pnpm的存储机制，使用内容哈希作为存储地址
pub struct ContentStore {
    /// 存储根目录
    store_dir: PathBuf,
}

impl ContentStore {
    pub fn new(store_dir: PathBuf) -> Result<Self> {
        // 确保存储目录存在
        fs::create_dir_all(&store_dir)
            .context("无法创建存储目录")?;
        
        Ok(Self { store_dir })
    }

    /// 计算包的内容哈希
    pub fn compute_hash<P: AsRef<Path>>(&self, package_dir: P) -> Result<String> {
        let mut hasher = Sha256::new();
        
        // 遍历包目录下的所有文件
        for entry in WalkDir::new(package_dir.as_ref())
            .follow_links(false)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            // 读取文件内容
            let content = fs::read(entry.path())
                .context(format!("无法读取文件: {}", entry.path().display()))?;
            
            // 更新哈希
            hasher.update(&content);
            
            // 同时哈希文件路径（相对路径）
            if let Ok(rel_path) = entry.path().strip_prefix(package_dir.as_ref()) {
                hasher.update(rel_path.to_string_lossy().as_bytes());
            }
        }
        
        // 返回十六进制哈希值
        Ok(hex::encode(hasher.finalize()))
    }

    /// 将包存储到content-addressable store
    pub fn store_package<P: AsRef<Path>>(&self, package_dir: P, integrity: &str) -> Result<PathBuf> {
        let store_path = self.get_package_path(integrity);
        
        // 如果已存在，直接返回
        if store_path.exists() {
            return Ok(store_path);
        }
        
        // 创建存储目录
        fs::create_dir_all(&store_path)
            .context("无法创建包存储目录")?;
        
        // 复制包内容到存储目录
        self.copy_dir_all(package_dir.as_ref(), &store_path)?;
        
        Ok(store_path)
    }

    /// 通过硬链接创建包的本地副本
    pub fn link_package(&self, integrity: &str, target_dir: &Path) -> Result<()> {
        let store_path = self.get_package_path(integrity);
        
        if !store_path.exists() {
            anyhow::bail!("包不存在于存储中: {}", integrity);
        }
        
        // 确保目标目录存在
        fs::create_dir_all(target_dir)
            .context("无法创建目标目录")?;
        
        // 遍历存储目录，为每个文件创建硬链接
        for entry in WalkDir::new(&store_path)
            .follow_links(false)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            let rel_path = entry.path().strip_prefix(&store_path)
                .context("无法获取相对路径")?;
            let target_path = target_dir.join(rel_path);
            
            // 创建父目录
            if let Some(parent) = target_path.parent() {
                fs::create_dir_all(parent)?;
            }
            
            // 创建硬链接
            if target_path.exists() {
                fs::remove_file(&target_path)?;
            }
            
            #[cfg(unix)]
            std::os::unix::fs::hard_link(entry.path(), &target_path)
                .context(format!("无法创建硬链接: {}", target_path.display()))?;
            
            #[cfg(windows)]
            std::os::windows::fs::hard_link(entry.path(), &target_path)
                .context(format!("无法创建硬链接: {}", target_path.display()))?;
        }
        
        Ok(())
    }

    /// 获取包在存储中的路径
    fn get_package_path(&self, integrity: &str) -> PathBuf {
        // 使用哈希的前2个字符作为子目录，类似git的对象存储
        let prefix = &integrity[..2];
        let suffix = &integrity[2..];
        self.store_dir.join(prefix).join(suffix)
    }

    /// 递归复制目录
    fn copy_dir_all(&self, src: &Path, dst: &Path) -> Result<()> {
        fs::create_dir_all(dst)?;
        
        for entry in fs::read_dir(src)? {
            let entry = entry?;
            let ty = entry.file_type()?;
            let dst_path = dst.join(entry.file_name());
            
            if ty.is_dir() {
                self.copy_dir_all(&entry.path(), &dst_path)?;
            } else {
                fs::copy(entry.path(), dst_path)?;
            }
        }
        
        Ok(())
    }

    /// 清理未使用的包
    pub fn prune(&self, used_packages: &[String]) -> Result<usize> {
        let mut removed_count = 0;
        
        // 遍历存储目录
        for entry in fs::read_dir(&self.store_dir)? {
            let entry = entry?;
            if !entry.file_type()?.is_dir() {
                continue;
            }
            
            let prefix_dir = entry.path();
            for sub_entry in fs::read_dir(&prefix_dir)? {
                let sub_entry = sub_entry?;
                let hash = format!("{}{}", 
                    prefix_dir.file_name().unwrap().to_string_lossy(),
                    sub_entry.file_name().to_string_lossy()
                );
                
                // 如果不在使用列表中，删除
                if !used_packages.contains(&hash) {
                    fs::remove_dir_all(sub_entry.path())?;
                    removed_count += 1;
                }
            }
        }
        
        Ok(removed_count)
    }

    /// 获取存储统计信息
    pub fn get_stats(&self) -> Result<StoreStats> {
        let mut package_count = 0;
        let mut total_size = 0u64;
        
        for entry in WalkDir::new(&self.store_dir)
            .follow_links(false)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.file_type().is_file() {
                total_size += entry.metadata()?.len();
            } else if entry.file_type().is_dir() && entry.depth() == 2 {
                package_count += 1;
            }
        }
        
        Ok(StoreStats {
            package_count,
            total_size,
        })
    }
}

/// 存储统计信息
#[derive(Debug)]
pub struct StoreStats {
    /// 包数量
    pub package_count: usize,
    /// 总大小（字节）
    pub total_size: u64,
}

impl StoreStats {
    /// 格式化大小为人类可读格式
    pub fn format_size(&self) -> String {
        const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
        let mut size = self.total_size as f64;
        let mut unit_idx = 0;
        
        while size >= 1024.0 && unit_idx < UNITS.len() - 1 {
            size /= 1024.0;
            unit_idx += 1;
        }
        
        format!("{:.2} {}", size, UNITS[unit_idx])
    }
}
