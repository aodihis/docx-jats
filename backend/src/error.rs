use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("ZIP error: {0}")]
    ZipError(#[from] zip::result::ZipError),

    #[error("XML error: {0}")]
    XmlError(#[from] quick_xml::Error),

    #[error("Invalid DOCX: {0}")]
    InvalidDocx(String),

    #[error("Multipart error: {0}")]
    MultipartError(#[from] axum::extract::multipart::MultipartError),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            AppError::InvalidDocx(_) => (StatusCode::UNPROCESSABLE_ENTITY, self.to_string()),
            AppError::MultipartError(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
        };

        tracing::error!("{}", message);

        (status, Json(json!({ "success": false, "error": message }))).into_response()
    }
}
