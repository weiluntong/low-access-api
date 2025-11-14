// Backend Configuration
// Loads configuration from multiple sources with precedence:
// CLI args > Environment variables > Config file > Built-in defaults

use serde::Deserialize;
use config::{Config, ConfigError, Environment, File};
use clap::Parser;
use std::sync::OnceLock;

/// SSO Backend Server
#[derive(Parser, Debug)]
#[command(name = "low-access-api")]
#[command(about = "SSO authentication backend for LoW Net", long_about = None)]
struct CliArgs {
    /// Path to config file
    #[arg(short, long, default_value = "config.toml")]
    config: String,

    /// Server bind address (IP:PORT)
    #[arg(long)]
    bind_address: Option<String>,

    /// Google OAuth Client ID
    #[arg(long)]
    google_client_id: Option<String>,

    /// Path to Tailscale OAuth secret file
    #[arg(long)]
    tailscale_oauth_secret_path: Option<String>,

    /// Tailscale API base URL
    #[arg(long)]
    tailscale_api_url: Option<String>,

    /// Database file path
    #[arg(long)]
    database_path: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    pub bind_address: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GoogleConfig {
    pub client_id: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TailscaleConfig {
    pub oauth_secret_path: String,
    pub api_url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    pub path: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SsoConfig {
    pub server: ServerConfig,
    pub google: GoogleConfig,
    pub tailscale: TailscaleConfig,
    pub database: DatabaseConfig,
}

impl SsoConfig {
    pub fn load() -> Result<Self, ConfigError> {
        let cli_args = CliArgs::parse();

        let config = Config::builder()
            // Built-in defaults (only for reasonable defaults that work across deployments)
            .set_default("server.bind_address", "0.0.0.0:3000")?
            .set_default("tailscale.api_url", "https://api.tailscale.com")?
            .set_default("database.path", "sso.db")?
            // google.client_id - REQUIRED, no default (must be user's own client ID)
            // tailscale.oauth_secret_path - REQUIRED, no default (must be configured for deployment)
            // Layer 1: Config file (if exists)
            .add_source(File::with_name(&cli_args.config).required(false))
            // Layer 2: Environment variables (with prefix LOW_ACCESS_)
            // e.g., LOW_ACCESS_SERVER__BIND_ADDRESS=..., LOW_ACCESS_GOOGLE__CLIENT_ID=...
            .add_source(
                Environment::with_prefix("LOW_ACCESS")
                    .prefix_separator("_")
                    .separator("__")
                    .try_parsing(true)
            )
            .build()?;

        let mut sso_config: SsoConfig = config.try_deserialize()?;

        // Layer 3: CLI arguments (highest priority)
        if let Some(bind_address) = cli_args.bind_address {
            sso_config.server.bind_address = bind_address;
        }
        if let Some(client_id) = cli_args.google_client_id {
            sso_config.google.client_id = client_id;
        }
        if let Some(oauth_secret_path) = cli_args.tailscale_oauth_secret_path {
            sso_config.tailscale.oauth_secret_path = oauth_secret_path;
        }
        if let Some(api_url) = cli_args.tailscale_api_url {
            sso_config.tailscale.api_url = api_url;
        }
        if let Some(db_path) = cli_args.database_path {
            sso_config.database.path = db_path;
        }

        Ok(sso_config)
    }

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

// Global config instance
static CONFIG: OnceLock<SsoConfig> = OnceLock::new();

pub fn get_config() -> &'static SsoConfig {
    CONFIG.get_or_init(|| {
        SsoConfig::load().unwrap_or_else(|e| {
            eprintln!("Failed to load configuration: {}", e);
            std::process::exit(1);
        })
    })
}
