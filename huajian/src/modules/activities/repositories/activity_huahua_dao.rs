use rdatabase::MySqlPool;
use crate::modules::activities::models::activity_huahua::ActivityHuahua;

pub struct ActivityHuahuaDao<'a> {
    db: &'a MySqlPool,
}

impl<'a> ActivityHuahuaDao<'a> {
    pub fn new(db: &'a MySqlPool) -> Self {
        Self { db }
    }

    pub async fn find_by_id(&self, id: i64) -> Result<Option<ActivityHuahua>, sqlx::Error> {
        // 实现查询逻辑
    }

    pub async fn find_by_email(&self, email: &str) -> Result<Option<ActivityHuahua>, sqlx::Error> {
        // 实现查询逻辑
    }
    

    // 其他数据库操作...
}
