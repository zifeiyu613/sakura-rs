use std::sync::Arc;
use uuid::Uuid;

use crate::domain::models::user::User;
use crate::error::AppError;
use crate::server::AppState;
use crate::utils::pagination::{Paginated, PaginationParams};

pub struct UserService {
    state: Arc<AppState>,
}

impl UserService {
    pub fn new(state: Arc<AppState>) -> Self {
        Self { state }
    }

    pub async fn list_users(&self, pagination: PaginationParams) -> Result<Paginated<User>, AppError> {
        let page = pagination.page.unwrap_or(1);
        let page_size = pagination.page_size.unwrap_or(20);
        let offset = (page - 1) * page_size;

        // 获取总记录数
        let total = sqlx::query!(
            r#"SELECT COUNT(*) as count FROM users"#
        )
            .fetch_one(&self.state.db)
            .await?
            .count as u64;

        // 获取分页数据
        let users = sqlx::query_as!(
            User,
            r#"
            SELECT * FROM users
            ORDER BY created_at DESC
            LIMIT ? OFFSET ?
            "#,
            page_size,
            offset
        )
            .fetch_all(&self.state.db)
            .await?;

        Ok(Paginated {
            items: users,
            total,
            page,
            page_size,
        })
    }

    pub async fn get_user(&self, id: Uuid) -> Result<User, AppError> {
        let user = sqlx::query_as!(
            User,
            r#"SELECT * FROM users WHERE id = ?"#,
            id
        )
            .fetch_optional(&self.state.db)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("User with ID {} not found", id)))?;

        Ok(user)
    }

    pub async fn create_user(&self, email: &str, name: &str, password: &str) -> Result<User, AppError> {
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
        let auth_service = crate::domain::services::auth_service::AuthService::new(self.state.clone());
        let password_hash = auth_service.hash_password(password)?;

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

        Ok(user)
    }

    pub async fn update_user(
        &self,
        id: Uuid,
        name: Option<String>,
        email: Option<String>
    ) -> Result<User, AppError> {
        // 检查用户是否存在
        let mut user = self.get_user(id).await?;

        // 如果更新邮箱，检查邮箱是否已存在
        if let Some(new_email) = &email {
            if new_email != &user.email {
                let existing_user = sqlx::query!(
                    r#"SELECT id FROM users WHERE email = ? AND id != ?"#,
                    new_email,
                    id
                )
                    .fetch_optional(&self.state.db)
                    .await?;

                if existing_user.is_some() {
                    return Err(AppError::Validation("Email already exists".to_string()));
                }

                user.email = new_email.clone();
            }
        }

        // 更新其他字段
        if let Some(new_name) = &name {
            user.name = new_name.clone();
        }

        user.updated_at = chrono::Utc::now();

        // 更新数据库
        sqlx::query!(
            r#"
            UPDATE users
            SET name = ?, email = ?, updated_at = ?
            WHERE id = ?
            "#,
            user.name,
            user.email,
            user.updated_at,
            user.id
        )
            .execute(&self.state.db)
            .await?;

        Ok(user)
    }

    pub async fn delete_user(&self, id: Uuid) -> Result<(), AppError> {
        // 检查用户是否存在
        let _ = self.get_user(id).await?;

        // 删除用户
        sqlx::query!(
            r#"DELETE FROM users WHERE id = ?"#,
            id
        )
            .execute(&self.state.db)
            .await?;

        Ok(())
    }
}
