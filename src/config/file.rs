// Configuration file loader

use config::{ConfigBuilder, File};
use config::builder::DefaultState;

/// Load configuration from TOML config file
/// File is optional - if not found, will silently skip
pub fn load_from_file(builder: ConfigBuilder<DefaultState>, config_file_path: &str)
    -> ConfigBuilder<DefaultState>
{
    builder.add_source(File::with_name(config_file_path).required(false))
}
