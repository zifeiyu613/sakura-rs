use std::collections::HashSet;
use crate::{AppConfig, ConfigError};

/// 配置验证器特质
pub trait ConfigValidator {
    fn validate(&self, config: &AppConfig) -> Result<(), ConfigError>;
}

/// 组合多个验证器
pub struct ValidatorChain {
    validators: Vec<Box<dyn ConfigValidator>>,
}

impl ValidatorChain {
    pub fn new() -> Self {
        Self { validators: Vec::new() }
    }

    pub fn add<V: ConfigValidator + 'static>(mut self, validator: V) -> Self {
        self.validators.push(Box::new(validator));
        self
    }

    pub fn validate(&self, config: &AppConfig) -> Result<(), ConfigError> {
        for validator in &self.validators {
            validator.validate(config)?;
        }
        Ok(())
    }
}

impl Default for ValidatorChain {
    fn default() -> Self {
        Self::new()
    }
}

/// 必需字段验证器
pub struct RequiredFieldsValidator {
    fields: HashSet<String>,
}

impl RequiredFieldsValidator {
    pub fn new() -> Self {
        Self { fields: HashSet::new() }
    }

    pub fn require<S: Into<String>>(mut self, field: S) -> Self {
        self.fields.insert(field.into());
        self
    }
}

impl ConfigValidator for RequiredFieldsValidator {
    fn validate(&self, config: &AppConfig) -> Result<(), ConfigError> {
        for field in &self.fields {
            let parts: Vec<&str> = field.split('.').collect();

            if !self.field_exists(config, &parts) {
                return Err(ConfigError::MissingKey(field.clone()));
            }
        }
        Ok(())
    }
}

impl RequiredFieldsValidator {
    fn field_exists(&self, config: &AppConfig, path: &[&str]) -> bool {
        if path.is_empty() {
            return true;
        }

        // 处理预设字段
        match path[0] {
            "service" => {
                if path.len() == 1 {
                    return true;
                }
                match path[1] {
                    "name" => !config.service.name.is_empty(),
                    "host" => !config.service.host.is_empty(),
                    "port" => true, // Port always has a default
                    "environment" => !config.service.environment.is_empty(),
                    "version" => config.service.version.is_some(),
                    _ => false,
                }
            },
            "database" => {
                if path.len() == 1 {
                    return !config.database.is_empty();
                }
                if path.len() == 2 {
                    return config.database.contains_key(path[1]);
                }
                // Check database specific fields
                if let Some(db_config) = config.database.get(path[1]) {
                    match path[2] {
                        "driver" => !db_config.driver.is_empty(),
                        "host" => !db_config.host.is_empty(),
                        "username" => !db_config.username.is_empty(),
                        "password" => !db_config.password.is_empty(),
                        "database" => !db_config.database.is_empty(),
                        _ => false,
                    }
                } else {
                    false
                }
            },
            "redis" => {
                if path.len() == 1 {
                    return config.redis.is_some();
                }
                if let Some(redis_config) = &config.redis {
                    match path[1] {
                        // Assume Redis config has fields like host, port, etc.
                        _ => false, // Implementation depends on your Redis config structure
                    }
                } else {
                    false
                }
            },
            _ => {
                // Check custom fields
                config.contains(path[0])
            }
        }
    }
}

/// 值范围验证器
pub struct RangeValidator {
    validations: Vec<RangeValidation>,
}

struct RangeValidation {
    field: String,
    min: Option<i64>,
    max: Option<i64>,
}

impl RangeValidator {
    pub fn new() -> Self {
        Self { validations: Vec::new() }
    }

    pub fn validate_range<S: Into<String>>(
        mut self,
        field: S,
        min: Option<i64>,
        max: Option<i64>
    ) -> Self {
        self.validations.push(RangeValidation {
            field: field.into(),
            min,
            max,
        });
        self
    }
}

impl ConfigValidator for RangeValidator {
    fn validate(&self, config: &AppConfig) -> Result<(), ConfigError> {
        for validation in &self.validations {
            // 使用Serde的反序列化功能提取值
            if let Some(value) = config.get::<i64>(&validation.field) {
                if let Some(min) = validation.min {
                    if value < min {
                        return Err(ConfigError::InvalidValue {
                            key: validation.field.clone(),
                            message: format!("Value {} is less than minimum {}", value, min),
                        });
                    }
                }

                if let Some(max) = validation.max {
                    if value > max {
                        return Err(ConfigError::InvalidValue {
                            key: validation.field.clone(),
                            message: format!("Value {} is greater than maximum {}", value, max),
                        });
                    }
                }
            }
        }
        Ok(())
    }
}

/// 环境验证器
pub struct EnvironmentValidator {
    allowed_environments: HashSet<String>,
}

impl EnvironmentValidator {
    pub fn new() -> Self {
        let mut validator = Self { allowed_environments: HashSet::new() };

        // 添加默认的环境
        validator.allowed_environments.insert("development".to_string());
        validator.allowed_environments.insert("test".to_string());
        validator.allowed_environments.insert("staging".to_string());
        validator.allowed_environments.insert("production".to_string());

        validator
    }

    pub fn add_environment<S: Into<String>>(mut self, env: S) -> Self {
        self.allowed_environments.insert(env.into());
        self
    }
}

impl ConfigValidator for EnvironmentValidator {
    fn validate(&self, config: &AppConfig) -> Result<(), ConfigError> {
        let env = &config.service.environment;

        if !self.allowed_environments.contains(env) {
            return Err(ConfigError::InvalidValue {
                key: "service.environment".to_string(),
                message: format!(
                    "Environment '{}' is not valid. Allowed: {:?}",
                    env,
                    self.allowed_environments
                ),
            });
        }

        Ok(())
    }
}

impl Default for EnvironmentValidator {
    fn default() -> Self {
        Self::new()
    }
}

// 添加一个便捷函数到ConfigBuilder
impl crate::ConfigBuilder {
    pub fn validate_with(self, validator: &ValidatorChain) -> Result<AppConfig, ConfigError> {
        let config = self.build()?;
        validator.validate(&config)?;
        Ok(config)
    }
}
