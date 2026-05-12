use axum::{routing::{get, post}, Router};
use super::handlers;

pub fn build_router() -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/convert", post(handlers::convert_handler))
}

async fn health() -> &'static str {
    tracing::debug!("incoming GET /health");
    "ok"
}
