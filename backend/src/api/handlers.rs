use axum::{extract::Multipart, Json};
use serde::Serialize;
use tracing::{debug, instrument};

use crate::{
    error::AppError,
    extractor,
    models::DocumentMetadata,
    parser,
    validation,
    xml,
};

#[derive(Serialize)]
pub struct ConvertResponse {
    pub success: bool,
    pub xml: String,
    pub warnings: Vec<String>,
    pub metadata: DocumentMetadata,
}

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

    // Parse DOCX zip archive -> raw XML strings
    let raw_docx = parser::zip::extract_docx(&bytes)?;

    // Parse raw document XML into traversable structure
    let parsed = parser::raw_xml::parse_document_xml(&raw_docx.document_xml)?;

    // Extract document model using heuristics
    let mut document = extractor::extract_document(&parsed, raw_docx.styles_xml.as_deref())?;

    // Generate JATS XML
    let jats_xml = xml::generator::generate_jats(&document)?;

    // Validate well-formedness, append any warnings
    let mut xml_warnings = validation::wellformed::check(&jats_xml);
    document.warnings.append(&mut xml_warnings);

    let metadata = document.metadata();

    Ok(Json(ConvertResponse {
        success: true,
        xml: jats_xml,
        warnings: document.warnings,
        metadata,
    }))
}
