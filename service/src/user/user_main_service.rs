use crate::database_manager::get_db_pool;
use sakura_entity::user::user_main::UserMain;

use sqlx::Row;

pub async fn query_token(uid: u64) -> Option<String> {

    let pool = get_db_pool().await;

    let row = sqlx::query("select * from t_user_main where uid = ?")
        .bind(uid)
        .fetch_one(pool).await;

    let token = match row {
        Ok(row) => {
            Some(row.get::<String, _>("token"))
        },
        Err(_) => {
            None
        }
    };
    token
}

// 1482675766000
pub async fn query_all(timestamp: u64) -> Vec<UserMain> {
    let pool = get_db_pool().await;

    // 执行 SET 语句
    // sqlx::query("SET optimizer_switch='index_merge=off'")
    //     .execute(pool)
    //     .await
    //     .unwrap();

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
        .fetch_all(pool).await.unwrap();

    let uid_list = raws.iter().map(|x| {
        x.uid
    }).collect::<Vec<i64>>();

    println!("uid_list = {:?}", uid_list);

    raws
}