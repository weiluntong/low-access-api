use axum::{
    routing::{get, post},
    Router,
};
use tower_http::cors::CorsLayer;
use tracing::{info, level_filters::LevelFilter};
use std::net::SocketAddr;

mod google;
mod config;
mod db;
mod models;
mod handlers;

use config::get_config;
use handlers::{health_check, validate_token, generate_tailscale_token};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(LevelFilter::INFO)
        .init();

    // Initialize database
    let db = db::init_db().await?;
    
    info!("Database initialized successfully");

    // Build our application with routes
    let app = Router::new()
        .route("/", get(health_check))
        .route("/auth/validate", get(validate_token))
        .route("/auth/generate-token", post(generate_tailscale_token))
        .layer(CorsLayer::permissive()) // Allow CORS for frontend
        .with_state(db);

    // Run the server
    let config = get_config();
    let addr: SocketAddr = config.server.bind_address.parse()
        .map_err(|_| anyhow::anyhow!("Invalid bind address: {}", config.server.bind_address))?;
    info!("Server starting on {}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
