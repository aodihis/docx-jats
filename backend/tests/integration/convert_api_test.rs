use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum_test::TestServer;
use std::io::{Cursor, Write};
use zip::write::SimpleFileOptions;

fn build_router() -> axum::Router {
    docx_jast_backend::api::routes::build_router()
}

/// Build a minimal valid .docx byte vector in memory.
fn make_minimal_docx(document_xml: &str) -> Vec<u8> {
    let buf = Vec::new();
    let cursor = Cursor::new(buf);
    let mut zip = zip::ZipWriter::new(cursor);
    let options = SimpleFileOptions::default();

    // Required entry
    zip.start_file("word/document.xml", options).unwrap();
    zip.write_all(document_xml.as_bytes()).unwrap();

    // Optional styles
    zip.start_file("word/styles.xml", options).unwrap();
    zip.write_all(b"<w:styles/>").unwrap();

    zip.finish().unwrap().into_inner()
}

/// A realistic minimal word/document.xml with heading + abstract + body + references.
fn realistic_document_xml() -> String {
    r#"<?xml version="1.0" encoding="UTF-8"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:body>
    <w:p>
      <w:pPr><w:pStyle w:val="Title"/></w:pPr>
      <w:r><w:t>A Study on Testing</w:t></w:r>
    </w:p>
    <w:p>
      <w:pPr><w:pStyle w:val="Heading1"/></w:pPr>
      <w:r><w:t>Abstract</w:t></w:r>
    </w:p>
    <w:p>
      <w:pPr><w:pStyle w:val="Normal"/></w:pPr>
      <w:r><w:t>This paper investigates testing methodology.</w:t></w:r>
    </w:p>
    <w:p>
      <w:pPr><w:pStyle w:val="Heading1"/></w:pPr>
      <w:r><w:t>Introduction</w:t></w:r>
    </w:p>
    <w:p>
      <w:pPr><w:pStyle w:val="Normal"/></w:pPr>
      <w:r><w:t>Testing is important.</w:t></w:r>
    </w:p>
    <w:p>
      <w:pPr><w:pStyle w:val="Heading1"/></w:pPr>
      <w:r><w:t>References</w:t></w:r>
    </w:p>
    <w:p>
      <w:pPr><w:pStyle w:val="Normal"/></w:pPr>
      <w:r><w:t>Smith, J. (2020). Testing things. Journal of Tests.</w:t></w:r>
    </w:p>
  </w:body>
</w:document>"#
        .to_string()
}

fn ambiguous_document_xml() -> String {
    // No recognisable title or abstract styles — triggers warnings
    r#"<?xml version="1.0" encoding="UTF-8"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:body>
    <w:p><w:r><w:t>Just some text with no styles at all.</w:t></w:r></w:p>
    <w:p><w:r><w:t>Another paragraph.</w:t></w:r></w:p>
  </w:body>
</w:document>"#
        .to_string()
}

// ── helpers ──────────────────────────────────────────────────────────────────

fn multipart_body(docx_bytes: &[u8]) -> (String, Vec<u8>) {
    let boundary = "TestBoundary1234";
    let mut body = Vec::new();

    // Part header
    write!(body, "--{boundary}\r\n").unwrap();
    write!(
        body,
        "Content-Disposition: form-data; name=\"file\"; filename=\"test.docx\"\r\n"
    )
    .unwrap();
    write!(body, "Content-Type: application/vnd.openxmlformats-officedocument.wordprocessingml.document\r\n\r\n").unwrap();
    body.extend_from_slice(docx_bytes);
    write!(body, "\r\n--{boundary}--\r\n").unwrap();

    (
        format!("multipart/form-data; boundary={boundary}"),
        body,
    )
}

// ── tests ─────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_health_endpoint() {
    let server = TestServer::new(build_router()).unwrap();
    let response = server.get("/health").await;
    assert_eq!(response.status_code(), StatusCode::OK);
}

#[tokio::test]
async fn test_convert_minimal_docx_success() {
    let server = TestServer::new(build_router()).unwrap();
    let docx = make_minimal_docx(&realistic_document_xml());
    let (content_type, body) = multipart_body(&docx);

    let response = server
        .post("/convert")
        .content_type(&content_type)
        .bytes(body.into())
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);

    let json: serde_json::Value = response.json();
    assert_eq!(json["success"], true);
    assert!(!json["xml"].as_str().unwrap_or("").is_empty());
    assert!(json["xml"].as_str().unwrap().contains("<article"));
}

#[tokio::test]
async fn test_convert_returns_metadata() {
    let server = TestServer::new(build_router()).unwrap();
    let docx = make_minimal_docx(&realistic_document_xml());
    let (content_type, body) = multipart_body(&docx);

    let response = server
        .post("/convert")
        .content_type(&content_type)
        .bytes(body.into())
        .await;

    let json: serde_json::Value = response.json();
    assert!(json["metadata"].is_object());
    assert!(json["metadata"]["section_count"].as_u64().unwrap_or(0) > 0);
}

#[tokio::test]
async fn test_convert_invalid_file_returns_error() {
    let server = TestServer::new(build_router()).unwrap();
    let (content_type, body) = multipart_body(b"this is definitely not a docx file");

    let response = server
        .post("/convert")
        .content_type(&content_type)
        .bytes(body.into())
        .await;

    // Should be a client or server error, not 200
    assert_ne!(response.status_code(), StatusCode::OK);
    let json: serde_json::Value = response.json();
    assert_eq!(json["success"], false);
}

#[tokio::test]
async fn test_convert_missing_file_field_returns_error() {
    let server = TestServer::new(build_router()).unwrap();

    // Empty multipart with no "file" field
    let boundary = "TestBoundary1234";
    let body = format!("--{boundary}--\r\n");
    let content_type = format!("multipart/form-data; boundary={boundary}");

    let response = server
        .post("/convert")
        .content_type(&content_type)
        .bytes(body.into_bytes().into())
        .await;

    assert_ne!(response.status_code(), StatusCode::OK);
}

#[tokio::test]
async fn test_warnings_returned_for_ambiguous_doc() {
    let server = TestServer::new(build_router()).unwrap();
    let docx = make_minimal_docx(&ambiguous_document_xml());
    let (content_type, body) = multipart_body(&docx);

    let response = server
        .post("/convert")
        .content_type(&content_type)
        .bytes(body.into())
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let json: serde_json::Value = response.json();
    let warnings = json["warnings"].as_array().unwrap();
    assert!(!warnings.is_empty(), "expected warnings for ambiguous document");
}
