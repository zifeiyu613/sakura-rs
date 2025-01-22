use serde::Deserialize;
use std::path::Path;
use std::fs;

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

impl RedisConfig {
    /// 从指定路径加载配置文件
    pub fn from_file(path: &str) -> Self {
        // let base_dir = env::current_dir().expect("Failed to get current directory");
        // let absolute_path = base_dir.join(path);
        println!("Using absolute path: {}", path);
        // 读取文件内容
        let config_str = fs::read_to_string(Path::new(path)).expect("Could not read RedisConfig file");
        // 使用 `toml` crate 解析配置
        let config: RedisConfig = toml::from_str(&config_str).expect("Failed to parse toml");
        config
    }

}