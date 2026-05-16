use axum::{extract::Multipart, Json};
use serde::{Deserialize, Serialize};
use tracing::{debug, instrument};
use utoipa::ToSchema;

const MAX_FILE_BYTES: usize = 25 * 1024 * 1024; // 25 MB

use crate::{
    error::AppError,
    extractor,
    models::{Document, DocumentContent, DocumentMetadata},
    parser,
    validation,
    xml,
};

#[derive(Serialize, ToSchema)]
pub struct ConvertResponse {
    pub success: bool,
    pub xml: String,
    pub warnings: Vec<String>,
    pub metadata: DocumentMetadata,
    pub document: DocumentContent,
}

#[derive(Deserialize, ToSchema)]
pub struct RegenerateRequest {
    pub document: DocumentContent,
}

#[derive(Serialize, ToSchema)]
pub struct RegenerateResponse {
    pub success: bool,
    pub xml: String,
    pub warnings: Vec<String>,
}

#[utoipa::path(
    post,
    path = "/convert",
    request_body(description = "Multipart form-data with a `file` field containing the .docx file."),
    responses(
        (status = 200, description = "Successful conversion", body = ConvertResponse),
        (status = 400, description = "Missing `file` field in multipart body"),
        (status = 422, description = "File is not a valid DOCX"),
        (status = 500, description = "Internal server error"),
    ),
    tag = "conversion"
)]
#[instrument(skip(multipart))]
pub async fn convert_handler(mut multipart: Multipart) -> Result<Json<ConvertResponse>, AppError> {
    debug!("incoming POST /convert");

    let mut file_bytes: Option<Vec<u8>> = None;

    while let Some(field) = multipart.next_field().await? {
        if field.name() == Some("file") {
            let data = field.bytes().await?;
            debug!(bytes = data.len(), "file field received");
            file_bytes = Some(data.to_vec());
            break;
        }
    }

    let bytes = file_bytes.ok_or_else(|| AppError::InvalidDocx("No file field in request".into()))?;

    if bytes.len() > MAX_FILE_BYTES {
        return Err(AppError::InvalidDocx(format!(
            "File too large: {} MB. Maximum allowed size is 25 MB.",
            bytes.len() / 1024 / 1024
        )));
    }

    // Parse DOCX zip archive -> raw XML strings
    let raw_docx = parser::zip::extract_docx(&bytes)?;

    // Parse raw document XML into traversable structure
    let parsed = parser::raw_xml::parse_document_xml(&raw_docx.document_xml)?;

    // Extract document model using heuristics
    let mut document = extractor::extract_document(&parsed, raw_docx.styles_xml.as_deref(), raw_docx.core_xml.as_deref())?;

    // Generate JATS XML
    let jats_xml = xml::generator::generate_jats(&document)?;

    // Validate well-formedness, append any warnings
    let mut xml_warnings = validation::wellformed::check(&jats_xml);
    document.warnings.append(&mut xml_warnings);

    let metadata = document.metadata();
    let document_content = DocumentContent::from(&document);

    Ok(Json(ConvertResponse {
        success: true,
        xml: jats_xml,
        warnings: document.warnings,
        metadata,
        document: document_content,
    }))
}

#[utoipa::path(
    post,
    path = "/regenerate",
    request_body(content = RegenerateRequest, description = "Edited document content to regenerate JATS XML from."),
    responses(
        (status = 200, description = "Regenerated JATS XML", body = RegenerateResponse),
        (status = 422, description = "Invalid document content"),
        (status = 500, description = "Internal server error"),
    ),
    tag = "conversion"
)]
#[instrument(skip(payload))]
pub async fn regenerate_handler(Json(payload): Json<RegenerateRequest>) -> Result<Json<RegenerateResponse>, AppError> {
    debug!("incoming POST /regenerate");

    let mut document = Document::from(payload.document);

    let jats_xml = xml::generator::generate_jats(&document)?;

    let mut xml_warnings = validation::wellformed::check(&jats_xml);
    document.warnings.append(&mut xml_warnings);

    Ok(Json(RegenerateResponse {
        success: true,
        xml: jats_xml,
        warnings: document.warnings,
    }))
}
