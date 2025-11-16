use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: String,
    pub email: String,
    pub name: String,
    pub status: String,  // 'pending', 'approved', 'denied'
    pub created_at: DateTime<Utc>,
    pub last_login: DateTime<Utc>,
}
