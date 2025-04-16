use crate::config::ServiceConfig;
use anyhow::{anyhow, Error, Result};
use std::{
    fs,
    process::{Command, Stdio},
};
use std::fs::{File, Permissions};
use std::io::{Read, Write};
// use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::Output;
use log::info;

pub enum ServiceStatus {
    Running(u32, u16),
    Stopped,
    Error(Error),
}

pub struct ServiceManager {
    name: String,
    port: u16,
    config: ServiceConfig,
}

impl ServiceManager {
    pub fn new(name: &str, port: u16, config: ServiceConfig) -> Self {
        Self {
            name: name.to_string(),
            port,
            config,
        }
    }

    pub fn from_existing(name: &str) -> Result<Self> {
        let config = ServiceConfig::load(name, None)?;
        Ok(Self {
            name: name.to_string(),
            port: 0, // 从PID文件加载实际端口
            config,
        })
    }

    pub async fn start_daemon(&self) -> Result<(), Error> {
        let mut cmd = Command::new(&self.config.bin_path);
        cmd
            // .arg("--service-name")
            // .arg(&self.name)
            .arg("--port")
            .arg(self.port.to_string())
            .current_dir(&self.config.work_dir);

        let child = cmd
            .stdout(File::create(&self.config.log_file)?)
            .stderr(Stdio::inherit())
            .spawn()?;

        self.write_pid(child.id(), self.port).await?;

        let output = child.wait_with_output().expect("failed to wait on child");

        if output.status.success() {
            info!("✅ {} 启动成功", &self.name);
            Ok(())
        } else {
            Err(anyhow!("启动失败"))
        }
        // self.wait_ready().await?;
        // Ok(())
    }

    // fn wait_ready(&self) -> Result<()> {
    //     let client = reqwest::blocking::Client::new();
    //     let start_time = std::time::Instant::now();
    //     let timeout = Duration::from_secs(30);
    //
    //     loop {
    //         if start_time.elapsed() > timeout {
    //             return Err(anyhow::anyhow!("Service startup timeout"));
    //         }
    //
    //         match client.get(&self.health_check_url).send() {
    //             Ok(res) if res.status().is_success() => return Ok(()),
    //             _ => std::thread::sleep(Duration::from_millis(500)),
    //         }
    //     }
    // }

    pub async fn start_foreground(&self) -> Result<(), Error> {
        let mut cmd = Command::new(&self.config.bin_path);
        // 设置工作目录
        cmd.current_dir(&self.config.work_dir);
        // 添加环境变量
        match &self.config.custom_config {
            Some(path) => cmd.env("APP_CONFIG_PATH", path.as_path()),
            None => {
                log::warn!("Custom rconfig path not set, using default.");
                cmd.env("APP_CONFIG_PATH", Path::new("/default/path"))
            }
        };
        println!("Running command: {:?}", &cmd);

        // 直接运行前台进程并输出到标准输出和标准错误
        let mut child = cmd
            .stdout(Stdio::inherit())  // 标准输出直接显示在控制台
            .stderr(Stdio::inherit())  // 标准错误也直接显示在控制台
            .spawn()?;  // 启动进程

        self.write_pid(child.id(), self.port).await?;

        let status = child.wait()?;

        if !status.success() {
            // let mut stdout = String::new();
            // let mut stderr = String::new();
            // child.stdout.take().unwrap().read_to_string(&mut stdout)?;
            // child.stderr.take().unwrap().read_to_string(&mut stderr)?;
            // eprintln!("STDOUT: {}", stdout);
            // eprintln!("STDERR: {}", stderr);
            return Err(anyhow::anyhow!("进程失败，退出状态码: {}", status.code().unwrap_or(-1)).into());
        }
        Ok(())
    }

    async fn write_pid(&self, pid: u32, port: u16) -> Result<()> {
        let mut file = File::create(&self.config.pid_file)?;
        file.write_all(format!("{}:{}", pid, port).as_bytes())?;
        Ok(())
    }

    pub async fn stop(&self) -> Result<()> {
        let (pid, _) = self.read_pid().await?;

        Command::new("kill")
            .arg("-TERM")
            .arg(pid.to_string())
            .status()?;

        fs::remove_file(&self.config.pid_file)?;
        Ok(())
    }


    pub async fn restart(&self) -> Result<(), Error> {
        self.stop().await?;
        self.start_daemon().await?;
        Ok(())
    }

    async fn read_pid(&self) -> Result<(u32, u16)> {
        let contents = tokio::fs::read_to_string(&self.config.pid_file).await?;
        let mut parts = contents.split(':');
        let pid = parts.next().unwrap().parse()?;
        let port = parts.next().unwrap().parse()?;
        Ok((pid, port))
    }

    // 为每个服务实例创建独立的工作目录
    pub(crate) fn prepare_environment(&self) -> Result<(), Error> {
        fs::create_dir_all(&self.config.work_dir)?;
        // fs::set_permissions(&self.rconfig.work_dir, Permissions::from_mode(0o755))?;
        Ok(())
    }

    /// 获取所有服务实例
    pub fn list_services() -> Result<Vec<String>> {
        let run_dir = Path::new("/var/run");
        let mut services = Vec::new();

        for entry in fs::read_dir(run_dir)? {
            let path = entry?.path();
            if let Some(name) = path.file_name()
                .and_then(|n| n.to_str())
                .and_then(|n| n.strip_prefix("api-"))
                .and_then(|n| n.strip_suffix(".pid"))
            {
                services.push(name.to_string());
            }
        }
        Ok(services)
    }

    /// 验证服务名称有效性
    pub fn validate_name(name: &str) -> Result<()> {
        if name.is_empty() || name.len() > 64 {
            anyhow::bail!("Service name must be 1-64 characters");
        }

        if name.chars().any(|c| !c.is_ascii_alphanumeric() && c != '-') {
            anyhow::bail!("Service name contains invalid characters");
        }

        Ok(())
    }

    /// 检查服务状态
    pub async fn check_status(name: &str) -> ServiceStatus {
        let config = match ServiceConfig::load(name, None) {
            Ok(cfg) => cfg,
            Err(e) => return ServiceStatus::Error(anyhow!(format!("配置加载失败: {}", e))),
        };

        // 检查PID文件是否存在
        if !config.pid_file.exists() {
            return ServiceStatus::Stopped;
        }

        // 读取PID文件内容
        let pid_content = match tokio::fs::read_to_string(&config.pid_file).await {
            Ok(content) => content,
            Err(e) => return ServiceStatus::Error(anyhow!(format!("无法读取PID文件: {}", e))),
        };

        // 解析PID和端口
        let (pid, port) = match Self::parse_pid_content(&pid_content) {
            Some((p, port)) => (p, port),
            None => return ServiceStatus::Error(anyhow!("PID文件格式错误".to_string())),
        };

        // 检查进程是否在运行
        match Command::new("ps")
            .arg("-p")
            .arg(pid.to_string())
            .arg("-o")
            .arg("comm=")
            .output()
        {
            Ok(output) if output.status.success() => {
                let process_name = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !process_name.is_empty() {
                    ServiceStatus::Running(pid, port)
                } else {
                    ServiceStatus::Stopped
                }
            }
            Ok(_) => ServiceStatus::Stopped,
            Err(e) => ServiceStatus::Error(anyhow!(format!("进程检查失败: {}", e))),
        }
    }

    /// 解析PID文件内容
    fn parse_pid_content(content: &str) -> Option<(u32, u16)> {
        let mut parts = content.split(':');
        let pid = parts.next()?.parse().ok()?;
        let port = parts.next()?.parse().ok()?;
        Some((pid, port))
    }

}



#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_check_status_running() {
        let pid_file = NamedTempFile::new().unwrap();
        fs::write(pid_file.path(), "12345:8080").unwrap();

        let config = ServiceConfig {
            pid_file: pid_file.path().to_path_buf(),
            ..Default::default()
        };

        let status = ServiceManager::check_status("test").await;

        assert!(matches!(status, ServiceStatus::Running(12345, 8080 )));
    }

    #[tokio::test]
    async fn test_check_status_stopped() {
        let pid_file = NamedTempFile::new().unwrap();
        let config = ServiceConfig {
            pid_file: pid_file.path().to_path_buf(),
            ..Default::default()
        };

        let status = ServiceManager::check_status("test").await;

        assert!(matches!(status, ServiceStatus::Stopped));
    }

    #[tokio::test]
    async fn test_parse_pid_content() {
        let result = ServiceManager::parse_pid_content("12345:8080");
        assert_eq!(result, Some((12345, 8080)));

        let result = ServiceManager::parse_pid_content("invalid");
        assert_eq!(result, None);
    }
}