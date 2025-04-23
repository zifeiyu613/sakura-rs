use std::sync::Arc;
use rdatabase::DbPool;


#[derive(Debug, Clone)]
pub struct AppState {

    pub db: DbPool,
    // pub redis: Arc<RedisClient>,
    // pub rabbit: Arc<RabbitConnection>,
    
    pub app_config: Arc<rconfig::AppConfig>,
    
}