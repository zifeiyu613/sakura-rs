use crate::api::{home, landing_pages, pay_manage_handler};
use crate::config::Config;
use crate::infrastructure::database::DbManager;
use crate::middleware::{decryptor::decrypt, logger::log_request};
use axum::extract::State;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{middleware, Extension, Json, Router};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::sync::Arc;
use std::time::Duration;
use redis::aio::ConnectionManager;
use crate::errors::ApiError;
use crate::infrastructure::redis::client::RedisClient;
use crate::infrastructure::redis::lock::RedisLock;

#[derive(Clone, Debug)]
pub struct AppState {
    /// 配置
    pub config: Config,
    /// 数据库
    pub db_manager: DbManager,
}

pub async fn create_app() -> Result<Router, ApiError> {
    // 加载配置
    let config = Config::load().await?;

    // 初始化数据库连接
    let db_manager = DbManager::new(&config).await?;

    let redis_connect = init_redis(&config).await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    let redis_lock = RedisLock::new(redis_connect.clone())
        .with_prefix("app:v1:lock:");

    // 初始化其他服务...
    // let redis = init_redis(&rconfig).await?;
    // let amqp = init_rabbitmq(&rconfig).await?;

    let state = AppState {
        config,
        db_manager,
        // redis,
        // amqp,
    };

    let shared_state = Arc::new(state);

    // 创建一个设置扩展的中间件
    // let set_extensions = middleware::from_fn(move |mut req: Request, next: Next| {
    //     req.extensions_mut().insert(shared_state.clone());
    //     next.run(req)
    // });

    let yice_routes = Router::new()
        .nest("/home", home::routes(shared_state.clone()))
        .nest("/web", landing_pages::routes())
        .nest("/recharge", pay_manage_handler::routes());

    let router = Router::new()
        .route("/", get(|| async { "<h1>Hello, World!</h1>" }))
        .route("/test", get(handle_test).post(handle_test))
        .route("/test1", get(handle_test1).post(handle_test1))
        .nest_service("/yice", yice_routes)
        .layer(middleware::from_fn(log_request))
        .layer(middleware::from_fn(decrypt))
        .layer(Extension(shared_state.clone()))
        .with_state(shared_state);

    Ok(router)
}


async fn init_redis(config: &Config) -> crate::infrastructure::redis::error::Result<ConnectionManager> {
    // 创建Redis客户端
    let redis_client = RedisClient::builder(&config.redis.uri)
        .connection_timeout(Duration::from_secs(3))
        .build()?;

    // 测试Redis连接
    redis_client.ping().await?;

    // 获取连接管理器
    let connection = redis_client.get_connection_manager().await?;

    Ok(connection)
}



#[derive(Serialize, Deserialize, Debug, Clone, FromRow)]
struct TradeOrderArchivesRelation {
    id: u64,
    out_trade_no: Option<String>,
    mobile: Option<String>,
    device_code: Option<String>,
    username: String,
    update_time: DateTime<Utc>,
    create_time: DateTime<Utc>,
}

async fn handle_test(State(state): State<Arc<AppState>>) -> Result<impl IntoResponse, ApiError> {
    // 获取 sm_phoenix 数据库连接池
    let pool = state
        .db_manager
        .sm_phoenix()
        .ok_or_else(|| ApiError::Internal("sm_phoenix database not available".to_string()))?;

    let row: Option<TradeOrderArchivesRelation> = sqlx::query_as(r#"SELECT *  FROM t_trade_order_archives_relation WHERE id = ?"#)
        .bind(13674)
        .fetch_optional(pool)
        .await?;

    Ok(Json(row))

}

async fn handle_test1(Extension(state): Extension<Arc<AppState>>) -> Result<impl IntoResponse, ApiError> {
    // 获取 sm_phoenix 数据库连接池
    let pool = state
        .db_manager
        .sm_phoenix()
        .ok_or_else(|| ApiError::Internal("sm_phoenix database not available".to_string()))?;

    let row: Option<TradeOrderArchivesRelation> = sqlx::query_as(r#"SELECT *  FROM t_trade_order_archives_relation WHERE id = ?"#)
        .bind(13674)
        .fetch_optional(pool)
        .await?;

    Ok(Json(row))

}
