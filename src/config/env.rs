// Environment variable loader

use config::{ConfigBuilder, Environment};
use config::builder::DefaultState;

/// Load configuration from environment variables
///
/// Environment variables use LOW_ACCESS_ prefix with __ as nested separator
///
/// Examples:
/// - LOW_ACCESS_SERVER__BIND_ADDRESS="127.0.0.1:8080"
/// - LOW_ACCESS_GOOGLE__CLIENT_ID="your-client-id.apps.googleusercontent.com"
/// - LOW_ACCESS_TAILSCALE__API_URL="https://api.tailscale.com/api/v2"
/// - LOW_ACCESS_DATABASE__PATH="/var/lib/sso/sso.db"
pub fn load_from_env(builder: ConfigBuilder<DefaultState>)
    -> ConfigBuilder<DefaultState>
{
    builder.add_source(
        Environment::with_prefix("LOW_ACCESS")
            .prefix_separator("_")
            .separator("__")
            .try_parsing(true)
    )
}
