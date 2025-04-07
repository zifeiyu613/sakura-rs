use crate::app::AppState;
use crate::error::YiceError;
use crate::utils::constants::TENANT_ID;
use crate::utils::enums;
use axum::routing::post;
use axum::{Extension, Json, Router};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::sync::Arc;
use tracing::log::info;
use url::Url;

pub(crate) fn routes() -> Router {
    let recharge_routes = Router::new().route("/getPayManageList", post(get_pay_manage_list));

    recharge_routes
}

#[derive(Serialize, Deserialize, Debug, Clone, FromRow)]
struct PayManageList {
    id: i64,
    name: String,
    #[serde(rename = "payLogo")]
    pay_logo: Option<String>,
    #[serde(rename = "paySubType")]
    pay_sub_type: u8,
    #[serde(rename = "payType")]
    pay_type: u8,
}

//  {"uri": "/recharge/getPayManageList", "ip": "null", "参数": "{"application":"yice","channel":"TEST_CHANNEL","device":"6527645354ea4692b17799ac0a1fb313","deviceCode":"HONOR-DUK-AL20","packageName":"com.kaiqi.yice","plainText":false,"source":1,"subChannel":"TEST_SUB_CHANNEL","uid":1,"version":"1000"}"}
//  {"code":0,"data":{"payManageList":[{"id":170,"name":"支付宝","payLogo":"http://image-uat.ihuajian.net/fdfsType/img/M00/00/B5/L2iGdGKqi6uAPb8nAAAEpb1ItKg479.png","payStatus":true,"paySubType":301,"payType":6,"remark":"易测"},{"id":182,"name":"微信","payLogo":"http://image-uat.ihuajian.net/fdfsType/img/M00/00/B5/L2iGdGKqi6uAe3rNAAAE0ARRDoY712.png","payStatus":true,"paySubType":335,"payType":5,"remark":"易测"}]},"msg":"操作成功","success":true}
async fn get_pay_manage_list(
    Extension(state): Extension<Arc<AppState>>,
) -> Result<Json<String>, YiceError> {
    info!(
        "Got a request to get pay manage list, with state: {:?}",
        state
    );

    let pool = state.db_manager.sakura_pay()?;

    let packname = "";

    let result: Option<Vec<PayManageList>> = sqlx::query_as(
        r#"
    SELECT * FROM t_app_pay_manage
    WHERE pay_status = ?
    and package_name = ?
    and tenant_id = ? order by asc

    "#,
    )
    // .bind(enums::State::Open)
    .bind(packname)
    .bind(TENANT_ID)
    .fetch_all(pool);

    Ok(Json("get_pay_manage_list".to_string()))
}

pub struct Recharges {
    pub path: String,
}

impl Recharges {
    pub fn new<P: Into<String>>(path: P) -> Self {
        Self { path: path.into() }
    }
}
