// src/database/transaction.rs  
use async_trait::async_trait;
use sqlx::{MySql, MySqlPool, Transaction};
use std::future::Future;
use std::pin::Pin;

#[async_trait]
pub trait DatabaseTransaction {
    // 事务执行方法  
    async fn transaction<F, R, E>(&self, f: F) -> Result<R, E>
    where
        F: FnOnce(&mut Transaction<'_, MySql>) -> Pin<Box<dyn Future<Output = Result<R, E>> + Send>>
        + Send,
        E: From<sqlx::Error>;

    // 嵌套事务方法  
    async fn nested_transaction<F, R, E>(&self, f: F) -> Result<R, E>
    where
        F: FnOnce(&mut Transaction<'_, MySql>) -> Pin<Box<dyn Future<Output = Result<R, E>> + Send>>
        + Send,
        E: From<sqlx::Error>;
}

// 为MySqlPool实现DatabaseTransaction特征  
#[async_trait]
impl DatabaseTransaction for MySqlPool {
    async fn transaction<F, R, E>(&self, f: F) -> Result<R, E>
    where
        F: FnOnce(&mut Transaction<'_, MySql>) -> Pin<Box<dyn Future<Output = Result<R, E>> + Send>>
        + Send,
        E: From<sqlx::Error>,
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
        E: From<sqlx::Error>,
    {
        let mut tx = self.begin().await?;
        let save_point = format!("SP_{}", uuid::Uuid::new_v4().simple());
        sqlx::query(&format!("SAVEPOINT {}", save_point))
            .execute(&mut tx)
            .await?;

        let result = f(&mut tx).await;

        match result {
            Ok(value) => {
                sqlx::query(&format!("RELEASE SAVEPOINT {}", save_point))
                    .execute(&mut tx)
                    .await?;
                tx.commit().await?;
                Ok(value)
            }
            Err(err) => {
                sqlx::query(&format!("ROLLBACK TO SAVEPOINT {}", save_point))
                    .execute(&mut tx)
                    .await?;
                tx.rollback().await?;
                Err(err)
            }
        }
    }
}  