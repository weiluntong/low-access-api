// CLI argument loading logic

use clap::Parser;
use config::{ConfigBuilder, ConfigError, Map, Source, Value, ValueKind};
use config::builder::DefaultState;
use crate::config::models::CliArgs;

/// CLI Source that implements the config::Source trait
#[derive(Debug, Clone)]
struct CliSource {
    cli_args: CliArgs,
}

impl Source for CliSource {
    fn clone_into_box(&self) -> Box<dyn Source + Send + Sync> {
        Box::new(Self {
            cli_args: self.cli_args.clone(),
        })
    }

    fn collect(&self) -> Result<Map<String, Value>, ConfigError> {
        let mut map = Map::new();

        if let Some(bind_address) = &self.cli_args.bind_address {
            map.insert("server.bind_address".to_string(), Value::new(None, ValueKind::String(bind_address.clone())));
        }
        if let Some(log_level) = &self.cli_args.log_level {
            map.insert("server.log_level".to_string(), Value::new(None, ValueKind::String(log_level.clone())));
        }
        if let Some(client_id) = &self.cli_args.google_client_id {
            map.insert("google.client_id".to_string(), Value::new(None, ValueKind::String(client_id.clone())));
        }
        if let Some(oauth_secret_path) = &self.cli_args.tailscale_oauth_secret_path {
            map.insert("tailscale.oauth_secret_path".to_string(), Value::new(None, ValueKind::String(oauth_secret_path.clone())));
        }
        if let Some(api_url) = &self.cli_args.tailscale_api_url {
            map.insert("tailscale.api_url".to_string(), Value::new(None, ValueKind::String(api_url.clone())));
        }
        if !self.cli_args.tailscale_auth_key_tags.is_empty() {
            let array_values: Vec<Value> = self.cli_args.tailscale_auth_key_tags
                .iter()
                .map(|s| Value::new(None, ValueKind::String(s.clone())))
                .collect();
            map.insert("tailscale.auth_key_tags".to_string(), Value::new(None, ValueKind::Array(array_values)));
        }
        if let Some(db_path) = &self.cli_args.database_path {
            map.insert("database.path".to_string(), Value::new(None, ValueKind::String(db_path.clone())));
        }

        Ok(map)
    }
}

/// Get the config file path from CLI arguments
pub fn get_config_file_path() -> String {
    let cli_args = CliArgs::parse();
    cli_args.config
}

/// Load configuration from CLI arguments
pub fn load_from_cli(builder: ConfigBuilder<DefaultState>) -> ConfigBuilder<DefaultState> {
    let cli_args = CliArgs::parse();
    builder.add_source(CliSource { cli_args })
}
