use axum::{routing::{get, post}, Json, Router};
use axum::response::Html;
use utoipa::OpenApi;

use super::handlers;
use crate::models::{Author, DocumentContent, DocumentMetadata, Reference, Section};

#[derive(OpenApi)]
#[openapi(
    info(
        title = "docx-jats API",
        description = "Convert DOCX research manuscripts to JATS XML.",
        version = "0.1.0",
    ),
    paths(health, handlers::convert_handler, handlers::regenerate_handler),
    components(schemas(
        handlers::ConvertResponse,
        handlers::RegenerateRequest,
        handlers::RegenerateResponse,
        DocumentMetadata,
        DocumentContent,
        Author,
        Section,
        Reference,
    )),
    tags(
        (name = "health",     description = "Liveness check"),
        (name = "conversion", description = "DOCX → JATS XML conversion"),
    )
)]
struct ApiDoc;

async fn openapi_json() -> Json<utoipa::openapi::OpenApi> {
    Json(ApiDoc::openapi())
}

async fn swagger_ui() -> Html<&'static str> {
    Html(include_str!("../../static/swagger_ui.html"))
}

pub fn build_router() -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/convert", post(handlers::convert_handler))
        .route("/regenerate", post(handlers::regenerate_handler))
        .route("/api-docs/openapi.json", get(openapi_json))
        .route("/swagger-ui", get(swagger_ui))
}

#[utoipa::path(
    get,
    path = "/health",
    responses(
        (status = 200, description = "Server is running", body = str),
    ),
    tag = "health"
)]
async fn health() -> &'static str {
    tracing::debug!("incoming GET /health");
    "ok"
}
