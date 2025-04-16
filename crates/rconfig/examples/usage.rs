use config::{
    AppConfig, ConfigBuilder, ConfigError,
    validation::{ValidatorChain, RequiredFieldsValidator, EnvironmentValidator},
    watcher::{ConfigChangeObserver, LoggingObserver},
    template::TemplateEngine,
    extension::PaymentConfig,
};
use std::time::Duration;

// 自定义配置变更观察者
struct DatabaseReconnector;

impl ConfigChangeObserver for DatabaseReconnector {
    fn on_config_changed(&self, old_config: &AppConfig, new_config: &AppConfig) {
        // 检查数据库配置是否变更
        if let (Some(old_db), Some(new_db)) = (old_config.main_database(), new_config.main_database()) {
            if old_db != new_db {
                println!("Database configuration changed, reconnecting...");
                // 在实际应用中，这里会执行数据库重连逻辑
            }
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // === 基本用法 ===

    // 1. 简单加载配置
    let simple_config = ConfigBuilder::default().build()?;
    println!("Service name: {}", simple_config.service_name());

    // === 高级用法 ===

    // 2. 使用模板引擎
    let mut engine = TemplateEngine::default();
    engine.set_variable("SERVICE_NAME", "my-awesome-service")
        .set_variable("ENV", "development")
        .set_variable("DB_HOST", "localhost")
        .set_variable("DB_PORT", "5432");

    // 3. 使用验证器
    let validator = ValidatorChain::default()
        .add(RequiredFieldsValidator::new()
            .require("service.name")
            .require("service.environment")
            .require("database.main"))
        .add(EnvironmentValidator::default());

    // 4. 自定义扩展配置
    let payment_config = PaymentConfig {
        api_key: "test_key".to_string(),
        api_secret: "test_secret".to_string(),
        endpoint: "https://api.payment.com".to_string(),
        timeout_secs: 30,
    };

    // 5. 构建配置并启用热重载
    let config_handle = ConfigBuilder::new()
        .with_default_config()
        .with_file("./rconfig/rconfig.yaml")
        .with_env_prefix("MYAPP")
        .with_cli_args()
        .with_remote("https://config-server.example.com/config/my-service")
        .with_extension_trait(payment_config)
        .with_template_engine(&engine)
        .validate(&validator)?
        .with_hot_reload()?
        .add_observer(LoggingObserver)
        .add_observer(DatabaseReconnector)
        .start();

    // 6. 获取当前配置
    let config = config_handle.get_config();
    println!("Service: {} ({})", config.service_name(), config.service.environment);

    if let Some(db) = config.main_database() {
        println!("Database: {}@{}:{}/{}",
                 db.username, db.host, db.port, db.database);
    }

    // 7. 访问自定义配置
    if let Some(payment: PaymentConfig) = config.get("payment") {
        println!("Payment API: {}", payment.endpoint);
    }

    // 8. 模拟应用运行，配置监控继续在后台进行
    println!("Application running. Configuration will be monitored for changes...");
    std::thread::sleep(Duration::from_secs(300)); // 运行5分钟

    // 9. 停止配置监控
    config_handle.stop()?;
    println!("Application shutting down");

    Ok(())
}
