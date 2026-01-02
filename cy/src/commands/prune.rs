use anyhow::Result;
use crate::config::{LockFile, get_store_dir};
use crate::store::ContentStore;
use crate::utils;

pub async fn run() -> Result<()> {
    utils::info("清理未使用的包...");

    // 加载锁文件
    let lock = match LockFile::load() {
        Ok(lock) => lock,
        Err(_) => {
            utils::warning("未找到锁文件");
            return Ok(());
        }
    };

    // 获取所有使用中的包的哈希
    let used_packages: Vec<String> = lock
        .packages
        .values()
        .map(|p| p.integrity.clone())
        .collect();

    // 清理存储
    let store_dir = get_store_dir()?;
    let store = ContentStore::new(store_dir)?;
    let removed = store.prune(&used_packages)?;

    if removed > 0 {
        utils::success(&format!("已清理 {} 个未使用的包", removed));
    } else {
        utils::info("没有需要清理的包");
    }

    Ok(())
}
