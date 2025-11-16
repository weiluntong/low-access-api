// Configuration data models

pub mod server;
pub mod google;
pub mod tailscale;
pub mod database;
pub mod cli;

pub use server::ServerConfig;
pub use google::GoogleConfig;
pub use tailscale::TailscaleConfig;
pub use database::DatabaseConfig;
pub use cli::CliArgs;
