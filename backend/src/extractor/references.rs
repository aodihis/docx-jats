use crate::{models::Reference, parser::raw_xml::ParsedXml};

const REFERENCE_HEADINGS: &[&str] = &["References", "Bibliography", "Works Cited"];

/// Extract references from the document.
///
/// Strategy: find a heading whose text matches a known reference section name,
/// then collect all following non-empty paragraphs as raw reference strings.
pub fn extract_references(parsed: &ParsedXml, warnings: &mut Vec<String>) -> Vec<Reference> {
    let paras = &parsed.paragraphs;

    let ref_start = paras.iter().position(|p| {
        REFERENCE_HEADINGS
            .iter()
            .any(|h| p.text.trim().eq_ignore_ascii_case(h))
    });

    let Some(start) = ref_start else {
        warnings.push("No references section found".into());
        return Vec::new();
    };

    let refs: Vec<Reference> = paras[start + 1..]
        .iter()
        .filter(|p| !p.text.is_empty())
        .enumerate()
        .map(|(i, p)| Reference {
            id: format!("ref-{}", i + 1),
            raw_text: p.text.clone(),
        })
        .collect();

    if refs.is_empty() {
        warnings.push("References heading found but no reference entries detected".into());
    }

    refs
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::raw_xml::{Paragraph, ParsedXml};

    fn make(style: Option<&str>, text: &str) -> Paragraph {
        Paragraph {
            style: style.map(String::from),
            text: text.to_string(),
        }
    }

    fn parsed(paras: Vec<Paragraph>) -> ParsedXml {
        ParsedXml { paragraphs: paras }
    }

    #[test]
    fn test_references_detected() {
        let p = parsed(vec![
            make(Some("Heading1"), "References"),
            make(Some("Normal"), "Smith et al. (2020). Example."),
            make(Some("Normal"), "Doe (2021). Another."),
        ]);
        let mut w = vec![];
        let refs = extract_references(&p, &mut w);
        assert_eq!(refs.len(), 2);
        assert_eq!(refs[0].id, "ref-1");
        assert!(refs[0].raw_text.contains("Smith"));
    }

    #[test]
    fn test_bibliography_alias() {
        let p = parsed(vec![
            make(Some("Heading1"), "Bibliography"),
            make(Some("Normal"), "Author (2022)."),
        ]);
        let mut w = vec![];
        let refs = extract_references(&p, &mut w);
        assert_eq!(refs.len(), 1);
    }

    #[test]
    fn test_no_references_section() {
        let p = parsed(vec![make(Some("Normal"), "Just a paragraph.")]);
        let mut w = vec![];
        let refs = extract_references(&p, &mut w);
        assert!(refs.is_empty());
        assert!(!w.is_empty(), "expected warning about missing references");
    }

    #[test]
    fn test_empty_references_section() {
        let p = parsed(vec![make(Some("Heading1"), "References")]);
        let mut w = vec![];
        let refs = extract_references(&p, &mut w);
        assert!(refs.is_empty());
        assert!(w.iter().any(|msg| msg.contains("no reference entries")));
    }
}
