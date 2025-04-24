use crate::server::AppState;
use axum::routing::post;
use axum::{Extension, Json, Router};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::sync::Arc;
use std::time::Instant;
use axum::response::IntoResponse;
use serde_json::{json, Value};
use tracing::log::{debug, info};
use url::Url;
use crate::constants::{State, DEFAULT_PACKAGE_NAME};
use app_enumeta::app_macro::App;
use crate::domain::repositories;
use repositories::PayManageRepository;
use crate::domain::dtos::OrderDTO;
use crate::domain::models::pay_manage::AppPayManageRecord;
use crate::domain::services::PayManageService;
use crate::errors::ApiError;
use crate::errors::response::ApiResponse;
use crate::middleware::extract::ApiRequest;

pub(crate) fn routes() -> Router {
    Router::new().route("/getPayManageList", post(get_pay_manage_list))
}


async fn get_pay_manage_list(
    Extension(state): Extension<Arc<AppState>>,
    api_request: ApiRequest<OrderDTO>,
) -> Result<ApiResponse<Value>, ApiError> {

    // 1. 提取并验证请求参数
    let (base_param, order_dto) = match (api_request.base, api_request.nested) {
        (Some(base), Some(order)) => (base, order),
        _ => return Err(ApiError::BadRequest("接收到空请求参数".to_string()))
    };

    info!("处理支付管理列表请求: base_param={:?}, order_dto={:?}", base_param, order_dto);

    // 2. 获取数据库连接池
    let pool = state.db_manager.sakura_pay()?;

    // 3. 创建服务实例并调用业务逻辑
    let payment_service = PayManageService::new(pool);
    let result = payment_service
        .get_pay_manage_list(base_param.package_name.as_deref())
        .await?;

    // 4. 将结果转换为API响应格式
    Ok(ApiResponse::success(json!({
        "list": result,
        "total": result.len()
    })))
}

pub struct Recharges {
    pub path: String,
}

impl Recharges {
    pub fn new<P: Into<String>>(path: P) -> Self {
        Self { path: path.into() }
    }
}

async fn benchmark_query_strategies(
    Extension(state): Extension<Arc<AppState>>,
    package_name: &str,
    iterations: usize
) {
    let pool = state.db_manager.sakura_pay().unwrap();

    let repository = PayManageRepository::new(pool);

    // 方案一: IN查询+内存过滤
    let start = Instant::now();
    for _ in 0..iterations {
        let mut result = repository.get_list_flexible(
            Some(App::YiCe.id()),
            Some(&[package_name, DEFAULT_PACKAGE_NAME]),
            Some(State::Open)
        ).await.unwrap();

        let filtered = result.into_iter()
            .filter(|item| item.package_name.as_deref().map_or(false, |name| name == package_name))
            .collect::<Vec<_>>();
    }
    let strategy1_time = start.elapsed();

    // 方案二: 两次独立查询
    let start = Instant::now();
    for _ in 0..iterations {
        let primary = repository.get_list_flexible(
            Some(App::YiCe.id()),
            Some(&[package_name]),
            Some(State::Open)
        ).await.unwrap();

        if primary.is_empty() {
            let default = repository.get_list_flexible(
                Some(App::YiCe.id()),
                Some(&[DEFAULT_PACKAGE_NAME]),
                Some(State::Open)
            ).await.unwrap();
        }
    }
    let strategy2_time = start.elapsed();

    info!("IN查询+内存过滤: {:?}", strategy1_time);
    info!("两次独立查询: {:?}", strategy2_time);
}

