// Backend Configuration
// This file contains configuration values that should match the frontend config

#[derive(Debug, Clone)]
pub struct SsoConfig {
    /// Google OAuth Client ID - must match frontend
    pub google_client_id: String,
    
    /// Backend server bind address
    pub bind_address: String,
}

impl SsoConfig {
    pub fn new() -> Self {
        Self {
            // Google OAuth Client ID (must match frontend config)
            google_client_id: "699013215587-keu4ptegcvp29456ucc0eed1dd6u2cg9.apps.googleusercontent.com".to_string(),
            
            // Server bind address
            bind_address: "127.0.0.1:3000".to_string(),
        }
    }
    
    /// Get the full audience string for JWT validation
    pub fn audience(&self) -> &str {
        &self.google_client_id
    }
}

// Global config instance
use std::sync::OnceLock;
static CONFIG: OnceLock<SsoConfig> = OnceLock::new();

pub fn get_config() -> &'static SsoConfig {
    CONFIG.get_or_init(|| SsoConfig::new())
}