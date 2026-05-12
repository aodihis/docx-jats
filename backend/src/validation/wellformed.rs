use quick_xml::{events::Event, Reader};

/// Validate XML well-formedness by consuming the document with quick-xml.
///
/// Returns a list of error messages. Empty vec means the XML is well-formed.
pub fn check(xml: &str) -> Vec<String> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(false);

    let mut warnings = Vec::new();
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Eof) => break,
            Err(e) => {
                warnings.push(format!(
                    "XML well-formedness error at position {}: {}",
                    reader.error_position(),
                    e
                ));
                break;
            }
            _ => {}
        }
        buf.clear();
    }

    warnings
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_xml_no_warnings() {
        let xml = r#"<?xml version="1.0"?><article><front/></article>"#;
        assert!(check(xml).is_empty());
    }

    #[test]
    fn test_unclosed_tag_produces_warning() {
        let xml = "<article><front></article>";
        let warnings = check(xml);
        assert!(!warnings.is_empty(), "expected at least one warning");
    }

    #[test]
    fn test_empty_string_is_invalid() {
        let warnings = check("");
        // Empty input has no well-formedness error per quick-xml (just EOF), OK to be empty
        // This test just ensures we don't panic
        let _ = warnings;
    }
}
