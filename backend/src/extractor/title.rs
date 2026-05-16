use crate::parser::raw_xml::ParsedXml;

const TITLE_STYLES: &[&str] = &["Title", "Heading1", "1"];
const SUBTITLE_STYLES: &[&str] = &["Subtitle", "subtitle"];

/// Extract the document title.
///
/// Strategy (in order):
/// 1. First paragraph whose style is in TITLE_STYLES.
/// 2. First non-empty paragraph as a fallback (adds a warning).
pub fn extract_title(parsed: &ParsedXml, warnings: &mut Vec<String>) -> Option<String> {
    // Preferred: recognised title style
    for para in &parsed.paragraphs {
        if let Some(style) = &para.style {
            if TITLE_STYLES.iter().any(|s| style.eq_ignore_ascii_case(s))
                && !para.text.is_empty()
            {
                return Some(para.text.clone());
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

/// Extract the document subtitle — a paragraph styled "Subtitle" immediately
/// after (or near) the title. Returns `None` if no subtitle style is found.
pub fn extract_subtitle(parsed: &ParsedXml) -> Option<String> {
    parsed.paragraphs.iter().find(|p| {
        p.style
            .as_deref()
            .map(|s| SUBTITLE_STYLES.iter().any(|st| s.eq_ignore_ascii_case(st)))
            .unwrap_or(false)
            && !p.text.is_empty()
    }).map(|p| p.text.clone())
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

    #[test]
    fn test_subtitle_extracted_when_styled() {
        let p = parsed(vec![
            make(Some("Title"), "Main Title"),
            make(Some("Subtitle"), "A Longer Subtitle Phrase"),
        ]);
        assert_eq!(
            extract_subtitle(&p),
            Some("A Longer Subtitle Phrase".into())
        );
    }

    #[test]
    fn test_subtitle_none_when_absent() {
        let p = parsed(vec![
            make(Some("Title"), "Main Title"),
            make(Some("Normal"), "Body paragraph"),
        ]);
        assert_eq!(extract_subtitle(&p), None);
    }

    #[test]
    fn test_subtitle_case_insensitive_style() {
        let p = parsed(vec![make(Some("SUBTITLE"), "Sub")]);
        assert_eq!(extract_subtitle(&p), Some("Sub".into()));
    }
}
