use std::fs;
use std::path::Path;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct DbConfig {
    pub phoenix: Option<DatabaseConfig>,
    pub huajian_activity: Option<DatabaseConfig>,
    pub huajian_live: Option<DatabaseConfig>,
}

#[derive(Debug, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub idle_timeout: u64,
}


impl DbConfig {
    /// 从指定路径加载配置文件
    pub fn from_file(path: &str) -> Self {
        // 读取文件内容
        let config_str = fs::read_to_string(Path::new(path)).expect("Could not read DatabaseConfig file");
        // 使用 `toml` crate 解析配置
        let config: DbConfig = toml::from_str(&config_str).expect("Failed to parse toml");
        config
    }
}
