use std::{
    env,
    path::{Path, PathBuf},
};
use std::ops::Add;
use anyhow::{Context, Result};
use config::{Config, Environment, File};
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct ServiceConfig {
    pub work_dir: PathBuf,
    pub log_file: PathBuf,
    pub pid_file: PathBuf,
    pub bin_path: PathBuf,
    pub custom_config: Option<PathBuf>,
}

impl ServiceConfig {
    /// 配置加载优先级：
    /// 1. 默认配置文件 (/etc/webctl/default.toml)
    /// 2. 自定义配置文件 (如果提供)
    /// 3. 环境变量 (WEBCTL_*)
    pub fn load(name: &str, custom_path: Option<String>) -> Result<Self> {
        // 初始化配置构建器
        let mut config_builder = Config::builder();

        // 添加默认配置路径
        let default_config_path = Path::new("/Users/will/RustroverProjects/sakura/config.toml");
        if default_config_path.exists() {
            config_builder = config_builder.add_source(File::from(default_config_path));
        } else {
            log::warn!("默认配置文件不存在: {:?}", default_config_path);
        }

        // 添加自定义配置文件（如果提供）
        if let Some(custom_path) = &custom_path {
            let custom_path = PathBuf::from(custom_path);
            if custom_path.exists() {
                config_builder = config_builder.add_source(File::from(custom_path.clone()));
            } else {
                return Err(anyhow::anyhow!("自定义配置文件不存在: {:?}", custom_path).into());
            }
        }

        // 3. 添加环境变量配置，WEBCTL_为环境变量前缀
        // WEBCTL_NAME="webctl-service"
        // WEBCTL_PORT=8080
        // WEBCTL_LOG_FILE="/var/log/webctl/service.log"
        // WEBCTL_PID_FILE="/var/run/webctl/service.pid"
        // WEBCTL_BIN_PATH="/usr/local/bin/webctl"
        // WEBCTL_CUSTOM_CONFIG="/etc/webctl/custom_config.toml"
        config_builder = config_builder.add_source(Environment::with_prefix("WEBCTL"));

        // 4. 构建并解析配置文件
        let config = config_builder.build().context("无法加载配置文件")?;

        // 5. 尝试从配置文件获取 web_bin_path 字段的值
        let bin_path = config
            .get::<String>("service.bin_path")  // 尝试从配置文件中读取 web_bin_path 配置
            .with_context(|| "配置中未找到 bin_path")?;  // 如果没有找到，返回详细错误信息

        let sanitize_name = Self::sanitize_name(name);

        // 6. 如果没有找到 web_bin_path，可以提供默认值
        let bin_path = PathBuf::from(bin_path).join(&sanitize_name);

        let base_dir = config
            .get::<String>("service.base_dir")  // 尝试从配置文件中读取 web_bin_path 配置
            .with_context(|| "配置中未找到 base_dir")?;  // 如果没有找到，返回详细错误信息
        // 获取基础目录路径，使用服务名称生成独特目录
        // let base_dir = Path::new("/Users/will/.cargo/webctl")
        //     .join(&sanitize_name);

        let base_dir = PathBuf::from(base_dir).join(&sanitize_name);

        // 7. 返回配置实例
        Ok(Self {
            work_dir: base_dir.clone(),
            log_file: base_dir.clone().join("service.log"),
            pid_file: base_dir.join(PathBuf::from(format!("{}.pid", &sanitize_name))),
            bin_path,  // 使用从配置文件或默认值读取的 bin_path
            custom_config: Some(default_config_path.into()),
        })
    }

    /// 清理服务名称，确保服务名称符合文件系统的规范
    fn sanitize_name(name: &str) -> String {
        name.replace(|c: char| !c.is_ascii_alphanumeric(), "-") // 替换非法字符
            .trim_matches('-') // 去掉两端的 `-`
            .to_lowercase() // 转为小写
    }

    /// 获取实际使用的配置文件路径
    pub fn effective_config_path(&self) -> PathBuf {
        self.custom_config
            .clone()
            .unwrap_or_else(|| PathBuf::from("/etc/webctl/default.toml"))
    }
}