use std::io::{Cursor, Read};
use zip::ZipArchive;

use crate::{error::AppError, parser::RawDocx};

/// Unzip a .docx byte buffer and extract the relevant XML entry points.
pub fn extract_docx(bytes: &[u8]) -> Result<RawDocx, AppError> {
    let cursor = Cursor::new(bytes);
    let mut archive = ZipArchive::new(cursor)?;

    let document_xml = read_entry(&mut archive, "word/document.xml").map_err(|_| {
        AppError::InvalidDocx("word/document.xml not found in archive".into())
    })?;

    let styles_xml = read_entry(&mut archive, "word/styles.xml").ok();
    let core_xml = read_entry(&mut archive, "docProps/core.xml").ok();

    Ok(RawDocx {
        document_xml,
        styles_xml,
        core_xml,
    })
}

fn read_entry(archive: &mut ZipArchive<Cursor<&[u8]>>, name: &str) -> Result<String, AppError> {
    let mut entry = archive.by_name(name)?;
    let mut content = String::new();
    entry.read_to_string(&mut content)?;
    Ok(content)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use zip::write::SimpleFileOptions;

    fn make_docx(document_xml: &str, include_styles: bool) -> Vec<u8> {
        let buf = Vec::new();
        let cursor = Cursor::new(buf);
        let mut zip = zip::ZipWriter::new(cursor);
        let options = SimpleFileOptions::default();

        zip.start_file("word/document.xml", options).unwrap();
        zip.write_all(document_xml.as_bytes()).unwrap();

        if include_styles {
            zip.start_file("word/styles.xml", options).unwrap();
            zip.write_all(b"<styles/>").unwrap();
        }

        zip.finish().unwrap().into_inner()
    }

    #[test]
    fn test_extract_valid_docx() {
        let bytes = make_docx("<document/>", true);
        let raw = extract_docx(&bytes).unwrap();
        assert_eq!(raw.document_xml, "<document/>");
        assert!(raw.styles_xml.is_some());
    }

    #[test]
    fn test_extract_without_styles() {
        let bytes = make_docx("<document/>", false);
        let raw = extract_docx(&bytes).unwrap();
        assert!(raw.styles_xml.is_none());
    }

    #[test]
    fn test_reject_non_zip() {
        let result = extract_docx(b"not a zip file");
        assert!(result.is_err());
    }

    #[test]
    fn test_missing_document_xml() {
        // ZIP with no word/document.xml entry
        let buf = Vec::new();
        let cursor = Cursor::new(buf);
        let mut zip = zip::ZipWriter::new(cursor);
        let options = SimpleFileOptions::default();
        zip.start_file("other.xml", options).unwrap();
        let bytes = zip.finish().unwrap().into_inner();

        let result = extract_docx(&bytes);
        assert!(matches!(result, Err(AppError::InvalidDocx(_))));
    }
}
