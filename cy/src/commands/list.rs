use anyhow::Result;
use crate::config::LockFile;
use crate::utils;

pub async fn run() -> Result<()> {
    utils::info("已安装的包:");

    // 加载锁文件
    let lock = match LockFile::load() {
        Ok(lock) => lock,
        Err(_) => {
            utils::warning("未找到锁文件，请先运行 'cy install'");
            return Ok(());
        }
    };

    if lock.packages.is_empty() {
        utils::info("没有已安装的包");
        return Ok(());
    }

    // 显示包列表
    println!();
    for (name, pkg) in &lock.packages {
        println!("  {} @ {}", name, pkg.version);
        if !pkg.dependencies.is_empty() {
            for (dep_name, dep_version) in &pkg.dependencies {
                println!("    └─ {} @ {}", dep_name, dep_version);
            }
        }
    }
    println!();

    utils::success(&format!("共 {} 个包", lock.packages.len()));

    Ok(())
}
