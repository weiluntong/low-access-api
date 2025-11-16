// Built-in default configuration values

use config::{ConfigBuilder, ConfigError};
use config::builder::DefaultState;

/// Set built-in defaults for all configuration values
/// These are the lowest priority and will be overridden by file, env, and CLI
pub fn set_defaults(builder: ConfigBuilder<DefaultState>)
    -> Result<ConfigBuilder<DefaultState>, ConfigError>
{
    Ok(builder
        .set_default("server.bind_address", "0.0.0.0:3000")?
        .set_default("server.log_level", "info")?
        .set_default("tailscale.api_url", "https://api.tailscale.com/api/v2")?
        .set_default("database.path", "sso.db")?)
    // Note: google.client_id and tailscale.oauth_secret_path are REQUIRED (no defaults)
}
