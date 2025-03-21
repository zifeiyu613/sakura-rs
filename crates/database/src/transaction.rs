// // src/database/transaction.rs
// use async_trait::async_trait;
// use sqlx::{Acquire, MySql, MySqlPool, Transaction};
// use std::future::Future;
// use std::pin::Pin;
//
// #[async_trait]
// pub trait DatabaseTransaction {
//     // 事务执行方法
//     async fn transaction<F, R, E>(&self, f: F) -> Result<R, E>
//     where
//         F: FnOnce(&mut Transaction<'_, MySql>) -> Pin<Box<dyn Future<Output = Result<R, E>> + Send>>
//         + Send,
//         R: Send + 'static,
//         E: From<sqlx::Error> + Send;
//
//     // 嵌套事务方法
//     async fn nested_transaction<F, R, E>(&self, f: F) -> Result<R, E>
//     where
//         F: FnOnce(&mut Transaction<'_, MySql>) -> Pin<Box<dyn Future<Output = Result<R, E>> + Send>>
//         + Send,
//         R: Send + 'static,
//         E: From<sqlx::Error> + Send;
// }
//
// // 为MySqlPool实现DatabaseTransaction特征
// #[async_trait]
// impl DatabaseTransaction for MySqlPool {
//     async fn transaction<F, R, E>(&self, f: F) -> Result<R, E>
//     where
//         F: FnOnce(&mut Transaction<'_, MySql>) -> Pin<Box<dyn Future<Output = Result<R, E>> + Send>>
//         + Send,
//         R: Send + 'static,
//         E: From<sqlx::Error> + Send,
//     {
//         let mut tx = self.begin().await?;
//         let result = f(&mut tx).await;
//
//         match result {
//             Ok(value) => {
//                 tx.commit().await?;
//                 Ok(value)
//             }
//             Err(err) => {
//                 tx.rollback().await?;
//                 Err(err)
//             }
//         }
//     }
//
//     async fn nested_transaction<F, R, E>(&self, f: F) -> Result<R, E>
//     where
//         F: FnOnce(&mut Transaction<'_, MySql>) -> Pin<Box<dyn Future<Output = Result<R, E>> + Send>>
//         + Send,
//         R: Send + 'static,
//         E: From<sqlx::Error> + Send,
//     {
//         // 利用 SQLx 原生的事务嵌套能力
//         self.transaction(|tx| Box::pin(async move {
//             // SQLx 会自动将嵌套的 begin 转换为 savepoint
//             let mut nested_tx = tx.begin().await?;
//
//             let result = f(&mut nested_tx).await;
//
//             match result {
//                 Ok(value) => {
//                     nested_tx.commit().await?;
//                     Ok(value)
//                 }
//                 Err(err) => {
//                     // 尝试回滚，但不覆盖原始错误
//                     let _ = nested_tx.rollback().await;
//                     Err(err)
//                 }
//             }
//         })).await
//     }
// }