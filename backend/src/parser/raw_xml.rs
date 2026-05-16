use quick_xml::{escape::unescape, events::Event, Reader};

use crate::error::AppError;

/// A single logical paragraph extracted from word/document.xml.
#[derive(Debug, Clone)]
pub struct Paragraph {
    /// Resolved style name (e.g. "Heading1", "Normal", "Abstract").
    pub style: Option<String>,
    /// Plain text content, runs concatenated.
    pub text: String,
}

/// The result of parsing word/document.xml: an ordered list of paragraphs.
#[derive(Debug)]
pub struct ParsedXml {
    pub paragraphs: Vec<Paragraph>,
}

/// Parse word/document.xml into a flat list of paragraphs.
///
/// DOCX structure (simplified):
///   <w:body>
///     <w:p>                    <- paragraph
///       <w:pPr><w:pStyle w:val="Heading1"/></w:pPr>
///       <w:r><w:t>text</w:t></w:r>  <- run
///     </w:p>
///   </w:body>
pub fn parse_document_xml(xml: &str) -> Result<ParsedXml, AppError> {
    let mut reader = Reader::from_str(xml);
    // Do NOT trim_text — it trims each event independently and would collapse
    // the space around entity-split text (e.g. "B., " + " Zibbitova" from &amp;).
    // We only capture text inside <w:t> (guarded by in_text), so structural
    // whitespace-only events are already ignored by the state machine.

    let mut paragraphs: Vec<Paragraph> = Vec::new();

    // State
    let mut in_paragraph = false;
    let mut in_run = false;
    let mut in_text = false;
    let mut in_ppr = false;
    let mut current_style: Option<String> = None;
    let mut current_text = String::new();

    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                let name = e.name();
                let local = local_name(name.as_ref());
                match local {
                    b"p" => {
                        in_paragraph = true;
                        current_style = None;
                        current_text.clear();
                    }
                    b"pPr" => in_ppr = true,
                    b"pStyle" if in_ppr => {
                        if let Some(val) = e.attributes().flatten().find(|a| a.key.as_ref() == b"w:val" || local_name(a.key.as_ref()) == b"val") {
                            current_style = Some(String::from_utf8_lossy(&val.value).into_owned());
                        }
                    }
                    b"r" => in_run = true,
                    b"t" if in_run => in_text = true,
                    _ => {}
                }
            }
            Ok(Event::End(ref e)) => {
                let name = e.name();
                let local = local_name(name.as_ref());
                match local {
                    b"p" => {
                        if in_paragraph {
                            paragraphs.push(Paragraph {
                                style: current_style.take(),
                                text: current_text.trim().to_string(),
                            });
                            in_paragraph = false;
                        }
                    }
                    b"pPr" => in_ppr = false,
                    b"r" => in_run = false,
                    b"t" => in_text = false,
                    _ => {}
                }
            }
            Ok(Event::Text(ref e)) if in_text => {
                let raw = e.decode().unwrap_or_default();
                let s = unescape(&raw).unwrap_or_else(|_| raw.clone());
                current_text.push_str(&s);
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(AppError::XmlError(e)),
            _ => {}
        }
        buf.clear();
    }

    Ok(ParsedXml { paragraphs })
}

/// Strip namespace prefix, returning just the local part as bytes.
fn local_name(name: &[u8]) -> &[u8] {
    name.splitn(2, |&b| b == b':').last().unwrap_or(name)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn wrap(inner: &str) -> String {
        format!(
            r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
<w:body>{inner}</w:body></w:document>"#
        )
    }

    fn para(style: &str, text: &str) -> String {
        format!(
            r#"<w:p><w:pPr><w:pStyle w:val="{style}"/></w:pPr><w:r><w:t>{text}</w:t></w:r></w:p>"#
        )
    }

    #[test]
    fn test_parse_single_paragraph() {
        let xml = wrap(&para("Normal", "Hello world"));
        let parsed = parse_document_xml(&xml).unwrap();
        assert_eq!(parsed.paragraphs.len(), 1);
        assert_eq!(parsed.paragraphs[0].text, "Hello world");
        assert_eq!(parsed.paragraphs[0].style.as_deref(), Some("Normal"));
    }

    #[test]
    fn test_parse_heading_style() {
        let xml = wrap(&para("Heading1", "Introduction"));
        let parsed = parse_document_xml(&xml).unwrap();
        assert_eq!(parsed.paragraphs[0].style.as_deref(), Some("Heading1"));
    }

    #[test]
    fn test_parse_no_style() {
        let xml = wrap(r#"<w:p><w:r><w:t>plain</w:t></w:r></w:p>"#);
        let parsed = parse_document_xml(&xml).unwrap();
        assert_eq!(parsed.paragraphs[0].style, None);
        assert_eq!(parsed.paragraphs[0].text, "plain");
    }

    #[test]
    fn test_parse_multiple_paragraphs() {
        let xml = wrap(&format!(
            "{}{}{}",
            para("Heading1", "Title"),
            para("Normal", "Body"),
            para("Normal", "More body"),
        ));
        let parsed = parse_document_xml(&xml).unwrap();
        assert_eq!(parsed.paragraphs.len(), 3);
    }

    #[test]
    fn test_empty_paragraphs_included() {
        let xml = wrap(r#"<w:p><w:r><w:t></w:t></w:r></w:p>"#);
        let parsed = parse_document_xml(&xml).unwrap();
        assert_eq!(parsed.paragraphs.len(), 1);
        assert_eq!(parsed.paragraphs[0].text, "");
    }
}
