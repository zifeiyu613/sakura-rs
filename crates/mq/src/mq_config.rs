use std::{env, fs};
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct RabbitMQConfig {
    pub rabbit: MqConfig,

}

#[derive(Debug, Deserialize, Clone)]
pub struct MqConfig {
    pub uri: String,
    pub pool_max_size: usize,
}

const DEFAULT_CONFIG_PATH: &str = "config.toml";

impl RabbitMQConfig {

    /// 从指定路径加载配置文件
    pub fn load_config() -> Self {
        // 获取项目根目录下的 `config.toml`
        let config_path = env::var("APP_CONFIG_PATH").unwrap_or_else(|_|
            if fs::exists("mq_config.toml").is_ok() {
                "mq_config.toml".to_string()
            } else {
                DEFAULT_CONFIG_PATH.to_string()
            }
        );

        if let Ok(config_content) = fs::read_to_string(&config_path) {
            if let Ok(parsed_config) = toml::from_str::<RabbitMQConfig>(&config_content) {
                println!("✅ RabbitMQ 配置已加载, {}", &config_path);
                parsed_config
            } else {
                panic!("❌ 配置文件格式错误，请检查 `{}`", &config_path);
            }
        } else {
            panic!("❌ 读取 `{}` 失败，请确保配置文件存在", &config_path);
        }

    }
}