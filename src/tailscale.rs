use reqwest;
use anyhow::{Result, anyhow};
use tracing::{info, error, debug};
use crate::config::get_config;
use crate::models::{
    CreateAuthKeyRequest, CreateAuthKeyResponse, OAuthTokenResponse, CachedToken,
    Capabilities, DeviceCapabilities, DeviceCreate,
};
use std::sync::OnceLock;
use tokio::sync::RwLock;
use std::time::{SystemTime, UNIX_EPOCH};

// Global token cache
static TOKEN_CACHE: OnceLock<RwLock<Option<CachedToken>>> = OnceLock::new();

fn get_token_cache() -> &'static RwLock<Option<CachedToken>> {
    TOKEN_CACHE.get_or_init(|| RwLock::new(None))
}

/// Exchange OAuth client credentials for a temporary access token
///
/// Tailscale OAuth uses the client credentials grant flow. The access token
/// expires after 1 hour. This function caches the token and reuses it until
/// it expires, reducing unnecessary API calls when multiple users sign up together.
async fn get_oauth_access_token() -> Result<String> {
    let cache = get_token_cache();
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    // Check if we have a valid cached token (with 60s buffer before expiry)
    {
        let cache_read = cache.read().await;
        if let Some(cached) = cache_read.as_ref() {
            if cached.expires_at > now + 60 {
                debug!("Using cached OAuth access token (expires in {} seconds)", cached.expires_at - now);
                return Ok(cached.token.clone());
            } else {
                debug!("Cached OAuth token expired, fetching new one");
            }
        }
    }

    // No valid cached token, fetch a new one
    let config = get_config();

    // Read the OAuth client secret from file
    let client_secret = config.read_tailscale_secret()
        .map_err(|e| anyhow!("Failed to read Tailscale OAuth secret: {}", e))?;

    let token_url = format!("{}/oauth/token", config.tailscale.api_url);

    debug!("Exchanging OAuth credentials for access token");

    // Make OAuth token exchange request
    // Note: Only client_secret is sent. OAuth 2.0 spec requires client_id too,
    // but Tailscale's implementation doesn't enforce it.
    let client = reqwest::Client::new();
    let response = client
        .post(&token_url)
        .form(&[("client_secret", client_secret.as_str())])
        .send()
        .await
        .map_err(|e| anyhow!("Failed to exchange OAuth credentials: {}", e))?;

    let status = response.status();

    if !status.is_success() {
        let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        error!("OAuth token exchange failed ({}): {}", status, error_text);
        return Err(anyhow!("OAuth token exchange failed {}: {}", status, error_text));
    }

    let token_response: OAuthTokenResponse = response
        .json()
        .await
        .map_err(|e| anyhow!("Failed to parse OAuth token response: {}", e))?;

    debug!("Successfully obtained OAuth access token (expires in {} seconds)", token_response.expires_in);

    // Cache the new token
    let expires_at = now + token_response.expires_in;
    {
        let mut cache_write = cache.write().await;
        *cache_write = Some(CachedToken {
            token: token_response.access_token.clone(),
            expires_at,
        });
    }

    Ok(token_response.access_token)
}

/// Generate a Tailscale auth key for a user
///
/// This creates a reusable, preauthorized auth key that expires in 2 hours.
/// The key allows the user to register their device on the tailnet as a non-ephemeral device.
pub async fn generate_auth_key(user_email: &str) -> Result<String> {
    let config = get_config();

    // Step 1: Exchange OAuth client credentials for access token
    let access_token = get_oauth_access_token().await?;

    let api_url = format!("{}/tailnet/-/keys", config.tailscale.api_url);

    // Create auth key request
    let request_body = CreateAuthKeyRequest {
        capabilities: Capabilities {
            devices: DeviceCapabilities {
                create: DeviceCreate {
                    reusable: true,
                    ephemeral: false,
                    preauthorized: true,
                    tags: config.tailscale.auth_key_tags.clone(),
                },
            },
        },
        expiry_seconds: 7200,
        // Tailscale requires alphanumeric + hyphen/space only in descriptions
        description: Some(format!("Auth key for user {}",
            user_email.chars()
                .map(|c| if c.is_alphanumeric() { c } else { '-' })
                .collect::<String>()
        )),
    };

    debug!("Creating auth key with tags: {:?}", config.tailscale.auth_key_tags);

    // Make API request to create auth key using the access token
    let client = reqwest::Client::new();
    let response = client
        .post(&api_url)
        .header("Authorization", format!("Bearer {}", access_token))
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await
        .map_err(|e| anyhow!("Failed to send request to Tailscale API: {}", e))?;

    let status = response.status();

    if !status.is_success() {
        let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        error!("Tailscale API error ({}): {}", status, error_text);
        return Err(anyhow!("Tailscale API returned error {}: {}", status, error_text));
    }

    let auth_key_response: CreateAuthKeyResponse = response
        .json()
        .await
        .map_err(|e| anyhow!("Failed to parse Tailscale API response: {}", e))?;

    info!("Successfully generated Tailscale auth key for user: {}", user_email);

    Ok(auth_key_response.key)
}
