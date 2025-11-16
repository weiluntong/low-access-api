// CLI argument data structure

use clap::Parser;

/// SSO Backend Server CLI Arguments
#[derive(Parser, Debug, Clone)]
#[command(name = "low-access-api")]
#[command(about = "SSO authentication backend for LoW Net", long_about = None)]
pub struct CliArgs {
    /// Path to config file
    #[arg(short, long, default_value = "config.toml")]
    pub config: String,

    /// Server bind address (IP:PORT)
    #[arg(long)]
    pub bind_address: Option<String>,

    /// Google OAuth Client ID
    #[arg(long)]
    pub google_client_id: Option<String>,

    /// Path to Tailscale OAuth secret file
    #[arg(long)]
    pub tailscale_oauth_secret_path: Option<String>,

    /// Tailscale API base URL
    #[arg(long)]
    pub tailscale_api_url: Option<String>,

    /// Tailscale auth key tags (can be specified multiple times)
    #[arg(long = "tailscale-auth-key-tag")]
    pub tailscale_auth_key_tags: Vec<String>,

    /// Database file path
    #[arg(long)]
    pub database_path: Option<String>,

    /// Log level (trace, debug, info, warn, error)
    #[arg(long)]
    pub log_level: Option<String>,
}
