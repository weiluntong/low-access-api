use axum::{
    extract::State,
    http::{StatusCode, HeaderMap},
    response::Json,
};
use sqlx::SqlitePool;
use tracing::{info, warn, error};
use crate::models::{User, GenerateTokenRequest, ValidateTokenResponse, GenerateTokenResponse};
use crate::{google, db, tailscale};

pub async fn health_check() -> &'static str {
    "LoW Access API is running!"
}

pub async fn validate_token(
    State(pool): State<SqlitePool>,
    headers: HeaderMap,
) -> Result<Json<ValidateTokenResponse>, StatusCode> {
    info!("Received token validation request");

    // Extract token from Authorization header
    let auth_header = headers.get("authorization")
        .and_then(|value| value.to_str().ok())
        .ok_or_else(|| {
            info!("Missing or invalid Authorization header");
            StatusCode::UNAUTHORIZED
        })?;

    let token = if auth_header.starts_with("Bearer ") {
        &auth_header[7..] // Remove "Bearer " prefix
    } else {
        info!("Authorization header doesn't start with 'Bearer '");
        return Err(StatusCode::UNAUTHORIZED);
    };

    // Step 1: Validate the Google token
    let user = match google::validate_google_id_token(token).await {
        Ok(user) => user,
        Err(e) => {
            info!("Token validation failed: {}", e);
            return Ok(Json(ValidateTokenResponse {
                success: false,
                user: None,
                message: format!("Invalid token: {}", e),
            }));
        }
    };

    // Step 2: Check if user is authorized in our database
    match check_user_authorization(&pool, &user).await {
        Ok(authorized_user) => {
            info!("User {} is authorized and logged in", authorized_user.email);
            Ok(Json(ValidateTokenResponse {
                success: true,
                user: Some(authorized_user),
                message: "Authentication and authorization successful".to_string(),
            }))
        }
        Err(e) => {
            info!("User {} authorization failed: {}", user.email, e);
            Ok(Json(ValidateTokenResponse {
                success: false,
                user: None,
                message: format!("Access denied: {}", e),
            }))
        }
    }
}

async fn check_user_authorization(pool: &SqlitePool, user: &User) -> Result<User, String> {
    // Check if user exists in our database
    let existing_user = sqlx::query_as::<_, User>(
        "SELECT id, email, name, status, created_at, last_login FROM users WHERE email = ?"
    )
    .bind(&user.email)
    .fetch_optional(pool)
    .await
    .map_err(|e| {
        // Provide more specific error messages based on the error type
        match e {
            sqlx::Error::Io(_) => "Database connection failed. Please try again later.".to_string(),
            sqlx::Error::Database(_) => "Database query failed. The service may be temporarily unavailable.".to_string(),
            sqlx::Error::Tls(_) => "Database connection security error. Please contact support.".to_string(),
            sqlx::Error::Protocol(_) => "Database communication error. Please try again.".to_string(),
            sqlx::Error::PoolTimedOut => "Database is overloaded. Please try again in a moment.".to_string(),
            sqlx::Error::PoolClosed => "Database service is currently unavailable. Please try again later.".to_string(),
            _ => format!("Database service error: {}. Please contact support if this persists.", e),
        }
    })?;

    match existing_user {
        Some(mut db_user) => {
            // User exists - update login time and return with their current status
            db_user.last_login = chrono::Utc::now();
            
            if let Err(e) = db::upsert_user(pool, &db_user).await {
                warn!("Failed to update user login time: {}", e);
            }

            Ok(db_user)
        }
        None => {
            // User doesn't exist - CREATE THEM with 'pending' status
            let now = chrono::Utc::now();
            let new_user = User {
                id: user.id.clone(),
                email: user.email.clone(),
                name: user.name.clone(),
                status: "pending".to_string(),  // Default to pending approval
                created_at: now,
                last_login: now,
            };

            // Insert the new user into database
            if let Err(e) = db::upsert_user(pool, &new_user).await {
                return Err(match e.downcast_ref::<sqlx::Error>() {
                    Some(sqlx::Error::Database(_)) => "Unable to create user account due to database constraints. Please contact support.".to_string(),
                    Some(sqlx::Error::Io(_)) => "Database connection lost while creating account. Please try signing in again.".to_string(),
                    Some(sqlx::Error::PoolTimedOut) => "Database is busy. Please try again in a moment.".to_string(),
                    Some(sqlx::Error::PoolClosed) => "Database service unavailable. Please try again later.".to_string(),
                    _ => "Unable to create user account. Please try again or contact support if this persists.".to_string(),
                });
            }

            info!("New user {} created with pending status", new_user.email);
            // Return the new user so frontend can show pending page
            Ok(new_user)
        }
    }
}

pub async fn generate_tailscale_token(
    State(pool): State<SqlitePool>,
    Json(payload): Json<GenerateTokenRequest>,
) -> Result<Json<GenerateTokenResponse>, StatusCode> {
    // Validate the Google ID token and get user info
    let user = match google::validate_google_id_token(&payload.id_token).await {
        Ok(user) => user,
        Err(e) => {
            info!("Token validation failed: {}", e);
            return Ok(Json(GenerateTokenResponse {
                success: false,
                tailscale_token: None,
                message: format!("Invalid token: {}", e),
            }));
        }
    };

    // Check user authorization (this also validates their current status)
    let authorized_user = match check_user_authorization(&pool, &user).await {
        Ok(user) => user,
        Err(e) => {
            return Ok(Json(GenerateTokenResponse {
                success: false,
                tailscale_token: None,
                message: e,
            }));
        }
    };

    // Only approved users can generate tokens
    if authorized_user.status != "approved" {
        let message = match authorized_user.status.as_str() {
            "pending" => "Your account is pending approval. Cannot generate tokens yet.".to_string(),
            "denied" => "Your account has been denied access. Cannot generate tokens.".to_string(),
            _ => "Account status does not allow token generation.".to_string(),
        };

        return Ok(Json(GenerateTokenResponse {
            success: false,
            tailscale_token: None,
            message,
        }));
    }

    // Generate Tailscale auth key
    match tailscale::generate_auth_key(&authorized_user.email).await {
        Ok(auth_key) => {
            Ok(Json(GenerateTokenResponse {
                success: true,
                tailscale_token: Some(auth_key),
                message: "Tailscale auth key generated successfully".to_string(),
            }))
        }
        Err(e) => {
            error!("Failed to generate Tailscale auth key for {}: {}", authorized_user.email, e);
            Ok(Json(GenerateTokenResponse {
                success: false,
                tailscale_token: None,
                message: "Unable to generate network access token. Please try again later or contact support if this persists.".to_string(),
            }))
        }
    }
}
