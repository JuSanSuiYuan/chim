use anyhow::{Context, Result};
use crate::config::PackageConfig;
use crate::utils;

pub async fn run(package: &str, dev: bool) -> Result<()> {
    let (name, version) = utils::parse_package_spec(package);
    let version = version.unwrap_or_else(|| "latest".to_string());

    utils::info(&format!(
        "添加{}依赖: {}@{}",
        if dev { "开发" } else { "" },
        name,
        version
    ));

    // 加载配置
    let mut config = PackageConfig::load()
        .context("无法加载 package.chim，请先运行 'cy init'")?;

    // 添加依赖
    config.add_dependency(name.clone(), version.clone(), dev);

    // 保存配置
    config.save("package.chim")?;

    utils::success(&format!("已添加 {}@{}", name, version));
    utils::info("运行 'cy install' 安装依赖");

    Ok(())
}
