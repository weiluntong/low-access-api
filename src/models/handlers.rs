use serde::{Deserialize, Serialize};
use super::user::User;

#[derive(Deserialize)]
pub struct GenerateTokenRequest {
    pub id_token: String,
}

#[derive(Serialize)]
pub struct ValidateTokenResponse {
    pub success: bool,
    pub user: Option<User>,
    pub message: String,
}

#[derive(Serialize)]
pub struct GenerateTokenResponse {
    pub success: bool,
    pub tailscale_token: Option<String>,
    pub message: String,
}
