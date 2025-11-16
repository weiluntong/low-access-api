use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct CreateAuthKeyRequest {
    pub capabilities: Capabilities,
    #[serde(rename = "expirySeconds")]
    pub expiry_seconds: u64,
    pub description: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct Capabilities {
    pub devices: DeviceCapabilities,
}

#[derive(Debug, Serialize)]
pub struct DeviceCapabilities {
    pub create: DeviceCreate,
}

#[derive(Debug, Serialize)]
pub struct DeviceCreate {
    pub reusable: bool,
    pub ephemeral: bool,
    pub preauthorized: bool,
    pub tags: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateAuthKeyResponse {
    pub key: String,
}

#[derive(Debug, Deserialize)]
pub struct OAuthTokenResponse {
    pub access_token: String,
    pub expires_in: u64,
    // Note: token_type and scope are also returned but not needed
}

// Cached access token with expiration
pub struct CachedToken {
    pub token: String,
    pub expires_at: u64, // Unix timestamp
}
