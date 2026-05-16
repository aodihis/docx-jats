use crate::parser::raw_xml::ParsedXml;

const ABSTRACT_STYLES: &[&str] = &["Abstract", "abstract"];

/// Extract the abstract text.
///
/// Strategy (in order):
/// 1. First paragraph whose style is in ABSTRACT_STYLES.
/// 2. The paragraph immediately following one whose text starts with "Abstract".
pub fn extract_abstract(parsed: &ParsedXml, warnings: &mut Vec<String>) -> Option<String> {
    let paras = &parsed.paragraphs;

    // Strategy 1: style match
    for para in paras {
        if let Some(style) = &para.style {
            if ABSTRACT_STYLES.iter().any(|s| style.eq_ignore_ascii_case(s))
                && !para.text.is_empty()
            {
                return Some(para.text.clone());
            }
        }
    }

    // Strategy 2: paragraph whose text is exactly "Abstract" → collect the next non-empty para
    for (i, para) in paras.iter().enumerate() {
        if para.text.trim().eq_ignore_ascii_case("abstract") {
            // Collect subsequent non-heading paragraphs as abstract body
            let body: String = paras[i + 1..]
                .iter()
                .take_while(|p| !is_heading(p))
                .filter(|p| !p.text.is_empty())
                .map(|p| p.text.as_str())
                .collect::<Vec<_>>()
                .join(" ");
            if !body.is_empty() {
                warnings.push("Abstract detected by keyword heading heuristic".into());
                return Some(body);
            }
        }
    }

    None
}

fn is_heading(para: &crate::parser::raw_xml::Paragraph) -> bool {
    para.style
        .as_deref()
        .map(|s| {
            s.to_ascii_lowercase().starts_with("heading")
                || s == "1"
                || s == "2"
                || s == "3"
        })
        .unwrap_or(false)
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
    fn test_abstract_style_detected() {
        let p = parsed(vec![make(Some("Abstract"), "This study examines...")]);
        let mut w = vec![];
        assert_eq!(
            extract_abstract(&p, &mut w),
            Some("This study examines...".into())
        );
    }

    #[test]
    fn test_abstract_keyword_heading() {
        let p = parsed(vec![
            make(Some("Heading1"), "Abstract"),
            make(Some("Normal"), "The abstract body."),
        ]);
        let mut w = vec![];
        let result = extract_abstract(&p, &mut w);
        assert_eq!(result, Some("The abstract body.".into()));
        assert!(!w.is_empty());
    }

    #[test]
    fn test_abstract_case_insensitive_style() {
        let p = parsed(vec![make(Some("abstract"), "Lower case style.")]);
        let mut w = vec![];
        assert!(extract_abstract(&p, &mut w).is_some());
    }

    #[test]
    fn test_no_abstract_returns_none() {
        let p = parsed(vec![
            make(Some("Heading1"), "Introduction"),
            make(Some("Normal"), "Body text."),
        ]);
        let mut w = vec![];
        assert_eq!(extract_abstract(&p, &mut w), None);
    }
}
