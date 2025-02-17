use std::{env, fs};
use std::collections::HashMap;
use serde::Deserialize;


/// **单个数据库的配置**
#[derive(Debug, Deserialize, Clone)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub idle_timeout: u64,
}

/// **数据库配置结构体**
#[derive(Debug, Deserialize, Clone)]
pub struct DbConfig {
    pub databases: HashMap<String, DatabaseConfig>,
}


/// **固定配置文件路径（默认项目根目录 `config.toml`）**
const DEFAULT_CONFIG_PATH: &str = "config.toml";

impl DbConfig {
    /// 从指定路径加载配置文件
    pub fn load_config() -> Self {
        // 获取项目根目录下的 `config.toml`
        let config_path = env::var("APP_CONFIG_PATH").unwrap_or_else(|_|
            DEFAULT_CONFIG_PATH.to_string()
        );

        let config_content = fs::read_to_string(&config_path)
            .unwrap_or_else(|_| panic!("❌ 读取 `{}` 失败，请确保配置文件存在", &config_path));

        let parsed_config: DbConfig = toml::from_str(&config_content)
            .unwrap_or_else(|_| panic!("❌ 配置文件格式错误，请检查 `{}`", &config_path));

        println!("✅ 数据库配置已加载: `{}`", &config_path);
        parsed_config

        // if let Ok(config_content) = fs::read_to_string(&config_path) {
        //     if let Ok(parsed_config) = toml::from_str::<DbConfig>(&config_content) {
        //         println!("✅ 数据库配置已加载, {}", &config_path);
        //         parsed_config
        //     } else {
        //         panic!("❌ 配置文件格式错误，请检查 `{}`", &config_path);
        //     }
        // } else {
        //     panic!("❌ 读取 `{}` 失败，请确保配置文件存在", &config_path);
        // }

    }
}
