use serde::Deserialize;
use std::{env, fs};

#[derive(Debug, Deserialize)]
pub struct RedisConfig {

    /// The URI for connecting to the Redis server. For example:
    /// <redis://127.0.0.1/>
    pub uri: String,


    pub pool_max_size: usize,

    /// The new connection will time out operations after `response_timeout` has passed.
    pub response_timeout: Option<u64>,

    /// Each connection attempt to the server will time out after `connection_timeout`.
    pub connection_timeout: Option<u64>,

    /// number_of_retries times, with an exponentially increasing delay
    pub number_of_retries: Option<usize>,

    /// The resulting duration is calculated by taking the base to the `n`-th power,
    /// where `n` denotes the number of past attempts.
    pub exponent_base: Option<u64>,

    /// A multiplicative factor that will be applied to the retry delay.
    ///
    /// For example, using a factor of `1000` will make each delay in units of seconds.
    pub factor: Option<u64>,

    /// Apply a maximum delay between connection attempts. The delay between attempts won't be longer than max_delay milliseconds.
    pub max_delay: Option<u64>,

}

const DEFAULT_CONFIG_PATH: &str = "config.toml";

impl RedisConfig {

    /// 从指定路径加载配置文件
    pub fn load_config() -> Self {
        // 获取项目根目录下的 `config.toml`
        let config_path = env::var("APP_CONFIG_PATH").unwrap_or_else(|_|
            if fs::exists("redis_config.toml").is_ok() {
                "redis_config.toml".to_string()
            } else {
                DEFAULT_CONFIG_PATH.to_string()
            }
        );

        if let Ok(config_content) = fs::read_to_string(&config_path) {
            if let Ok(parsed_config) = toml::from_str::<RedisConfig>(&config_content) {
                println!("✅ Redis配置已加载, {}", &config_path);
                parsed_config
            } else {
                panic!("❌ 配置文件格式错误，请检查 `{}`", &config_path);
            }
        } else {
            panic!("❌ 读取 `{}` 失败，请确保配置文件存在", &config_path);
        }

    }

}