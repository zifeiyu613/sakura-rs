// //! 查询辅助模块
// 
// use serde::Serialize;
// use sqlx::{Executor, FromRow, Row};
// 
// use crate::error::{DbError, Result};
// use crate::pool::DbPool;
// 
// /// 查询辅助扩展特性
// pub trait QueryExt {
//     /// 执行SQL并返回所有结果
//     async fn fetch_all_mapped<T>(&self, sql: &str) -> Result<Vec<T>>
//     where
//         T: for<'r> FromRow<'r, sqlx::any::AnyRow> + Send + Unpin;
// 
//     /// 执行SQL并返回单个结果
//     async fn fetch_one_mapped<T>(&self, sql: &str) -> Result<T>
//     where
//         T: for<'r> FromRow<'r, sqlx::any::AnyRow> + Send + Unpin;
// 
//     /// 执行SQL并返回可选的单个结果
//     async fn fetch_optional_mapped<T>(&self, sql: &str) -> Result<Option<T>>
//     where
//         T: for<'r> FromRow<'r, sqlx::any::AnyRow> + Send + Unpin;
// 
//     /// 执行插入操作并返回最后插入的ID
//     async fn insert<T>(&self, table: &str, data: &T) -> Result<i64>
//     where
//         T: Serialize + Send + Sync;
// 
//     /// 执行更新操作
//     async fn update<T>(&self, table: &str, data: &T, where_clause: &str) -> Result<u64>
//     where
//         T: Serialize + Send + Sync;
// 
//     /// 执行删除操作
//     async fn delete(&self, table: &str, where_clause: &str) -> Result<u64>;
// 
//     /// 执行计数查询
//     async fn count(&self, table: &str, where_clause: Option<&str>) -> Result<i64>;
// }
// 
// impl QueryExt for DbPool {
//     async fn fetch_all_mapped<T>(&self, sql: &str) -> Result<Vec<T>>
//     where
//         T: for<'r> FromRow<'r, sqlx::any::AnyRow> + Send + Unpin,
//     {
//         let rows = sqlx::query(sql)
//             .map(|row: sqlx::any::AnyRow| T::from_row(&row).unwrap())
//             .fetch_all(self.conn())
//             .await?;
// 
//         Ok(rows)
//     }
// 
//     async fn fetch_one_mapped<T>(&self, sql: &str) -> Result<T>
//     where
//         T: for<'r> FromRow<'r, sqlx::any::AnyRow> + Send + Unpin,
//     {
//         let row = sqlx::query(sql)
//             .map(|row: sqlx::any::AnyRow| T::from_row(&row).unwrap())
//             .fetch_one(self.conn())
//             .await?;
// 
//         Ok(row)
//     }
// 
//     async fn fetch_optional_mapped<T>(&self, sql: &str) -> Result<Option<T>>
//     where
//         T: for<'r> FromRow<'r, sqlx::any::AnyRow> + Send + Unpin,
//     {
//         let row = sqlx::query(sql)
//             .map(|row: sqlx::any::AnyRow| T::from_row(&row).unwrap())
//             .fetch_optional(self.conn())
//             .await?;
// 
//         Ok(row)
//     }
// 
//     async fn insert<T>(&self, table: &str, data: &T) -> Result<i64>
//     where
//         T: Serialize + Send + Sync,
//     {
//         // 将数据转换为JSON对象
//         let value = serde_json::to_value(data)?;
// 
//         if let serde_json::Value::Object(map) = value {
//             let mut keys = Vec::new();
//             let mut placeholders = Vec::new();
//             let mut values = Vec::new();
// 
//             for (key, value) in map {
//                 keys.push(key);
//                 placeholders.push("?".to_string());
//                 values.push(value);
//             }
// 
//             let sql = format!(
//                 "INSERT INTO {} ({}) VALUES ({})",
//                 table,
//                 keys.join(", "),
//                 placeholders.join(", ")
//             );
// 
//             // 构建查询
//             let mut query = sqlx::query(&sql);
//             for value in values {
//                 query = query.bind(value.to_string());
//             }
// 
//             // 执行
//             let result = query.execute(self.conn()).await?;
// 
//             // 获取最后插入的ID
//             let id: i64 = sqlx::query("SELECT LAST_INSERT_ID()")
//                 .map(|row: sqlx::any::AnyRow| row.get(0))
//                 .fetch_one(self.conn())
//                 .await
//                 .unwrap_or(0);
// 
//             Ok(id)
//         } else {
//             Err(DbError::Other("非对象类型不能用于更新".into()))
//         }
//     }
// 
//     async fn update<T>(&self, table: &str, data: &T, where_clause: &str) -> Result<u64>
//     where
//         T: Serialize + Send + Sync,
//     {
//         // 将数据转换为JSON对象
//         let value = serde_json::to_value(data)?;
//         
//         if let serde_json::Value::Object(map) = value {
//             let mut set_clauses = Vec::new();
//             let mut values = Vec::new();
// 
//             for (key, value) in map {
//                 set_clauses.push(format!("{} = ?", key));
//                 values.push(value);
//             }
// 
//             let sql = format!(
//                 "UPDATE {} SET {} WHERE {}",
//                 table,
//                 set_clauses.join(", "),
//                 where_clause
//             );
// 
//             // 构建查询
//             let mut query = sqlx::query(&sql);
//             for value in values {
//                 query = query.bind(value.to_string());
//             }
// 
//             // 执行
//             let result = query.execute(self.conn()).await?;
// 
//             Ok(result.rows_affected())
//         } else {
//             Err(DbError::Other("非对象类型不能用于更新".into()))
//         }
//     }
// 
//     async fn delete(&self, table: &str, where_clause: &str) -> Result<u64> {
//         let sql = format!("DELETE FROM {} WHERE {}", table, where_clause);
// 
//         let result = sqlx::query(&sql)
//             .execute(self.conn())
//             .await?;
// 
//         Ok(result.rows_affected())
//     }
// 
//     async fn count(&self, table: &str, where_clause: Option<&str>) -> Result<i64> {
//         let sql = match where_clause {
//             Some(clause) => format!("SELECT COUNT(*) FROM {} WHERE {}", table, clause),
//             None => format!("SELECT COUNT(*) FROM {}", table),
//         };
// 
//         let result: (i64,) = sqlx::query_as(&sql)
//             .fetch_one(self.conn())
//             .await?;
// 
//         Ok(result.0)
//     }
// }
