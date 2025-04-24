//! 便捷宏

/// 创建一个查询构建器
#[macro_export]
macro_rules! query_builder {
    ($db:expr, $table:expr) => {
        {
            let mut builder = sqlx::QueryBuilder::new("SELECT * FROM ");
            builder.push($table);
            builder
        }
    };
    ($db:expr, $table:expr, $columns:expr) => {
        {
            let mut builder = sqlx::QueryBuilder::new("SELECT ");
            builder.push($columns);
            builder.push(" FROM ");
            builder.push($table);
            builder
        }
    };
}

/// 从错误中提取最后插入的ID
#[macro_export]
macro_rules! with_transaction {
    ($pool:expr, $body:block) => {
        {
            let tx = $pool.begin_transaction().await?;
            let result = (|| async {
                let result = $body;
                Ok::<_, $crate::error::DbError>(result)
            })().await;
            
            match result {
                Ok(value) => {
                    tx.commit().await?;
                    Ok(value)
                },
                Err(e) => {
                    tx.rollback().await?;
                    Err(e)
                }
            }
        }
    };
}

/// 安全绑定参数到查询
#[macro_export]
macro_rules! bind_params {
    ($query:expr, $($param:expr),*) => {
        {
            let query = $query;
            $(
                let query = query.bind($param);
            )*
            query
        }
    };
}
