use anyhow::Result;
use crate::utils;

pub async fn run(package: &str) -> Result<()> {
    utils::info(&format!("包信息: {}", package));

    // TODO: 从注册表获取包信息
    println!("\n包名: {}", package);
    println!("版本: 未知");
    println!("描述: 暂无");
    println!("主页: 暂无");
    println!("\n");

    utils::warning("info 命令尚未完全实现");

    Ok(())
}
