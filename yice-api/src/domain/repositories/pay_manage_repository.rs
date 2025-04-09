use sqlx::{MySql, Pool};
use crate::domain::models::pay_manage::PayManageList;
use crate::utils::enums;

pub struct PayManageRepository<'a> {
    pool: &'a Pool<MySql>, // 或其他数据库类型
}

impl<'a> PayManageRepository<'a> {
    pub fn new(pool: &'a Pool<MySql>) -> Self {
        Self { pool }
    }

    // 基本查询
    pub async fn get_list(
        &self,
        status: enums::State,
        package_name: &str,
        tenant_id: u8
    ) -> Result<Vec<PayManageList>, sqlx::Error> {
        sqlx::query_as(r#"
            SELECT * FROM t_app_pay_manage
            WHERE pay_status = ?
            AND package_name = ?
            AND tenant_id = ?
            ORDER BY sort ASC
            "#)
            .bind(status)
            .bind(package_name)
            .bind(tenant_id)
            .fetch_all(self.pool).await
    }

    // 带分页的查询
    pub async fn get_list_paged(
        &self,
        status: enums::State,
        package_name: &str,
        tenant_id: u8,
        page: u32,
        page_size: u32
    ) -> Result<(Vec<PayManageList>, u64), sqlx::Error> {
        // 计算偏移量
        let offset = (page - 1) * page_size;

        // 查询数据
        let items = sqlx::query_as(r#"
            SELECT * FROM t_app_pay_manage
            WHERE pay_status = ?
            AND package_name = ?
            AND tenant_id = ?
            ORDER BY sort ASC
            LIMIT ? OFFSET ?
            "#)
            .bind(status)
            .bind(package_name)
            .bind(tenant_id)
            .bind(page_size)
            .bind(offset)
            .fetch_all(self.pool).await?;

        // 查询总数
        let total: (i64,) = sqlx::query_as(r#"
            SELECT COUNT(*) FROM t_app_pay_manage
            WHERE pay_status = ?
            AND package_name = ?
            AND tenant_id = ?
            "#)
            .bind(status)
            .bind(package_name)
            .bind(tenant_id)
            .fetch_one(self.pool).await?;

        Ok((items, total.0 as u64))
    }

    // 其他相关方法...
    pub async fn get_by_id(&self, id: i64) -> Result<Option<PayManageList>, sqlx::Error> {
        sqlx::query_as(r#"
            SELECT * FROM t_app_pay_manage
            WHERE id = ?
            "#)
            .bind(id)
            .fetch_optional(self.pool).await
    }

    pub async fn create(&self, item: &PayManageList) -> Result<i64, sqlx::Error> {
        // 实现创建逻辑...
        todo!()
    }

}