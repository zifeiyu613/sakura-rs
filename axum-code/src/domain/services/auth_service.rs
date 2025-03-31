use std::sync::Arc;
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation, Algorithm};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::models::user::User;
use crate::error::AppError;
use crate::server::AppState;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,     // Subject (user ID)
    pub exp: usize,      // Expiration time
    pub iat: usize,      // Issued at time
    pub email: String,   // User email
}

#[derive(Debug)]
pub struct AuthResult {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: u64,
}

pub struct AuthService {
    state: Arc<AppState>,
}

impl AuthService {
    pub fn new(state: Arc<AppState>) -> Self {
        Self { state }
    }

    pub async fn login(&self, email: &str, password: &str) -> Result<AuthResult, AppError> {
        // 从数据库获取用户
        let user = sqlx::query_as!(
            User,
            r#"SELECT * FROM users WHERE email = ?"#,
            email
        )
            .fetch_optional(&self.state.db)
            .await?
            .ok_or_else(|| AppError::Auth("Invalid email or password".to_string()))?;

        // 验证密码
        let is_valid = self.verify_password(password, &user.password_hash)?;
        if !is_valid {
            return Err(AppError::Auth("Invalid email or password".to_string()));
        }

        // 生成令牌
        self.generate_tokens(user).await
    }

    pub async fn register(&self, email: &str, password: &str, name: &str) -> Result<AuthResult, AppError> {
        // 检查邮箱是否已存在
        let existing_user = sqlx::query!(
            r#"SELECT id FROM users WHERE email = ?"#,
            email
        )
            .fetch_optional(&self.state.db)
            .await?;

        if existing_user.is_some() {
            return Err(AppError::Validation("Email already exists".to_string()));
        }

        // 哈希密码
        let password_hash = self.hash_password(password)?;

        // 创建用户
        let user = User::new(email, name, &password_hash);

        // 存储到数据库
        sqlx::query!(
            r#"
            INSERT INTO users (id, email, name, password_hash, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
            user.id,
            user.email,
            user.name,
            user.password_hash,
            user.created_at,
            user.updated_at
        )
            .execute(&self.state.db)
            .await?;

        // 生成令牌
        self.generate_tokens(user).await
    }

    pub async fn refresh_token(&self, refresh_token: &str) -> Result<AuthResult, AppError> {
        // 验证刷新令牌
        let user_id = self.validate_refresh_token(refresh_token).await?;

        // 获取用户信息
        let user = sqlx::query_as!(
            User,
            r#"SELECT * FROM users WHERE id = ?"#,
            user_id
        )
            .fetch_one(&self.state.db)
            .await?;

        // 生成新令牌
        self.generate_tokens(user).await
    }

    pub async fn logout(&self, user_id: Uuid) -> Result<(), AppError> {
        // 使用 Redis 将用户的所有令牌添加到黑名单
        let mut conn = self.state.redis.get_async_connection().await?;

        // 黑名单当前用户 ID 的所有令牌
        let key = format!("user:{}:tokens:blacklist", user_id);
        let _: () = redis::cmd("SET")
            .arg(&key)
            .arg("1")
            .arg("EX")
            .arg(self.state.config.auth.token_expiry_hours * 3600)
            .query_async(&mut conn)
            .await?;

        Ok(())
    }

    pub async fn validate_token(&self, token: &str) -> Result<Uuid, AppError> {
        // 解码并验证令牌
        let secret = self.state.config.auth.jwt_secret.as_bytes();
        let validation = Validation::new(Algorithm::HS256);

        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(secret),
            &validation,
        )
            .map_err(|_| AppError::Auth("Invalid token".to_string()))?;

        let user_id = Uuid::parse_str(&token_data.claims.sub)
            .map_err(|_| AppError::Auth("Invalid token subject".to_string()))?;

        // 检查令牌是否在黑名单中
        let mut conn = self.state.redis.get_async_connection().await?;
        let key = format!("user:{}:tokens:blacklist", user_id);
        let blacklisted: bool = redis::cmd("EXISTS")
            .arg(&key)
            .query_async(&mut conn)
            .await?;

        if blacklisted {
            return Err(AppError::Auth("Token has been revoked".to_string()));
        }

        Ok(user_id)
    }

    async fn generate_tokens(&self, user: User) -> Result<AuthResult, AppError> {
        let expiry_hours = self.state.config.auth.token_expiry_hours;
        let secret = self.state.config.auth.jwt_secret.as_bytes();

        // 创建访问令牌
        let exp = Utc::now()
            .checked_add_signed(Duration::hours(expiry_hours as i64))
            .expect("Invalid timestamp")
            .timestamp() as usize;

        let claims = Claims {
            sub: user.id.to_string(),
            exp,
            iat: Utc::now().timestamp() as usize,
            email: user.email.clone(),
        };

        let access_token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(secret),
        )
            .map_err(|e| AppError::Internal(format!("Failed to create token: {}", e)))?;

        // 创建刷新令牌 (有效期更长)
        let refresh_exp = Utc::now()
            .checked_add_signed(Duration::hours((expiry_hours * 24) as i64))
            .expect("Invalid timestamp")
            .timestamp() as usize;

        let refresh_claims = Claims {
            sub: user.id.to_string(),
            exp: refresh_exp,
            iat: Utc::now().timestamp() as usize,
            email: user.email,
        };

        let refresh_token = encode(
            &Header::default(),
            &refresh_claims,
            &EncodingKey::from_secret(secret),
        )
            .map_err(|e| AppError::Internal(format!("Failed to create refresh token: {}", e)))?;

        // 存储刷新令牌引用到 Redis，用于跟踪
        let mut conn = self.state.redis.get_async_connection().await?;
        let key = format!("user:{}:refresh_token", user.id);
        let _: () = redis::cmd("SET")
            .arg(&key)
            .arg(&refresh_token)
            .arg("EX")
            .arg(expiry_hours * 24 * 3600)
            .query_async(&mut conn)
            .await?;

        Ok(AuthResult {
            access_token,
            refresh_token,
            expires_in: expiry_hours * 3600,
        })
    }

    async fn validate_refresh_token(&self, token: &str) -> Result<Uuid, AppError> {
        // 解码令牌
        let secret = self.state.config.auth.jwt_secret.as_bytes();
        let validation = Validation::new(Algorithm::HS256);

        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(secret),
            &validation,
        )
            .map_err(|_| AppError::Auth("Invalid refresh token".to_string()))?;

        let user_id = Uuid::parse_str(&token_data.claims.sub)
            .map_err(|_| AppError::Auth("Invalid token subject".to_string()))?;

        // 检查令牌是否与 Redis 中存储的一致
        let mut conn = self.state.redis.get_async_connection().await?;
        let key = format!("user:{}:refresh_token", user_id);
        let stored_token: Option<String> = redis::cmd("GET")
            .arg(&key)
            .query_async(&mut conn)
            .await?;

        match stored_token {
            Some(stored) if stored == token => Ok(user_id),
            _ => Err(AppError::Auth("Invalid refresh token".to_string())),
        }
    }

    fn hash_password(&self, password: &str) -> Result<String, AppError> {
        // 使用 Argon2 哈希密码
        let salt = rand::random::<[u8; 32]>();
        let config = argon2::Config::default();

        let hash = argon2::hash_encoded(password.as_bytes(), &salt, &config)
            .map_err(|e| AppError::Internal(format!("Failed to hash password: {}", e)))?;

        Ok(hash)
    }

    fn verify_password(&self, password: &str, hash: &str) -> Result<bool, AppError> {
        let is_valid = argon2::verify_encoded(hash, password.as_bytes())
            .map_err(|e| AppError::Internal(format!("Failed to verify password: {}", e)))?;

        Ok(is_valid)
    }
}
