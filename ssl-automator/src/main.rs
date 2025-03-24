use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

use acme_lib::create_p384_key;
use acme_lib::persist::FilePersist;
use acme_lib::{Directory, DirectoryUrl};
use anyhow::{Context, Result};
use chrono::{DateTime, Duration as ChronoDuration, Utc};
use clap::{Parser, Subcommand};
use log::{debug, error, info, warn};
use openssl::x509::X509;
use sqlx::{Connection, SqliteConnection};
use thiserror::Error;
use tokio_cron_scheduler::{Job, JobScheduler};

#[derive(Error, Debug)]
enum SslError {
    #[error("证书验证失败: {0}")]
    ValidationFailed(String),

    #[error("证书处理失败: {0}")]
    CertificateError(String),

    #[error("IO错误: {0}")]
    IoError(#[from] io::Error),

    #[error("数据库错误: {0}")]
    DbError(#[from] sqlx::Error),

    #[error("ACME错误: {0}")]
    AcmeError(#[from] acme_lib::Error),
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// 配置文件路径
    #[arg(short, long, default_value = "~/.config/ssl-automator/config.json")]
    config: String,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// 立即检查并更新所有证书
    CheckNow,
    /// 添加新的域名
    AddDomain {
        /// 域名
        #[arg(long)]
        domain: String,
        /// Web 服务器根目录 (用于 HTTP 验证)
        #[arg(long)]
        webroot: String,
        /// 证书输出目录
        #[arg(long)]
        cert_dir: String,
        /// 重启 Web 服务器的命令
        #[arg(long)]
        restart_cmd: Option<String>,
    },
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
struct Config {
    /// 邮箱地址 (用于 Let's Encrypt 账户)
    email: String,
    /// 是否使用 Let's Encrypt 生产环境
    production: bool,
    /// 证书在多少天后需要更新
    renew_before_days: u64,
    /// 域名配置
    domains: Vec<DomainConfig>,
    /// 数据库路径
    db_path: String,
    /// ACME 账户持久化目录
    acme_dir: String,
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
struct DomainConfig {
    /// 域名
    domain: String,
    /// Web 服务器根目录，用于 HTTP 验证
    webroot: String,
    /// 证书输出目录
    cert_dir: String,
    /// 重启 Web 服务器的命令
    restart_cmd: Option<String>,
}

struct CertManager {
    config: Config,
    db: SqliteConnection,
}

impl CertManager {
    // 创建一个新的证书管理器实例
    async fn new(config: Config) -> Result<Self, anyhow::Error> {
        // 确保数据库目录存在
        let db_path = shellexpand::tilde(&config.db_path).to_string();
        let db_dir = Path::new(&db_path).parent().unwrap();
        fs::create_dir_all(db_dir)?;

        // 确保 ACME 账户目录存在
        let acme_dir = shellexpand::tilde(&config.acme_dir).to_string();
        fs::create_dir_all(&acme_dir)?;

        // 初始化数据库
        let mut conn = SqliteConnection::connect(&format!("sqlite:{}", db_path)).await?;
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS certificates (
                domain TEXT PRIMARY KEY,
                expiry TEXT NOT NULL,
                last_renewed TEXT NOT NULL
            )"
        ).execute(&mut conn).await?;

        Ok(CertManager {
            config,
            db: conn,
        })
    }

    // 克隆管理器（用于在异步任务中使用）
    async fn clone(&self) -> Result<Self, anyhow::Error> {
        let config = self.config.clone();
        let db = SqliteConnection::connect(&format!("sqlite:{}",
                                                    shellexpand::tilde(&config.db_path).to_string())).await?;

        Ok(CertManager {
            config,
            db,
        })
    }

    // 添加新域名并立即申请证书
    async fn add_domain(&mut self, domain: &str, webroot: &str, cert_dir: &str,
                        restart_cmd: Option<String>) -> Result<(), anyhow::Error> {
        // 检查域名是否已存在
        for existing in &self.config.domains {
            if existing.domain == domain {
                warn!("域名 {} 已存在，更新配置", domain);
                // 这里可以更新现有配置
                return Ok(());
            }
        }

        // 添加新域名到配置
        self.config.domains.push(DomainConfig {
            domain: domain.to_string(),
            webroot: webroot.to_string(),
            cert_dir: cert_dir.to_string(),
            restart_cmd: restart_cmd.clone(),
        });

        // 保存更新后的配置
        let config_path = shellexpand::tilde(&self.config.db_path)
            .to_string()
            .replace("certs.db", "config.json");
        let config_json = serde_json::to_string_pretty(&self.config)?;
        fs::write(&config_path, config_json)?;

        info!("已添加域名: {}", domain);

        // 立即尝试获取证书
        self.request_certificate(domain, webroot, cert_dir, restart_cmd).await?;

        Ok(())
    }

    // 检查并更新所有域名证书
    async fn check_and_renew_all(&mut self) -> Result<(), anyhow::Error> {
        let domains = self.config.domains.clone();
        for domain_config in domains {
            if let Err(e) = self.check_and_renew(&domain_config).await {
                error!("检查域名 {} 证书时出错: {}", domain_config.domain, e);
            }
        }
        Ok(())
    }

    // 检查并更新单个域名的证书
    async fn check_and_renew(&mut self, domain_config: &DomainConfig) -> Result<(), anyhow::Error> {
        let domain = &domain_config.domain;

        // 检查证书是否存在
        let cert_path = format!("{}/fullchain.pem", domain_config.cert_dir);
        let cert_exists = Path::new(&cert_path).exists();

        if !cert_exists {
            info!("证书不存在，为 {} 申请新证书", domain);
            self.request_certificate(
                domain,
                &domain_config.webroot,
                &domain_config.cert_dir,
                domain_config.restart_cmd.clone()
            ).await?;
            return Ok(());
        }

        // 检查证书过期时间
        let mut cert_file = File::open(&cert_path)?;
        let mut cert_data = Vec::new();
        cert_file.read_to_end(&mut cert_data)?;

        let cert = X509::from_pem(&cert_data)
            .map_err(|e| SslError::CertificateError(e.to_string()))?;

        let not_after = cert.not_after();
        let expiry = DateTime::parse_from_rfc2822(not_after.to_string().as_str())
            .map_err(|e| SslError::CertificateError(e.to_string()))?
            .with_timezone(&Utc);

        let now = Utc::now();
        let days_remaining = (expiry - now).num_days();

        info!("证书 {} 还有 {} 天过期", domain, days_remaining);

        if days_remaining <= self.config.renew_before_days as i64 {
            info!("证书接近过期，为 {} 更新证书", domain);
            self.request_certificate(
                domain,
                &domain_config.webroot,
                &domain_config.cert_dir,
                domain_config.restart_cmd.clone()
            ).await?;
        }

        Ok(())
    }

    // 请求新证书或更新现有证书
    async fn request_certificate(&mut self, domain: &str, webroot: &str, cert_dir: &str,
                                 restart_cmd: Option<String>) -> Result<(), anyhow::Error> {
        info!("为 {} 申请证书", domain);

        // 创建证书目录
        fs::create_dir_all(cert_dir)?;

        // 准备域名，包括 www 子域名
        let domains = vec![
            domain.to_string(),
            format!("www.{}", domain),
        ];

        // 创建 ACME 目录并获取账户
        let acme_dir = shellexpand::tilde(&self.config.acme_dir).to_string();
        let persist = FilePersist::new(&acme_dir);

        let dir_url = if self.config.production {
            DirectoryUrl::LetsEncrypt
        } else {
            DirectoryUrl::LetsEncryptStaging
        };

        let dir = Directory::from_url(persist, dir_url)?;
        let acc = dir.account(&self.config.email)?;

        // 开始新的订单
        let mut ord_new = acc.new_order(domain, &[])?;

        // 获取授权和HTTP挑战
        let ord_csr = loop {
            // 如果订单已经准备好，则跳出循环
            if let Some(ord_csr) = ord_new.confirm_validations() {
                break ord_csr;
            }

            // 获取授权列表
            let auths = ord_new.authorizations()?;

            // 处理每个域名授权
            for auth in auths {
                let challenge = auth.http_challenge();

                // 创建验证文件
                let token = challenge.http_token();
                let path = format!("{}/.well-known/acme-challenge/{}", webroot, token);

                // 确保挑战目录存在
                let challenge_dir = Path::new(&path).parent().unwrap();
                fs::create_dir_all(challenge_dir)?;

                // 写入挑战内容
                let proof = challenge.http_proof();
                let mut file = File::create(&path)?;
                file.write_all(proof.as_bytes())?;
                info!("已写入验证文件: {}", path);

                // 请求验证
                challenge.validate(5000)?;
                info!("已请求验证域名 {}", auth.domain_name());
            }

            // 等待验证完成
            tokio::time::sleep(Duration::from_secs(10)).await;

            // 更新订单状态
            ord_new.refresh()?;
        };

        // 生成私钥
        let pkey = create_p384_key();

        // 完成订单，获取证书
        let ord_cert = ord_csr.finalize_pkey(pkey, 5000)?;
        let cert = ord_cert.download_and_save_cert()?;

        // 写入证书文件
        fs::write(format!("{}/cert.pem", cert_dir), cert.certificate())?;
        fs::write(format!("{}/privkey.pem", cert_dir), cert.private_key())?;
        // fs::write(format!("{}/chain.pem", cert_dir), cert.certificate_chain())?;
        // fs::write(format!("{}/fullchain.pem", cert_dir), cert.fullchain_pem())?;

        info!("证书已保存到 {}", cert_dir);

        // 更新数据库记录
        let now = Utc::now().to_rfc3339();

        // 读取已下载的证书以获取到期时间
        let cert_data = fs::read(format!("{}/cert.pem", cert_dir))?;
        let x509 = X509::from_pem(&cert_data)
            .map_err(|e| SslError::CertificateError(e.to_string()))?;
        let not_after = x509.not_after();
        let expiry = DateTime::parse_from_rfc2822(not_after.to_string().as_str())
            .map_err(|e| SslError::CertificateError(e.to_string()))?
            .with_timezone(&Utc)
            .to_rfc3339();

        // 更新数据库
        sqlx::query(
            "INSERT OR REPLACE INTO certificates (domain, expiry, last_renewed) VALUES (?, ?, ?)"
        )
            .bind(domain)
            .bind(&expiry)
            .bind(&now)
            .execute(&mut self.db)
            .await?;

        // 执行重启命令（如果提供）
        if let Some(cmd) = restart_cmd {
            info!("重启 Web 服务器: {}", cmd);

            // 使用tokio的异步命令执行
            let output = tokio::process::Command::new("sh")
                .arg("-c")
                .arg(&cmd)
                .output()
                .await?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                warn!("重启命令失败: {}", stderr);
            } else {
                info!("Web 服务器重启成功");
            }
        }

        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    // 初始化日志
    env_logger::init();

    // 解析命令行参数
    let cli = Cli::parse();

    // 读取或创建配置文件
    let config_path = shellexpand::tilde(&cli.config).to_string();
    let config: Config = if Path::new(&config_path).exists() {
        let config_str = fs::read_to_string(&config_path)?;
        serde_json::from_str(&config_str)?
    } else {
        let config_dir = Path::new(&config_path).parent().unwrap();
        fs::create_dir_all(config_dir)?;

        // 创建默认配置
        let default_config = Config {
            email: "admin@example.com".to_string(),
            production: false,
            renew_before_days: 30,
            domains: Vec::new(),
            db_path: "~/.config/ssl-automator/certs.db".to_string(),
            acme_dir: "~/.config/ssl-automator/acme".to_string(),
        };

        let config_json = serde_json::to_string_pretty(&default_config)?;
        fs::write(&config_path, config_json)?;

        info!("已创建默认配置文件: {}", config_path);
        default_config
    };

    // 创建证书管理器
    let mut cert_manager = CertManager::new(config).await?;

    // 处理命令
    match &cli.command {
        Some(Commands::CheckNow) => {
            cert_manager.check_and_renew_all().await?;
            info!("证书检查完成");
        }
        Some(Commands::AddDomain { domain, webroot, cert_dir, restart_cmd }) => {
            cert_manager.add_domain(domain, webroot, cert_dir, restart_cmd.clone()).await?;
            info!("域名添加完成");
        }
        None => {
            // 创建定时任务调度器
            let scheduler = JobScheduler::new().await?;

            // 添加每日检查任务 (UTC 00:00)
            let task_config = cert_manager.config.clone();
            scheduler.add(
                Job::new_async("0 0 * * *", move |_uuid, _l| {
                    let config = task_config.clone();
                    Box::pin(async move {
                        match CertManager::new(config).await {
                            Ok(mut manager) => {
                                if let Err(e) = manager.check_and_renew_all().await {
                                    error!("自动证书检查失败: {}", e);
                                } else {
                                    info!("自动证书检查完成");
                                }
                            },
                            Err(e) => error!("创建证书管理器失败: {}", e),
                        }
                    })
                })?
            ).await?;

            // 启动调度器
            scheduler.start().await?;
            info!("SSL 自动化服务已启动，每天00:00(UTC)将自动检查证书");

            // 等待退出信号
            tokio::signal::ctrl_c().await?;
            info!("接收到终止信号，正在关闭...");
        }
    }

    Ok(())
}
