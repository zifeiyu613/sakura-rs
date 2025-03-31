use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub name: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl User {
    pub fn new(email: &str, name: &str, password_hash: &str) -> Self {
        Self {
            id: Uuid::new_v4(),
            email: email.to_string(),
            name: name.to_string(),
            password_hash: password_hash.to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
}
