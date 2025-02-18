use crate::service::user::user_main::UserMain;
use crate::service::enums::database::DatabaseType;
use sakura_database::pool_manager::POOL_MANAGER;
use sqlx::{Acquire, Row};

pub async fn query_token(uid: &i64) -> Option<String> {

    let pool = POOL_MANAGER.get_mysql_pool(DatabaseType::Phoenix.as_str()).await.unwrap();

    // ✅ `Arc<Pool<MySql>>` 需要解引用
    let pool = &*pool;

    let row = sqlx::query("select token from t_user_main where uid = ?")
        .bind(uid)
        .fetch_one(pool).await;

    // let row: Result<(i32, String), Error> = sqlx::query_as("SELECT uid, username FROM t_user_main WHERE uid = ?")
    //     .bind(2)
    //     .fetch_one(&mut conn)
    //     .await;
    //
    println!("token: {:?}", row);

    Some(row.unwrap().get("token"))
}

// 1482675766000
pub async fn query_all(timestamp: u64) -> Vec<UserMain> {
    let mut conn = POOL_MANAGER.get_mysql_connection(DatabaseType::Phoenix.as_str()).await.unwrap();

    let mut tx = conn.begin().await.unwrap(); // 开启事务

    // 执行 SET 语句
    sqlx::query("SET optimizer_switch='index_merge=off'")
        .execute(&mut *tx)
        .await
        .unwrap();

    let sql = r#"
        SELECT m.*
        FROM t_user_main m
            LEFT JOIN t_user_info info ON m.uid = info.uid
        WHERE info.is_auth = 2
              AND m.status = 0
              AND info.status = 1
              AND m.last_open_time > ?
        ORDER BY m.last_open_time DESC
        limit 100
    "#;

    let raws = sqlx::query_as::<_, UserMain>(sql)
        .bind(timestamp)
        .fetch_all(&mut *tx).await.unwrap();

    tx.commit().await.unwrap(); // 提交事务

    let uid_list = raws.iter().map(|x| {
        x.uid
    }).collect::<Vec<i64>>();

    println!("uid_list = {:?}", uid_list);

    raws
}


#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn test_query_token() {
        let uid = 2;
        let token = query_token(&uid).await;

        assert!(token.is_some());
        assert_eq!(token.unwrap(), "fb8427e74ac3f7a0d6bb8e58e7a799ad".to_string());

        let user_main_list = query_all(1482675766000).await;
        assert!(user_main_list.len() > 0);

    }
}