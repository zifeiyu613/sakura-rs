use sqlx::MySqlPool;
use tracing::log::info;
use app_enumeta::App;
use crate::constants::{State, DEFAULT_PACKAGE_NAME};
use crate::domain::models::pay_manage::AppPayManageRecord;
use crate::domain::repositories::PayManageRepository;
use crate::errors::ApiError;

pub struct PayManageService<'a> {

    repository: PayManageRepository<'a>

}

impl<'a> PayManageService<'a> {
    pub fn new(pool: &'a MySqlPool) -> Self {
        Self {
            repository: PayManageRepository::new(pool)
        }
    }

    /// 获取支付管理列表，基于包名过滤
    /// 如果特定包名没有数据，则回退到使用默认包名
    pub async fn get_pay_manage_list(
        &self,
        package_name: Option<&str>
    ) -> Result<Vec<AppPayManageRecord>, ApiError> {
        // 使用默认包名，如果未提供
        let package_name = package_name.unwrap_or(DEFAULT_PACKAGE_NAME);

        // 获取数据，包括指定包名和默认包名
        let result = self.repository.get_list_flexible(
            Some(App::YiCe.id()),
            Some(&[package_name, DEFAULT_PACKAGE_NAME]),
            Some(State::Open)
        ).await?;

        info!("查询到 {} 条支付记录", result.len());

        // 首先尝试只使用指定包名的数据
        let filtered: Vec<_> = result.iter()
            .filter(|item| item.package_name.as_deref() == Some(package_name))
            .cloned()
            .collect();

        // 决定使用过滤结果还是所有结果
        if !filtered.is_empty() {
            info!("使用特定包名 '{}' 筛选出 {} 条记录", package_name, filtered.len());
            Ok(filtered)
        } else {
            info!("特定包名 '{}' 没有匹配记录，使用所有查询结果", package_name);
            Ok(result)
        }
    }


}