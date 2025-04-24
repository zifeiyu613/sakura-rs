//! 系统设置相关常量

/// 密码加密迭代次数
pub const PASSWORD_HASH_ITERATIONS: u32 = 10000;

/// 令牌有效期(秒)
pub const TOKEN_VALIDITY_SECONDS: u64 = 86400; // 24小时

/// 验证码有效期(秒)
pub const VERIFICATION_CODE_TTL: u64 = 300; // 5分钟

/// 最大登录失败次数
pub const MAX_LOGIN_ATTEMPTS: u32 = 5;

/// 临时文件目录
pub const TEMP_DIRECTORY: &str = "/tmp/app_files";