pub mod raw_xml;
pub mod zip;

/// Raw contents extracted from a .docx ZIP archive.
#[derive(Debug)]
pub struct RawDocx {
    pub document_xml: String,
    pub styles_xml: Option<String>,
    /// docProps/core.xml — contains Dublin Core metadata (creator, title, …)
    pub core_xml: Option<String>,
}
