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

#[derive(Debug, Deserialize)]
pub struct GoogleIdTokenClaims {
    #[allow(dead_code)]
    pub iss: String,
    #[allow(dead_code)]
    pub aud: String,
    pub sub: String,
    pub email: String,
    pub name: Option<String>,
    #[allow(dead_code)]
    pub exp: i64,
    #[allow(dead_code)]
    pub iat: i64,
}
