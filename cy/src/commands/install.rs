use anyhow::{Context, Result};
use std::fs;
use std::path::Path;
use indicatif::{ProgressBar, ProgressStyle};
use crate::config::{PackageConfig, LockFile, LockedPackage, get_store_dir};
use crate::dependency::DependencyResolver;
use crate::store::ContentStore;
use crate::utils;

pub async fn run() -> Result<()> {
    utils::info("安装依赖...");

    // 加载配置文件
    let config = PackageConfig::load()
        .context("无法加载 package.chim，请先运行 'cy init'")?;

    // 检查是否有依赖
    if config.dependencies.is_empty() && config.dev_dependencies.is_empty() {
        utils::info("没有需要安装的依赖");
        return Ok(());
    }

    // 解析依赖
    utils::info("解析依赖树...");
    let mut resolver = DependencyResolver::new();
    let resolved = resolver.resolve(&config).await?;

    utils::success(&format!("已解析 {} 个包", resolved.len()));

    // 初始化存储
    let store_dir = get_store_dir()?;
    let store = ContentStore::new(store_dir)?;

    // 创建node_modules目录
    let node_modules = Path::new("node_modules");
    fs::create_dir_all(node_modules)?;

    // 创建进度条
    let pb = ProgressBar::new(resolved.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}")
            .unwrap()
            .progress_chars("#>-"),
    );

    // 安装每个包
    let mut lock = LockFile::new();
    
    for (name, version) in &resolved {
        pb.set_message(format!("安装 {}@{}", name, version));
        
        // TODO: 实际下载包
        // 这里使用模拟数据
        let integrity = format!("sha256-{}{}", name, version);
        
        // 链接包到node_modules
        let package_dir = node_modules.join(name);
        // store.link_package(&integrity, &package_dir)?;
        
        // 添加到锁文件
        lock.packages.insert(
            name.clone(),
            LockedPackage {
                version: version.clone(),
                integrity: integrity.clone(),
                dependencies: Default::default(),
                resolved: Some(format!("https://registry.chim.dev/{}/{}", name, version)),
            },
        );
        
        pb.inc(1);
    }

    pb.finish_with_message("安装完成");

    // 保存锁文件
    lock.save("cy-lock.toml")?;

    utils::success(&format!("成功安装 {} 个包", resolved.len()));

    // 显示存储统计
    if let Ok(stats) = store.get_stats() {
        utils::info(&format!(
            "存储: {} 个包, 总大小 {}",
            stats.package_count,
            stats.format_size()
        ));
    }

    Ok(())
}
