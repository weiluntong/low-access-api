// Backend Configuration Module
// Configuration is loaded from multiple sources with precedence:
// CLI args > Environment variables > Config file > Built-in defaults

mod models;
mod defaults;
mod file;
mod env;
mod cli;

use serde::Deserialize;
use std::sync::OnceLock;

pub use models::{ServerConfig, GoogleConfig, TailscaleConfig, DatabaseConfig};

/// Main configuration struct containing all application settings
#[derive(Debug, Clone, Deserialize)]
pub struct SsoConfig {
    pub server: ServerConfig,
    pub google: GoogleConfig,
    pub tailscale: TailscaleConfig,
    pub database: DatabaseConfig,
}

impl SsoConfig {
    /// Get the Google OAuth audience for JWT validation
    pub fn audience(&self) -> &str {
        &self.google.client_id
    }

    /// Read Tailscale OAuth secret from file
    pub fn read_tailscale_secret(&self) -> Result<String, std::io::Error> {
        std::fs::read_to_string(&self.tailscale.oauth_secret_path)
            .map(|s| s.trim().to_string())
    }
}

/// Load configuration from all sources
///
/// Loading order (highest to lowest priority):
/// 1. CLI arguments (--bind-address, etc.)
/// 2. Environment variables (LOW_ACCESS_SERVER__BIND_ADDRESS, etc.)
/// 3. Config file (config.toml)
/// 4. Built-in defaults
pub fn load_config() -> Result<SsoConfig, config::ConfigError> {
    use config::Config;

    // Build config from all sources (defaults → file → env → cli)
    // Later sources have higher priority
    let config_file_path = cli::get_config_file_path();
    let builder = Config::builder();
    let builder = defaults::set_defaults(builder)?;
    let builder = file::load_from_file(builder, &config_file_path);
    let builder = env::load_from_env(builder);
    let builder = cli::load_from_cli(builder);
    let config = builder.build()?;

    config.try_deserialize()
}

// Global config instance
static CONFIG: OnceLock<SsoConfig> = OnceLock::new();

/// Get the global configuration singleton
pub fn get_config() -> &'static SsoConfig {
    CONFIG.get_or_init(|| {
        load_config().unwrap_or_else(|e| {
            eprintln!("Failed to load configuration: {}", e);
            std::process::exit(1);
        })
    })
}
