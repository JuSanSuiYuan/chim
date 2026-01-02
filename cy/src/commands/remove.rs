use anyhow::{Context, Result};
use crate::config::PackageConfig;
use crate::utils;

pub async fn run(package: &str) -> Result<()> {
    utils::info(&format!("移除依赖: {}", package));

    // 加载配置
    let mut config = PackageConfig::load()
        .context("无法加载 package.chim")?;

    // 移除依赖
    if config.remove_dependency(package) {
        // 保存配置
        config.save("package.chim")?;
        utils::success(&format!("已移除 {}", package));
        utils::info("运行 'cy install' 更新依赖");
    } else {
        utils::warning(&format!("未找到依赖: {}", package));
    }

    Ok(())
}
