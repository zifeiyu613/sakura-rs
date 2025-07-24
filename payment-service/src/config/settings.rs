use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct AppSettings {
    pub database_url: String,
    pub server_host: String,
    pub server_port: u16,
    pub cache_ttl_seconds: u64,
    pub rate_limits: RateLimits,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RateLimits {
    pub default_rpm: u32,
    pub high_volume_rpm: u32,
}

impl AppSettings {
    pub fn from_env() -> Self {
        Self {
            database_url: std::env::var("DATABASE_URL")
                .unwrap_or_else(|_| "mysql://root:password@localhost/payment_service".to_string()),
            server_host: std::env::var("SERVER_HOST")
                .unwrap_or_else(|_| "0.0.0.0".to_string()),
            server_port: std::env::var("SERVER_PORT")
                .unwrap_or_else(|_| "3000".to_string())
                .parse()
                .unwrap_or(3000),
            cache_ttl_seconds: std::env::var("CACHE_TTL_SECONDS")
                .unwrap_or_else(|_| "300".to_string())
                .parse()
                .unwrap_or(300),
            rate_limits: RateLimits {
                default_rpm: std::env::var("RATE_LIMIT_DEFAULT_RPM")
                    .unwrap_or_else(|_| "100".to_string())
                    .parse()
                    .unwrap_or(100),
                high_volume_rpm: std::env::var("RATE_LIMIT_HIGH_VOLUME_RPM")
                    .unwrap_or_else(|_| "300".to_string())
                    .parse()
                    .unwrap_or(300),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_settings_from_env() {
        // 测试默认值
        let settings = AppSettings::from_env();
        assert_eq!(settings.server_port, 3000);
        assert_eq!(settings.cache_ttl_seconds, 300);
        assert_eq!(settings.rate_limits.default_rpm, 100);
        assert_eq!(settings.rate_limits.high_volume_rpm, 300);

        // 测试环境变量覆盖
        unsafe { 
            std::env::set_var("SERVER_PORT", "8080"); 
            std::env::set_var("CACHE_TTL_SECONDS", "600"); 
        }

        let settings = AppSettings::from_env();
        assert_eq!(settings.server_port, 8080);
        assert_eq!(settings.cache_ttl_seconds, 600);

        // 清理环境变量
        unsafe {
            std::env::remove_var("SERVER_PORT");
            std::env::remove_var("CACHE_TTL_SECONDS");
        }
        
    }
}