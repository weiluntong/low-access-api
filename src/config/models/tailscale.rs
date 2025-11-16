use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct TailscaleConfig {
    pub oauth_secret_path: String,
    pub api_url: String,
    #[serde(default)]
    pub auth_key_tags: Vec<String>,
}
