mod abstract_;
mod references;
mod sections;
mod title;

use crate::{error::AppError, models::Document, parser::raw_xml::ParsedXml};

/// Build a `Document` from parsed paragraphs using heuristic extraction.
///
/// Each sub-extractor runs independently and appends warnings rather than failing.
pub fn extract_document(parsed: &ParsedXml, _styles_xml: Option<&str>) -> Result<Document, AppError> {
    let mut doc = Document::new();

    doc.title = title::extract_title(parsed, &mut doc.warnings);
    doc.abstract_text = abstract_::extract_abstract(parsed, &mut doc.warnings);
    doc.sections = sections::extract_sections(parsed, &mut doc.warnings);
    doc.references = references::extract_references(parsed, &mut doc.warnings);

    if doc.title.is_none() {
        doc.add_warning("Could not determine document title");
    }
    if doc.abstract_text.is_none() {
        doc.add_warning("No abstract found");
    }

    Ok(doc)
}
