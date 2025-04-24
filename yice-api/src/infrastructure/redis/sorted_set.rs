/// 有序集合操作

use super::error::Result;
use super::serializer::{JsonSerializer, RedisSerializer};
use redis::{aio::ConnectionManager, AsyncCommands};
use serde::{de::DeserializeOwned, Serialize};
use std::sync::Arc;
use tracing::debug;

/// 有序集合元素
#[derive(Debug, Clone)]
pub struct ScoredValue<T> {
    pub score: f64,
    pub value: T,
}

/// 基础有序集合操作接口 - 对象安全版本
#[async_trait::async_trait]
pub trait SortedSetOps: Send + Sync {
    /// 添加元素到有序集合（原始字节）
    async fn zadd_raw(&self, key: &str, value: Vec<u8>, score: f64) -> Result<bool>;

    /// 获取元素的分数（原始字节）
    async fn zscore_raw(&self, key: &str, value: Vec<u8>) -> Result<Option<f64>>;

    /// 获取指定排名范围的元素（原始字节）
    async fn zrange_raw(&self, key: &str, start: isize, stop: isize) -> Result<Vec<Vec<u8>>>;

    /// 获取指定排名范围的元素（带分数，原始字节）
    async fn zrange_with_scores_raw(&self, key: &str, start: isize, stop: isize) -> Result<Vec<(f64, Vec<u8>)>>;

    /// 获取指定分数范围的元素（原始字节）
    async fn zrangebyscore_raw(&self, key: &str, min: f64, max: f64) -> Result<Vec<Vec<u8>>>;

    /// 获取指定分数范围的元素（带分数，原始字节）
    async fn zrangebyscore_with_scores_raw(&self, key: &str, min: f64, max: f64) -> Result<Vec<(f64, Vec<u8>)>>;

    /// 获取有序集合大小
    async fn zcard(&self, key: &str) -> Result<i64>;

    /// 删除元素（原始字节）
    async fn zrem_raw(&self, key: &str, value: Vec<u8>) -> Result<bool>;

    /// 获取元素排名（原始字节）
    async fn zrank_raw(&self, key: &str, value: Vec<u8>) -> Result<Option<i64>>;

    /// 增加元素分数（原始字节）
    async fn zincrby_raw(&self, key: &str, value: Vec<u8>, increment: f64) -> Result<f64>;

    /// 获取排名范围的元素（按降序，原始字节）
    async fn zrevrange_raw(&self, key: &str, start: isize, stop: isize) -> Result<Vec<Vec<u8>>>;
}

/// 扩展有序集合操作接口 - 提供泛型方法
#[async_trait::async_trait]
pub trait SortedSetOpsExt: SortedSetOps {
    /// 添加元素到有序集合
    async fn zadd<T: Serialize + Send + Sync>(&self, key: &str, value: &T, score: f64) -> Result<bool> {
        let serialized = serde_json::to_vec(value)?;
        self.zadd_raw(key, serialized, score).await
    }

    /// 获取元素的分数
    async fn zscore<T: Serialize + Send + Sync>(&self, key: &str, value: &T) -> Result<Option<f64>> {
        let serialized = serde_json::to_vec(value)?;
        self.zscore_raw(key, serialized).await
    }

    /// 获取指定排名范围的元素
    async fn zrange<T: DeserializeOwned + Send>(&self, key: &str, start: isize, stop: isize) -> Result<Vec<T>> {
        let raw_values = self.zrange_raw(key, start, stop).await?;
        let mut result = Vec::with_capacity(raw_values.len());

        for data in raw_values {
            let value: T = serde_json::from_slice(&data)?;
            result.push(value);
        }

        Ok(result)
    }

    /// 获取指定排名范围的元素（带分数）
    async fn zrange_with_scores<T: DeserializeOwned + Send>(&self, key: &str, start: isize, stop: isize) -> Result<Vec<ScoredValue<T>>> {
        let raw_values = self.zrange_with_scores_raw(key, start, stop).await?;
        let mut result = Vec::with_capacity(raw_values.len());

        for (score, data) in raw_values {
            let value: T = serde_json::from_slice(&data)?;
            result.push(ScoredValue { score, value });
        }

        Ok(result)
    }

    /// 获取指定分数范围的元素
    async fn zrangebyscore<T: DeserializeOwned + Send>(&self, key: &str, min: f64, max: f64) -> Result<Vec<T>> {
        let raw_values = self.zrangebyscore_raw(key, min, max).await?;
        let mut result = Vec::with_capacity(raw_values.len());

        for data in raw_values {
            let value: T = serde_json::from_slice(&data)?;
            result.push(value);
        }

        Ok(result)
    }

    /// 获取指定分数范围的元素（带分数）
    async fn zrangebyscore_with_scores<T: DeserializeOwned + Send>(&self, key: &str, min: f64, max: f64) -> Result<Vec<ScoredValue<T>>> {
        let raw_values = self.zrangebyscore_with_scores_raw(key, min, max).await?;
        let mut result = Vec::with_capacity(raw_values.len());

        for (score, data) in raw_values {
            let value: T = serde_json::from_slice(&data)?;
            result.push(ScoredValue { score, value });
        }

        Ok(result)
    }

    /// 删除元素
    async fn zrem<T: Serialize + Send + Sync>(&self, key: &str, value: &T) -> Result<bool> {
        let serialized = serde_json::to_vec(value)?;
        self.zrem_raw(key, serialized).await
    }

    /// 获取元素排名
    async fn zrank<T: Serialize + Send + Sync>(&self, key: &str, value: &T) -> Result<Option<i64>> {
        let serialized = serde_json::to_vec(value)?;
        self.zrank_raw(key, serialized).await
    }

    /// 增加元素分数
    async fn zincrby<T: Serialize + Send + Sync>(&self, key: &str, value: &T, increment: f64) -> Result<f64> {
        let serialized = serde_json::to_vec(value)?;
        self.zincrby_raw(key, serialized, increment).await
    }

    /// 获取排名范围的元素（按降序）
    async fn zrevrange<T: DeserializeOwned + Send>(&self, key: &str, start: isize, stop: isize) -> Result<Vec<T>> {
        let raw_values = self.zrevrange_raw(key, start, stop).await?;
        let mut result = Vec::with_capacity(raw_values.len());

        for data in raw_values {
            let value: T = serde_json::from_slice(&data)?;
            result.push(value);
        }

        Ok(result)
    }
}

// 为所有 SortedSetOps 实现者自动提供 SortedSetOpsExt 功能
impl<T: SortedSetOps + ?Sized> SortedSetOpsExt for T {}


/// Redis有序集合操作实现
#[derive(Clone)]
pub struct RedisSortedSet {
    connection_manager: ConnectionManager,
    serializer: JsonSerializer,
    prefix: String,
}

impl RedisSortedSet {
    /// 创建新的Redis有序集合操作
    pub fn new(connection_manager: ConnectionManager) -> Self {
        Self {
            connection_manager,
            serializer: JsonSerializer,
            prefix: "sortedset:".to_string(),
        }
    }


    /// 设置键前缀
    pub fn with_prefix<S: Into<String>>(mut self, prefix: S) -> Self {
        self.prefix = prefix.into();
        self
    }

    /// 获取完整的键名
    fn get_key(&self, key: &str) -> String {
        format!("{}{}", self.prefix, key)
    }
}

#[async_trait::async_trait]
impl SortedSetOps for RedisSortedSet {
    async fn zadd_raw(&self, key: &str, value: Vec<u8>, score: f64) -> Result<bool> {
        let mut conn = self.connection_manager.clone();
        let result: i32 = redis::cmd("ZADD")
            .arg(key)
            .arg(score)
            .arg(value)
            .query_async(&mut conn)
            .await?;
        Ok(result > 0)
    }

    async fn zscore_raw(&self, key: &str, value: Vec<u8>) -> Result<Option<f64>> {
        let mut conn = self.connection_manager.clone();
        let result: Option<f64> = redis::cmd("ZSCORE")
            .arg(key)
            .arg(value)
            .query_async(&mut conn)
            .await?;
        Ok(result)
    }

    async fn zrange_raw(&self, key: &str, start: isize, stop: isize) -> Result<Vec<Vec<u8>>> {
        let mut conn = self.connection_manager.clone();
        let result = redis::cmd("ZRANGE")
            .arg(key)
            .arg(start)
            .arg(stop)
            .query_async(&mut conn)
            .await?;
        Ok(result)
    }

    async fn zrange_with_scores_raw(&self, key: &str, start: isize, stop: isize) -> Result<Vec<(f64, Vec<u8>)>> {
        let mut conn = self.connection_manager.clone();
        let result = redis::cmd("ZRANGE")
            .arg(key)
            .arg(start)
            .arg(stop)
            .arg("WITHSCORES")
            .query_async(&mut conn)
            .await?;
        Ok(result)
    }

    async fn zrangebyscore_raw(&self, key: &str, min: f64, max: f64) -> Result<Vec<Vec<u8>>> {
        let mut conn = self.connection_manager.clone();
        let result = redis::cmd("ZRANGEBYSCORE")
            .arg(key)
            .arg(min)
            .arg(max)
            .query_async(&mut conn)
            .await?;
        Ok(result)
    }

    async fn zrangebyscore_with_scores_raw(&self, key: &str, min: f64, max: f64) -> Result<Vec<(f64, Vec<u8>)>> {
        let mut conn = self.connection_manager.clone();
        let result = redis::cmd("ZRANGEBYSCORE")
            .arg(key)
            .arg(min)
            .arg(max)
            .arg("WITHSCORES")
            .query_async(&mut conn)
            .await?;
        Ok(result)
    }

    async fn zcard(&self, key: &str) -> Result<i64> {
        let mut conn = self.connection_manager.clone();
        let result = redis::cmd("ZCARD")
            .arg(key)
            .query_async(&mut conn)
            .await?;
        Ok(result)
    }

    async fn zrem_raw(&self, key: &str, value: Vec<u8>) -> Result<bool> {
        let mut conn = self.connection_manager.clone();
        let result: i32 = redis::cmd("ZREM")
            .arg(key)
            .arg(value)
            .query_async(&mut conn)
            .await?;
        Ok(result > 0)
    }

    async fn zrank_raw(&self, key: &str, value: Vec<u8>) -> Result<Option<i64>> {
        let mut conn = self.connection_manager.clone();
        let result: Option<i64> = redis::cmd("ZRANK")
            .arg(key)
            .arg(value)
            .query_async(&mut conn)
            .await?;
        Ok(result)
    }

    async fn zincrby_raw(&self, key: &str, value: Vec<u8>, increment: f64) -> Result<f64> {
        let mut conn = self.connection_manager.clone();
        let result = redis::cmd("ZINCRBY")
            .arg(key)
            .arg(increment)
            .arg(value)
            .query_async(&mut conn)
            .await?;
        Ok(result)
    }

    async fn zrevrange_raw(&self, key: &str, start: isize, stop: isize) -> Result<Vec<Vec<u8>>> {
        let mut conn = self.connection_manager.clone();
        let result = redis::cmd("ZREVRANGE")
            .arg(key)
            .arg(start)
            .arg(stop)
            .query_async(&mut conn)
            .await?;
        Ok(result)
    }
}


#[cfg(test)]
mod tests {
    use serde::Serialize;
    use serde::Deserialize;
    use super::*;
    // #[tokio::test]
    // async fn test_sorted_set_ops() {
    //     // 创建Redis有序集合操作实例
    //     let conn_manager = redis::ConnectionManager::new(redis::Client::open("redis://127.0.0.1/")?)?;
    //     let redis_sorted_set = RedisSortedSetOps::new(conn_manager);
    //
    //     // 作为trait对象使用
    //     let leaderboard = LeaderboardManager::new(Arc::new(redis_sorted_set));
    //
    //     // 玩家结构体
    //     #[derive(Serialize, Deserialize)]
    //     struct Player {
    //         id: String,
    //         name: String,
    //     }
    //
    //     // 更新玩家分数
    //     let player = Player {
    //         id: "p123".to_string(),
    //         name: "张三".to_string()
    //     };
    //     leaderboard.update_score("game:highscores", &player, 1500.0).await?;
    //
    //     // 获取前10名玩家
    //     let top_players: Vec<ScoredValue<Player>> = leaderboard.get_top_players("game:highscores", 10).await?;
    //     for (index, scored_player) in top_players.iter().enumerate() {
    //         println!("第{}名: {} - 分数: {}",
    //                  index + 1,
    //                  scored_player.value.name,
    //                  scored_player.score
    //         );
    //     }
    //
    //     // 获取指定分数范围的玩家
    //     let players_in_range: Vec<ScoredValue<Player>> =
    //         leaderboard.get_players_by_score_range("game:highscores", 1000.0, 2000.0).await?;
    // }

}