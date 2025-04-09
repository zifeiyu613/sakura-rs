use crate::server::AppState;
use crate::errors::error::YiceError;
use axum::routing::post;
use axum::{Extension, Json, Router};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::sync::Arc;
use axum::response::IntoResponse;
use serde_json::{json, Value};
use tracing::log::info;
use url::Url;
use crate::middleware::decryptor::RequestData;
use crate::constants::{enums, DEFAULT_PACKAGE_NAME};
use app_enumeta::app_macro::App;
use crate::domain::repositories;
use repositories::PayManageRepository;
use crate::errors::response::ApiResponse;
use crate::params::{OrderDTO, RequestDto};
use crate::status::BusinessCode;

pub(crate) fn routes() -> Router {
    Router::new().route("/getPayManageList", post(get_pay_manage_list))
}


//  {"uri": "/recharge/getPayManageList", "ip": "null", "参数": "{"application":"yice","channel":"TEST_CHANNEL","device":"6527645354ea4692b17799ac0a1fb313","deviceCode":"HONOR-DUK-AL20","packageName":"com.kaiqi.yice","plainText":false,"source":1,"subChannel":"TEST_SUB_CHANNEL","uid":1,"version":"1000"}"}
//  {"code":0,"data":{"payManageList":[{"id":170,"name":"支付宝","payLogo":"http://image-uat.ihuajian.net/fdfsType/img/M00/00/B5/L2iGdGKqi6uAPb8nAAAEpb1ItKg479.png","payStatus":true,"paySubType":301,"payType":6,"remark":"易测"},{"id":182,"name":"微信","payLogo":"http://image-uat.ihuajian.net/fdfsType/img/M00/00/B5/L2iGdGKqi6uAe3rNAAAE0ARRDoY712.png","payStatus":true,"paySubType":335,"payType":5,"remark":"易测"}]},"msg":"操作成功","success":true}
async fn get_pay_manage_list(
    Extension(state): Extension<Arc<AppState>>,
    dto: RequestDto<OrderDTO>,
) -> Result<ApiResponse<Value>, YiceError> {
    info!(
        "Got a request to get pay manage list, with state: {:?}",
        state
    );
    let pool = match state.db_manager.sakura_pay() {
        Ok(pool) => pool,
        Err(err) => return Err(err)
    };

    match (dto.base, dto.inner) {
        (Some(base_param), Some(order_dto)) => {
            // 完整请求

            // 这里可以使用 order_dto 中的参数
            // 例如: let order_status = order_dto.status;
            info!("Got a request to get pay manage list, {:?}", order_dto);

            // 使用提取的公共方法
            let repository = PayManageRepository::new(pool);
            let mut result = match repository.get_list(
                enums::State::Open,
                &base_param.package_name(),
                App::YiCe.id(),
            ).await {
                Ok(list) => list,
                Err(err) =>  {
                    return Err(YiceError::from(err));
                }
            };
            info!("result:{:?}", result);

            if result.is_empty() {
                info!("No packages found");
                result = repository.get_list(
                    enums::State::Open,
                    DEFAULT_PACKAGE_NAME,
                    App::YiCe.id(),
                ).await?;
            }

            // 返回结果，这里可以返回 result 而不是原始参数
            Ok(ApiResponse::success(json!({
                "list": result,
                "total": result.len()
            })))
        }
        _ => {
            // 空请求 - 使用错误类型
            return Err(YiceError::BadRequest("接收到空请求参数".to_string()));
        }
    }
}

pub struct Recharges {
    pub path: String,
}

impl Recharges {
    pub fn new<P: Into<String>>(path: P) -> Self {
        Self { path: path.into() }
    }
}
