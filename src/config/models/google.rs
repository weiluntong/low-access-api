use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct GoogleConfig {
    pub client_id: String,
}
