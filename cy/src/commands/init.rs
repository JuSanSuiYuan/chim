use anyhow::{Context, Result};
use std::fs;
use std::io::{self, Write};
use crate::config::{PackageConfig, PackageInfo};
use crate::utils;

pub async fn run(name: Option<String>) -> Result<()> {
    utils::info("初始化Chim项目...");

    // 检查是否已存在配置文件
    if std::path::Path::new("package.chim").exists() {
        utils::warning("package.chim 已存在，是否覆盖？(y/N)");
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        if !input.trim().eq_ignore_ascii_case("y") {
            utils::info("已取消");
            return Ok(());
        }
    }

    // 获取项目名称
    let project_name = if let Some(name) = name {
        name
    } else {
        print!("项目名称: ");
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        input.trim().to_string()
    };

    // 获取版本
    print!("版本 (0.1.0): ");
    io::stdout().flush()?;
    let mut version = String::new();
    io::stdin().read_line(&mut version)?;
    let version = if version.trim().is_empty() {
        "0.1.0".to_string()
    } else {
        version.trim().to_string()
    };

    // 获取描述
    print!("描述: ");
    io::stdout().flush()?;
    let mut description = String::new();
    io::stdin().read_line(&mut description)?;
    let description = if description.trim().is_empty() {
        None
    } else {
        Some(description.trim().to_string())
    };

    // 创建配置
    let config = PackageConfig {
        package: PackageInfo {
            name: project_name.clone(),
            version,
            description,
            authors: vec![],
            license: Some("MIT".to_string()),
            homepage: None,
            keywords: vec![],
        },
        dependencies: Default::default(),
        dev_dependencies: Default::default(),
        scripts: Default::default(),
        repository: None,
    };

    // 保存配置
    config.save("package.chim")?;

    // 创建基本目录结构
    fs::create_dir_all("src")?;
    fs::create_dir_all("tests")?;

    // 创建主文件
    let main_content = r#"// Chim程序入口
print("Hello, Chim World!")

fn main() {
    print("欢迎使用Chim编程语言！")
}
"#;
    fs::write("src/main.chim", main_content)?;

    // 创建 .gitignore
    let gitignore_content = r#"# cy包管理器
node_modules/
.cy-cache/
cy-lock.toml

# 编译输出
*.wasm
*.wat
*.o
*.exe

# IDE
.vscode/
.idea/
*.swp
*.swo
*~
"#;
    fs::write(".gitignore", gitignore_content)?;

    utils::success(&format!("项目 '{}' 初始化成功！", project_name));
    utils::info("下一步：");
    println!("  cd {}", project_name);
    println!("  cy add <package>  # 添加依赖");
    println!("  cy install        # 安装依赖");

    Ok(())
}
