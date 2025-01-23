use std::fs;
use std::path::Path;

pub struct DatabaseConfig {
    pub phoenix_config: PhoenixConfig,
    pub activity_config: ActivityConfig,
}

pub struct PhoenixConfig {
    pub database_url: String,
    pub max_connections: u32,
    pub timeout: u64,
}

pub struct ActivityConfig {
    pub database_url: String,
    pub max_connections: u32,
    pub timeout: u64,
}


impl DatabaseConfig {
    /// 从指定路径加载配置文件
    pub fn from_file(path: &str) -> Self {
        // 读取文件内容
        let config_str = fs::read_to_string(Path::new(path)).expect("Could not read DatabaseConfig file");
        // 使用 `toml` crate 解析配置
        let config: DatabaseConfig = toml::from_str(&config_str).expect("Failed to parse toml");
        config
    }
}
