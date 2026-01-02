mod config;
mod dependency;
mod store;
mod commands;
mod utils;

use clap::{Parser, Subcommand};
use anyhow::Result;

#[derive(Parser)]
#[command(name = "cy")]
#[command(about = "cy - Chim编程语言包管理器", long_about = None)]
#[command(version = "0.1.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 初始化一个新的Chim项目
    Init {
        /// 项目名称
        name: Option<String>,
    },
    /// 安装所有依赖
    Install,
    /// 添加一个依赖
    Add {
        /// 包名（格式: name@version）
        package: String,
        /// 作为开发依赖添加
        #[arg(short, long)]
        dev: bool,
    },
    /// 移除一个依赖
    Remove {
        /// 包名
        package: String,
    },
    /// 更新依赖
    Update {
        /// 包名（如果不指定则更新所有依赖）
        package: Option<String>,
    },
    /// 清理未使用的包
    Prune,
    /// 显示包信息
    Info {
        /// 包名
        package: String,
    },
    /// 列出所有已安装的包
    List,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init { name } => commands::init::run(name).await,
        Commands::Install => commands::install::run().await,
        Commands::Add { package, dev } => commands::add::run(&package, dev).await,
        Commands::Remove { package } => commands::remove::run(&package).await,
        Commands::Update { package } => commands::update::run(package.as_deref()).await,
        Commands::Prune => commands::prune::run().await,
        Commands::Info { package } => commands::info::run(&package).await,
        Commands::List => commands::list::run().await,
    }
}
