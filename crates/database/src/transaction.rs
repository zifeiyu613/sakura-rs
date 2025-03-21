// src/database/transaction.rs
use async_trait::async_trait;
use sqlx::{Acquire, MySql, MySqlPool, Transaction};
use std::future::Future;
use std::pin::Pin;
use uuid::Uuid;

#[async_trait]
pub trait DatabaseTransaction {
    // 事务执行方法
    async fn transaction<F, R, E>(&self, f: F) -> Result<R, E>
    where
        F: FnOnce(&mut Transaction<'_, MySql>) -> Pin<Box<dyn Future<Output = Result<R, E>> + Send>>
        + Send,
        R: Send + 'static,
        E: From<sqlx::Error> + Send;

    // 嵌套事务方法
    async fn nested_transaction<F, R, E>(&self, f: F) -> Result<R, E>
    where
        F: FnOnce(&mut Transaction<'_, MySql>) -> Pin<Box<dyn Future<Output = Result<R, E>> + Send>>
        + Send,
        R: Send + 'static,
        E: From<sqlx::Error> + Send;
}

// 为MySqlPool实现DatabaseTransaction特征
#[async_trait]
impl DatabaseTransaction for MySqlPool {
    async fn transaction<F, R, E>(&self, f: F) -> Result<R, E>
    where
        F: FnOnce(&mut Transaction<'_, MySql>) -> Pin<Box<dyn Future<Output = Result<R, E>> + Send>>
        + Send,
        R: Send + 'static,
        E: From<sqlx::Error> + Send,
    {
        let mut tx = self.begin().await?;
        let result = f(&mut tx).await;

        match result {
            Ok(value) => {
                tx.commit().await?;
                Ok(value)
            }
            Err(err) => {
                tx.rollback().await?;
                Err(err)
            }
        }
    }

    async fn nested_transaction<F, R, E>(&self, f: F) -> Result<R, E>
    where
        F: FnOnce(&mut Transaction<'_, MySql>) -> Pin<Box<dyn Future<Output = Result<R, E>> + Send>>
        + Send,
        R: Send + 'static,
        E: From<sqlx::Error> + Send,
    {
        let mut tx = self.begin().await?;

        // 创建唯一的保存点名称
        let savepoint = format!("SP_{}", Uuid::new_v4().simple());

        // 创建保存点 - 修复执行器问题
        sqlx::query(&format!("SAVEPOINT {}", savepoint))
            .execute(&mut *tx)  // 使用 &mut *tx 或直接使用 tx 而不是 &mut tx
            .await?;

        // 执行传入的函数
        let result = f(&mut tx).await;

        match result {
            Ok(value) => {
                // 释放保存点
                sqlx::query(&format!("RELEASE SAVEPOINT {}", savepoint))
                    .execute(&mut *tx)
                    .await?;

                // 提交事务
                tx.commit().await?;
                Ok(value)
            }
            Err(err) => {
                // 回滚到保存点（如果发生错误，不要覆盖原始错误）
                let _ = sqlx::query(&format!("ROLLBACK TO SAVEPOINT {}", savepoint))
                    .execute(&mut *tx)
                    .await;

                // 回滚事务
                let _ = tx.rollback().await;
                Err(err)
            }
        }
    }
}