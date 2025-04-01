use crate::config::Config;
use crate::error::YiceError;
use crate::infrastructure::database::DbManager;
use axum::{Json, Router};
use axum::extract::State;
use axum::response::IntoResponse;
use axum::routing::get;
use std::sync::Arc;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::{FromRow, Type};

#[derive(Clone)]
pub struct AppState {
    /// 配置
    pub config: Config,
    /// 数据库
    pub db_manager: DbManager,
}

pub async fn create_app() -> Result<Router, YiceError> {
    // 加载配置
    let config = Config::load().await?;

    // 初始化数据库连接
    let db_manager = DbManager::new(&config).await?;

    // 初始化其他服务...
    // let redis = init_redis(&config).await?;
    // let amqp = init_rabbitmq(&config).await?;

    let state = AppState {
        config,
        db_manager,
        // redis,
        // amqp,
    };

    let shared_state = Arc::new(state);

    let router = Router::new()
        .route("/", get(|| async { "<h1>Hello, World!</h1>" }))
        .route("/test", get(handle_test))
        .with_state(shared_state);

    Ok(router)
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

async fn handle_test(State(state): State<Arc<AppState>>) -> Result<impl IntoResponse, YiceError> {
    // 获取 sm_phoenix 数据库连接池
    let pool = state
        .db_manager
        .sm_phoenix()
        .ok_or_else(|| YiceError::Internal("sm_phoenix database not available".to_string()))?;

    let row: Option<TradeOrderArchivesRelation> = sqlx::query_as(r#"SELECT *  FROM t_trade_order_archives_relation WHERE id = ?"#)
        .bind(13674)
        .fetch_optional(pool)
        .await?;

    Ok(Json(row))

}
