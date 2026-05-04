//! Local API server crate for know-now.
//!
//! Binds to 127.0.0.1 by default. Launch-token → session-cookie exchange
//! secures browser access. Write endpoints gated behind `allow-generate` feature
//! + runtime flag.

mod api;
mod launch_token;
mod routes;
mod security;
mod session;

use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;

use axum::Router;
use tokio::net::TcpListener;

pub use launch_token::LaunchToken;

#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub host: IpAddr,
    pub port: u16,
    pub allow_generate: bool,
    pub project_root: std::path::PathBuf,
}

impl ServerConfig {
    pub fn is_localhost(&self) -> bool {
        self.host == IpAddr::V4(std::net::Ipv4Addr::LOCALHOST)
            || self.host == IpAddr::V6(std::net::Ipv6Addr::LOCALHOST)
    }
}

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<ServerConfig>,
    pub launch_token: Arc<LaunchToken>,
    pub sessions: Arc<session::SessionStore>,
}

pub struct ServerHandle {
    pub url: String,
    pub launch_url: String,
    shutdown_tx: tokio::sync::oneshot::Sender<()>,
    join_handle: tokio::task::JoinHandle<()>,
}

impl ServerHandle {
    pub fn shutdown(self) {
        let _ = self.shutdown_tx.send(());
    }

    pub async fn wait(self) {
        let _ = self.join_handle.await;
    }
}

/// Start the server and return a handle with the launch URL.
///
/// # Errors
/// Returns an error if the TCP listener cannot bind to the configured address.
pub async fn start_server(config: ServerConfig) -> std::io::Result<ServerHandle> {
    let addr = SocketAddr::new(config.host, config.port);
    let listener = TcpListener::bind(addr).await?;
    let local_addr = listener.local_addr()?;

    let token = LaunchToken::new();
    let launch_path = format!("/__open?launch_token={}", token.value());

    let base_url = format!("http://{local_addr}");
    let launch_url = format!("{base_url}{launch_path}");

    let state = AppState {
        config: Arc::new(config),
        launch_token: Arc::new(token),
        sessions: Arc::new(session::SessionStore::new()),
    };

    let app = build_app(state, local_addr);

    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();

    let join_handle = tokio::spawn(async move {
        axum::serve(listener, app)
            .with_graceful_shutdown(async {
                let _ = shutdown_rx.await;
            })
            .await
            .ok();
    });

    Ok(ServerHandle {
        url: base_url,
        launch_url,
        shutdown_tx,
        join_handle,
    })
}

fn build_app(state: AppState, addr: SocketAddr) -> Router {
    let origin = format!("http://{addr}");

    routes::router(state.clone())
        .merge(api::router())
        .layer(security::cors_layer(&origin))
        .layer(security::x_content_type_options())
        .layer(security::x_frame_options())
        .layer(security::content_security_policy())
        .layer(security::referrer_policy())
        .with_state(state)
}
