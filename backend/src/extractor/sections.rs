use crate::{models::Section, parser::raw_xml::ParsedXml};

const HEADING_STYLES: &[(&str, u8)] = &[
    ("Heading1", 1),
    ("heading1", 1),
    ("1", 1),
    ("Heading2", 2),
    ("heading2", 2),
    ("2", 2),
    ("Heading3", 3),
    ("heading3", 3),
    ("3", 3),
];

const SKIP_HEADINGS: &[&str] = &["Abstract", "abstract", "Title", "title"];

/// Extract sections from the document body.
///
/// Each heading-style paragraph starts a new section. Paragraphs under a
/// heading are collected as `body`. If there are no headings, a single
/// unnamed section holds all body text.
pub fn extract_sections(parsed: &ParsedXml, warnings: &mut Vec<String>) -> Vec<Section> {
    let mut sections: Vec<Section> = Vec::new();
    let mut current: Option<Section> = None;
    let mut skip_zone = false; // true while inside a skipped heading's body

    for para in &parsed.paragraphs {
        if let Some(level) = heading_level(&para.style) {
            // Skip meta-headings like Abstract (and their body paragraphs)
            if SKIP_HEADINGS
                .iter()
                .any(|s| para.text.trim().eq_ignore_ascii_case(s))
            {
                skip_zone = true;
                continue;
            }
            skip_zone = false;

            // Flush current section
            if let Some(sec) = current.take() {
                sections.push(sec);
            }

            current = Some(Section {
                heading: if para.text.is_empty() {
                    None
                } else {
                    Some(para.text.clone())
                },
                level,
                body: Vec::new(),
            });
        } else {
            if skip_zone {
                continue;
            }
            match current.as_mut() {
                Some(sec) if !para.text.is_empty() => sec.body.push(para.text.clone()),
                None if !para.text.is_empty() => {
                    // Body text before any heading — create implicit section
                    current = Some(Section {
                        heading: None,
                        level: 1,
                        body: vec![para.text.clone()],
                    });
                }
                _ => {}
            }
        }
    }

    if let Some(sec) = current {
        sections.push(sec);
    }

    if sections.is_empty() {
        warnings.push("No sections detected in document".into());
    }

    sections
}

fn heading_level(style: &Option<String>) -> Option<u8> {
    style.as_deref().and_then(|s| {
        HEADING_STYLES
            .iter()
            .find(|(name, _)| name.eq_ignore_ascii_case(s))
            .map(|(_, lvl)| *lvl)
    })
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
    fn test_single_section() {
        let p = parsed(vec![
            make(Some("Heading1"), "Introduction"),
            make(Some("Normal"), "Body text."),
        ]);
        let mut w = vec![];
        let sections = extract_sections(&p, &mut w);
        assert_eq!(sections.len(), 1);
        assert_eq!(sections[0].heading.as_deref(), Some("Introduction"));
        assert_eq!(sections[0].body, vec!["Body text."]);
    }

    #[test]
    fn test_nested_headings() {
        let p = parsed(vec![
            make(Some("Heading1"), "Methods"),
            make(Some("Normal"), "Overview."),
            make(Some("Heading2"), "Participants"),
            make(Some("Normal"), "N=100."),
        ]);
        let mut w = vec![];
        let sections = extract_sections(&p, &mut w);
        assert_eq!(sections.len(), 2);
        assert_eq!(sections[0].level, 1);
        assert_eq!(sections[1].level, 2);
    }

    #[test]
    fn test_no_headings_one_implicit_section() {
        let p = parsed(vec![
            make(Some("Normal"), "Para one."),
            make(Some("Normal"), "Para two."),
        ]);
        let mut w = vec![];
        let sections = extract_sections(&p, &mut w);
        assert_eq!(sections.len(), 1);
        assert_eq!(sections[0].heading, None);
        assert_eq!(sections[0].body.len(), 2);
    }

    #[test]
    fn test_abstract_heading_skipped() {
        let p = parsed(vec![
            make(Some("Heading1"), "Abstract"),
            make(Some("Normal"), "The abstract."),
            make(Some("Heading1"), "Introduction"),
            make(Some("Normal"), "Intro body."),
        ]);
        let mut w = vec![];
        let sections = extract_sections(&p, &mut w);
        assert_eq!(sections.len(), 1);
        assert_eq!(sections[0].heading.as_deref(), Some("Introduction"));
    }
}
