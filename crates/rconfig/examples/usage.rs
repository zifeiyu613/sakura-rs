use rconfig::AppConfig;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 加载配置
    let config = AppConfig::new()
        .add_default("config/default")
        .add_environment()
        .build()?;

    // 使用默认数据库配置（向后兼容方式）
    let default_db_url = config.database().connection_url()?;
    println!("默认数据库: {}", default_db_url);

    // 使用多数据源功能获取特定数据库
    if let Some(read_db) = config.get_database(Some("read")) {
        let read_db_url = read_db.connection_url()?;
        println!("读库: {}", read_db_url);
    }

    if let Some(analytics_db) = config.get_database(Some("analytics")) {
        let analytics_db_url = analytics_db.connection_url()?;
        println!("分析数据库: {}", analytics_db_url);
    }

    // 获取所有配置的数据库名称
    println!("所有配置的数据库: {:?}", config.database_names());

    // 迭代所有数据库配置
    for (name, _) in config.databases.iter() {
        println!("配置了数据库: {}", name);
    }

    Ok(())
}

// 自定义配置结构
#[derive(serde::Deserialize)]
struct PaymentConfig {
    gateway: String,
    api_key: String,
    timeout: u64,
}

// 设置日志
fn setup_logging(log_config: &rconfig::LogConfig) -> Result<(), Box<dyn std::error::Error>> {
    // 这里使用日志配置初始化日志系统
    // ...
    Ok(())
}
