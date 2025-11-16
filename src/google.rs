use reqwest;
use jsonwebtoken::{decode, decode_header, DecodingKey, Validation, Algorithm};
use anyhow::{Result, anyhow};
use chrono::{Utc, DateTime};
use std::sync::{Arc, Mutex};
use std::sync::OnceLock;
use crate::models::{GoogleIdTokenClaims, GoogleJwks, GoogleJwk, User};
use crate::config::get_config;

// Thread-safe cache for Google's public keys
static GOOGLE_KEYS_CACHE: OnceLock<Arc<Mutex<Option<(DateTime<Utc>, GoogleJwks)>>>> = OnceLock::new();
const CACHE_DURATION_HOURS: i64 = 24;

fn get_cache() -> &'static Arc<Mutex<Option<(DateTime<Utc>, GoogleJwks)>>> {
    GOOGLE_KEYS_CACHE.get_or_init(|| Arc::new(Mutex::new(None)))
}

pub async fn validate_google_id_token(id_token: &str) -> Result<User> {
    // Decode the header to get the key ID
    let header = decode_header(id_token)?;
    let kid = header.kid.ok_or_else(|| anyhow!("Token missing key ID"))?;

    // Get Google's public keys
    let jwks = get_google_jwks().await?;
    
    // Find the matching key
    let jwk = jwks.keys.iter()
        .find(|key| key.kid == kid)
        .ok_or_else(|| anyhow!("Key ID not found in Google JWKS"))?;

    // Convert JWK to DecodingKey
    let decoding_key = jwk_to_decoding_key(jwk)?;

    // Set up validation parameters
    let mut validation = Validation::new(Algorithm::RS256);
    validation.set_audience(&[get_config().audience()]);
    validation.set_issuer(&["https://accounts.google.com", "accounts.google.com"]);

    // Validate the token
    let token_data = decode::<GoogleIdTokenClaims>(id_token, &decoding_key, &validation)?;
    let claims = token_data.claims;

    // Create user from claims
    let now = Utc::now();
    let user = User {
        id: claims.sub,
        email: claims.email,
        name: claims.name.unwrap_or_else(|| "Unknown".to_string()),
        status: "pending".to_string(), // Default status for new validation
        created_at: now,
        last_login: now,
    };

    Ok(user)
}

async fn get_google_jwks() -> Result<GoogleJwks> {
    let cache = get_cache();
    
    // Check cache first
    {
        let cache_guard = cache.lock().unwrap();
        if let Some((cached_at, jwks)) = cache_guard.as_ref() {
            if Utc::now().signed_duration_since(*cached_at).num_hours() < CACHE_DURATION_HOURS {
                return Ok(jwks.clone());
            }
        }
    } // Lock is released here

    // Fetch fresh keys from Google
    let response = reqwest::get("https://www.googleapis.com/oauth2/v3/certs").await?;
    let jwks: GoogleJwks = response.json().await?;

    // Update cache
    {
        let mut cache_guard = cache.lock().unwrap();
        *cache_guard = Some((Utc::now(), jwks.clone()));
    }

    Ok(jwks)
}

fn jwk_to_decoding_key(jwk: &GoogleJwk) -> Result<DecodingKey> {
    // Convert base64url encoded modulus and exponent to base64url strings
    // The jsonwebtoken crate expects base64url encoded strings, not raw bytes
    DecodingKey::from_rsa_components(&jwk.n, &jwk.e).map_err(|e| anyhow!("Failed to create decoding key: {}", e))
}
