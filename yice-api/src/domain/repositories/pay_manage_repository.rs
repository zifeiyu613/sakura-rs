use crate::constants::{DEFAULT_PACKAGE_NAME, State};
use crate::domain::models::pay_manage::{TAppPayManage, AppPayManageRecord};
use sea_query::{Expr, MysqlQueryBuilder, Order, Query};
use sea_query::ColumnRef::Asterisk;
use sea_query_binder::SqlxBinder;
use sqlx::mysql::MySqlRow;
use sqlx::{MySql, Pool};
use tracing::log::debug;

pub struct PayManageRepository<'a> {
    pool: &'a Pool<MySql>, // 或其他数据库类型
}

impl<'a> PayManageRepository<'a> {
    pub fn new(pool: &'a Pool<MySql>) -> Self {
        Self { pool }
    }

    pub async fn get_list_flexible(
        &self,
        tenant_id: Option<u8>,
        package_names: Option<&[&str]>,
        state: Option<State>,
    ) -> Result<Vec<AppPayManageRecord>, sqlx::Error> {
        // 构建查询
        AppPayQueryBuilder::new()
            .with_state(state)
            .with_package_names(package_names)
            .with_tenant_id(tenant_id)
            .with_sort_order(true)
            .execute(self.pool)
            .await
    }

    // 基本查询
    pub async fn get_list(
        &self,
        state: State,
        package_name: &str,
        tenant_id: u8,
    ) -> Result<Vec<AppPayManageRecord>, sqlx::Error> {
        sqlx::query_as::<_, AppPayManageRecord>(
            r#"
            SELECT * FROM t_app_pay_manage
            WHERE pay_status = ?
            AND package_name = ?
            AND tenant_id = ?
            ORDER BY sort ASC
            "#,
        )
        .bind(i8::from(state))
        .bind(package_name)
        .bind(tenant_id)
        .fetch_all(self.pool)
        .await
    }

    // 带分页的查询
    pub async fn get_list_paged(
        &self,
        status: State,
        package_name: &str,
        tenant_id: u8,
        page: u32,
        page_size: u32,
    ) -> Result<(Vec<AppPayManageRecord>, u64), sqlx::Error> {
        // 计算偏移量
        let offset = (page - 1) * page_size;

        // 查询数据
        let items = sqlx::query_as(
            r#"
            SELECT * FROM t_app_pay_manage
            WHERE pay_status = ?
            AND package_name = ?
            AND tenant_id = ?
            ORDER BY sort ASC
            LIMIT ? OFFSET ?
            "#,
        )
        .bind(status)
        .bind(package_name)
        .bind(tenant_id)
        .bind(page_size)
        .bind(offset)
        .fetch_all(self.pool)
        .await?;

        // 查询总数
        let total: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM t_app_pay_manage
            WHERE pay_status = ?
            AND package_name = ?
            AND tenant_id = ?
            "#,
        )
        .bind(status)
        .bind(package_name)
        .bind(tenant_id)
        .fetch_one(self.pool)
        .await?;

        Ok((items, total.0 as u64))
    }

    // 其他相关方法...
    pub async fn get_by_id(&self, id: i64) -> Result<Option<AppPayManageRecord>, sqlx::Error> {
        sqlx::query_as(
            r#"
            SELECT * FROM t_app_pay_manage
            WHERE id = ?
            "#,
        )
        .bind(id)
        .fetch_optional(self.pool)
        .await
    }

    pub async fn create(&self, item: &AppPayManageRecord) -> Result<i64, sqlx::Error> {
        // 实现创建逻辑...
        todo!()
    }
}

// 通用查询构建器
pub struct AppPayQueryBuilder<'a> {
    state: Option<State>,
    package_names: Option<&'a [&'a str]>,
    tenant_id: Option<u8>,
    sort_order: Option<Order>,
}

impl<'a> AppPayQueryBuilder<'a> {
    pub fn new() -> Self {
        Self {
            state: None,
            package_names: Some(&[DEFAULT_PACKAGE_NAME]),
            tenant_id: None,
            sort_order: Some(Order::Asc), // 默认排序
        }
    }

    pub fn with_state(mut self, state: Option<State>) -> Self {
        self.state = state;
        self
    }

    pub fn with_package_names(mut self, names: Option<&'a[&str]>) -> Self {
        self.package_names = names;
        self
    }

    pub fn with_tenant_id(mut self, tenant_id: Option<u8>) -> Self {
        self.tenant_id = tenant_id;
        self
    }

    pub fn with_sort_order(mut self, ascending: bool) -> Self {
        self.sort_order = Some(if ascending { Order::Asc } else { Order::Desc });
        self
    }

    pub async fn execute(self, pool: &Pool<MySql>) -> Result<Vec<AppPayManageRecord>, sqlx::Error> {
        let mut query = Query::select()
            .columns([Asterisk])
            .from(TAppPayManage::Table)
            .apply_if(self.state, |q, v| {
                q.and_where(Expr::col(TAppPayManage::PayStatus).eq(i8::from(v)));
            })
            .apply_if(self.package_names, |q, v| {
                let package_names = v.iter().map(|s| s.to_string()).collect::<Vec<_>>();
                q.and_where(Expr::col(TAppPayManage::PackageName).is_in(package_names));
            })
            .apply_if(self.tenant_id, |q, v| {
                q.and_where(Expr::col(TAppPayManage::TenantId).eq(v));
            })
            .apply_if(self.sort_order, |q, v| {
                q.order_by(TAppPayManage::Sort, v);
            })
            .to_owned();

        // 构建SQL和执行
        let (sql, values) = query.build_sqlx(MysqlQueryBuilder);
        debug!("执行SQL: {}, values:{:?}", sql, values.to_owned());

        // 执行查询
        sqlx::query_as_with::<_, AppPayManageRecord, _>(&sql, values)
            .fetch_all(pool)
            .await

    }
}
