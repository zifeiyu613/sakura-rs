mod config;
mod service;

use clap::{Parser, Subcommand};
use std::{env, path::PathBuf, process::exit, str};
use anyhow::{Context, Error};
use tokio::process::Command;
use log::{info, warn, error};
use crate::config::ServiceConfig;
use crate::service::{ServiceManager, ServiceStatus};

#[derive(Parser, Debug)]
#[command(name = "webctl")]
#[command(about = "CLI 工具用于管理 Web 服务", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// 启动 Web 服务
    Start {
        /// 服务名称
        #[arg(short, long)]
        name: String,

        /// 端口号（默认 8080）
        #[arg(short, long, default_value_t = 8080)]
        port: u16,

        /// 以守护进程模式运行
        #[arg(short, long, default_value_t = false)]
        daemon: bool,

        /// 配置文件路径
        #[arg(short, long)]
        config: Option<String>,
    },

    /// 停止 Web 服务
    Stop {
        /// 服务名称
        #[arg(short, long)]
        name: String,
    },

    /// 重启 Web 服务
    Restart {
        /// 服务名称
        #[arg(short, long)]
        name: String,
    },

    /// 查询 Web 服务状态
    Status {
        /// 服务名称
        #[arg(short, long)]
        name: String,
    },
}


/// **Web 服务管理工具**
///
/// ***usage：***
/// ```shell
/// # 启动服务（默认端口 8080）
/// cargo run -- start --name sakura-api --port 9090 --rconfig setting.toml
///
/// # 使用环境变量
/// export WEB_SERVICE_CONFIG=/etc/web_service.toml
/// cargo run -- start --name my_web_service --port 9000
///
/// # 停止服务
/// cargo run -- stop --name my_service
///
/// # 重启服务
/// cargo run -- restart --name my_service
///
/// # 查询服务状态
/// cargo run -- status --name my_service
/// ```
#[tokio::main]
async fn main() {
    env_logger::init();

    let cli = Cli::parse();

    if let Err(e) = match cli.command {
        Commands::Start { name, port, daemon, config } => {
            handle_start(&name, port, daemon, config).await.unwrap();
            Ok(())
        }
        Commands::Stop { name } => handle_stop(&name).await,
        Commands::Restart { name } => handle_restart(&name).await,
        Commands::Status { name } => handle_status(&name).await,
    } {
        error!("操作失败: {}", e);
        exit(1);
    }
}



/// 服务启动处理
async fn handle_start(name: &str, port: u16, daemon: bool, config: Option<String>) -> Result<(), Error> {
    let config = ServiceConfig::load(name, config)?;
    let service = ServiceManager::new(name, port, config);

    service.prepare_environment()
        .context("准备运行环境失败")?;

    if daemon {
        service.start_daemon().await
            .context("守护进程启动失败")?;
    } else {
        service.start_foreground().await
            .context("前台进程启动失败")?;
    }

    info!("✅ {} 启动成功 (端口: {})", name, port);
    Ok(())
}

/// 服务停止处理
async fn handle_stop(name: &str) -> Result<(), Error> {
    let service = ServiceManager::from_existing(name)
        .context("服务不存在")?;

    service.stop()
        .await
        .context("停止服务失败")?;

    info!("✅ {} 停止成功", name);
    Ok(())
}


/// 服务重启处理
async fn handle_restart(name: &str) -> Result<(), Error> {
    let service = ServiceManager::from_existing(name)
        .context("服务不存在")?;

    service.restart()
        .await
        .context("重启服务失败")?;

    info!("✅ {} 重启成功", name);
    Ok(())
}

/// 服务状态查询
async fn handle_status(name: &str) -> Result<(), Error> {
    match ServiceManager::check_status(name).await {
        ServiceStatus::Running(pid, port) => {
            println!("✅ {} 运行中 (PID: {}, 端口: {})", name, pid, port);
            Ok(())
        }
        ServiceStatus::Stopped => {
            println!("❌ {} 未运行", name);
            Ok(())
        }
        ServiceStatus::Error(e) => Err(e),
    }
}


//**************************************************************************************************





/// **获取配置路径**
fn get_config_path(config: Option<String>) -> PathBuf {
    if let Some(cfg) = config {
        PathBuf::from(cfg)
    } else {
        env::var("WEB_SERVICE_CONFIG")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("./rconfig.toml"))
    }
}


/// **启动服务**
async fn start_service(name: &str, port: u16, config_path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let output = Command::new("nohup")
        .arg(format!("./{}", name))
        .arg("--port")
        .arg(port.to_string())
        .arg("--rconfig")
        .arg(config_path.to_str().unwrap())
        .arg("&")
        .spawn()?
        .wait_with_output()
        .await?;

    if output.status.success() {
        info!("✅ {} 启动成功", name);
        Ok(())
    } else {
        Err("启动失败".into())
    }
}

/// **停止服务并获取端口号**
async fn stop_service(name: &str) -> Result<Option<u16>, Box<dyn std::error::Error>> {
    // 获取服务的 PID
    let output = Command::new("pgrep")
        .arg("-f")
        .arg(name)
        .output()
        .await?;

    if !output.status.success() {
        warn!("未找到服务: {}", name);
        return Ok(None);
    }

    let pid = str::from_utf8(&output.stdout)?.trim();
    info!("找到进程: {} (PID: {})", name, pid);

    // 通过 `lsof` 获取端口号
    let lsof_output = Command::new("lsof")
        .arg("-Pan")
        .arg("-p")
        .arg(pid)
        .arg("-i")
        .output()
        .await?;

    let lsof_str = str::from_utf8(&lsof_output.stdout)?;
    let port = extract_port_from_lsof(lsof_str);

    // 终止进程
    let kill_output = Command::new("kill")
        .arg("-9")
        .arg(pid)
        .spawn()?
        .wait_with_output()
        .await?;

    if kill_output.status.success() {
        info!("✅ {} 停止成功", name);
        Ok(port)
    } else {
        Err("停止失败".into())
    }
}

/// **解析 lsof 输出，获取端口号**
fn extract_port_from_lsof(lsof_output: &str) -> Option<u16> {
    for line in lsof_output.lines() {
        if line.contains("LISTEN") {
            if let Some(pos) = line.rfind(':') {
                if let Ok(port) = line[pos + 1..].trim().parse::<u16>() {
                    return Some(port);
                }
            }
        }
    }
    None
}

/// **重启服务**
async fn restart_service(name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let port = stop_service(name).await?;
    if let Some(port) = port {
        let config_path = get_config_path(None);
        start_service(name, port, &config_path).await?;
        Ok(())
    } else {
        Err("无法获取端口，重启失败".into())
    }
}

/// **查询服务状态**
async fn check_service_status(name: &str) -> Result<bool, Box<dyn std::error::Error>> {
    let output = Command::new("pgrep")
        .arg("-f")
        .arg(name)
        .output()
        .await?;

    Ok(output.status.success())
}
