use crate::parser::raw_xml::ParsedXml;

const TITLE_STYLES: &[&str] = &["Title", "Heading1", "1"];

/// Extract the document title.
///
/// Strategy (in order):
/// 1. First paragraph whose style is in TITLE_STYLES.
/// 2. First non-empty paragraph as a fallback (adds a warning).
pub fn extract_title(parsed: &ParsedXml, warnings: &mut Vec<String>) -> Option<String> {
    // Preferred: recognised title style
    for para in &parsed.paragraphs {
        if let Some(style) = &para.style {
            if TITLE_STYLES.iter().any(|s| style.eq_ignore_ascii_case(s)) {
                if !para.text.is_empty() {
                    return Some(para.text.clone());
                }
            }
        }
    }

    // Fallback: first non-empty paragraph
    for para in &parsed.paragraphs {
        if !para.text.is_empty() {
            warnings.push(format!(
                "Title heuristic: used first paragraph \"{}\" as title",
                truncate(&para.text, 40)
            ));
            return Some(para.text.clone());
        }
    }

    None
}

fn truncate(s: &str, max: usize) -> &str {
    if s.len() <= max {
        s
    } else {
        &s[..max]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::raw_xml::{ParsedXml, Paragraph};

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
    fn test_title_style_preferred() {
        let p = parsed(vec![
            make(Some("Normal"), "ignored"),
            make(Some("Heading1"), "My Paper"),
        ]);
        let mut w = vec![];
        assert_eq!(extract_title(&p, &mut w), Some("My Paper".into()));
        assert!(w.is_empty());
    }

    #[test]
    fn test_title_style_case_insensitive() {
        let p = parsed(vec![make(Some("title"), "Case Test")]);
        let mut w = vec![];
        assert_eq!(extract_title(&p, &mut w), Some("Case Test".into()));
    }

    #[test]
    fn test_fallback_first_paragraph() {
        let p = parsed(vec![make(Some("Normal"), "Fallback Title")]);
        let mut w = vec![];
        let result = extract_title(&p, &mut w);
        assert_eq!(result, Some("Fallback Title".into()));
        assert!(!w.is_empty(), "expected a warning for fallback");
    }

    #[test]
    fn test_empty_doc_returns_none() {
        let p = parsed(vec![]);
        let mut w = vec![];
        assert_eq!(extract_title(&p, &mut w), None);
    }
}
