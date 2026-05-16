use quick_xml::{escape::unescape, events::Event, Reader};

use crate::{models::Author, parser::raw_xml::ParsedXml};

const AUTHOR_STYLES: &[&str] = &["Author", "author", "Authors", "authors"];

/// Substrings that identify a tool-generated dc:creator value (not a real person).
const TOOL_SIGNATURES: &[&str] = &[
    "python",
    "microsoft",
    "ms word",
    "openoffice",
    "libreoffice",
    "aspose",
    "apache poi",
    "docx",
    "office user",
    "word user",
];

/// Maximum number of paragraphs after the title to inspect for positional authors.
const POSITIONAL_WINDOW: usize = 5;

/// Maximum character length for a paragraph to be considered a name candidate.
const MAX_NAME_LEN: usize = 120;

/// Extract authors.
///
/// Strategy (in order):
/// 1. `dc:creator` from docProps/core.xml — skipped if it looks tool-generated.
/// 2. Paragraphs styled as "Author" in the document body.
/// 3. Positional heuristic: short paragraphs immediately after the title and
///    before the first heading / abstract, treated as candidate author lines.
pub fn extract_authors(
    core_xml: Option<&str>,
    parsed: &ParsedXml,
    known_title: Option<&str>,
    warnings: &mut Vec<String>,
) -> Vec<Author> {
    // 1. Metadata
    if let Some(xml) = core_xml {
        let authors = from_core_xml(xml);
        if !authors.is_empty() {
            return authors;
        }
    }

    // 2. Explicit "Author" style
    let styled = from_body_style(parsed);
    if !styled.is_empty() {
        return styled;
    }

    // 3. Positional heuristic
    let positional = from_position(parsed, known_title);
    if !positional.is_empty() {
        warnings.push(
            "Authors were inferred from position (paragraphs after title). \
             Verify in the edit panel."
                .into(),
        );
        return positional;
    }

    warnings.push("No authors found — add them manually in the edit panel.".into());
    Vec::new()
}

// ── Source 1: docProps/core.xml ───────────────────────────────────────────────

fn from_core_xml(xml: &str) -> Vec<Author> {
    let raw = read_dc_creator(xml);
    let raw = raw.trim();
    if raw.is_empty() || is_tool_generated(raw) {
        return Vec::new();
    }
    split_creator(raw)
}

fn read_dc_creator(xml: &str) -> String {
    let mut reader = Reader::from_str(xml);
    // Do NOT trim_text — it trims each event independently and collapses the
    // space that separates entity-split text chunks (e.g. "B., " + " Zibbitova").
    // from_core_xml calls .trim() on the final result instead.
    let mut in_creator = false;
    let mut buf = Vec::new();
    let mut text = String::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let name = e.name();
                if local_name(name.as_ref()) == b"creator" {
                    in_creator = true;
                    text.clear();
                }
            }
            Ok(Event::End(ref e)) => {
                let name = e.name();
                if local_name(name.as_ref()) == b"creator" {
                    return text;
                }
            }
            Ok(Event::Text(ref e)) if in_creator => {
                let raw = e.decode().unwrap_or_default();
                let s = unescape(&raw).unwrap_or_else(|_| raw.clone());
                text.push_str(&s);
            }
            Ok(Event::Eof) | Err(_) => break,
            _ => {}
        }
        buf.clear();
    }

    String::new()
}

fn is_tool_generated(creator: &str) -> bool {
    let lower = creator.to_lowercase();
    TOOL_SIGNATURES.iter().any(|sig| {
        // Require the signature to appear as a whole token, not as a substring
        // of a longer word. Split on common punctuation/whitespace.
        lower.split(|c: char| c.is_whitespace() || matches!(c, '-' | '_' | '/' | '.' | ','))
            .any(|token| token == *sig)
    })
}

fn split_creator(raw: &str) -> Vec<Author> {
    // Semicolon-separated is unambiguous — use it directly.
    if raw.contains(';') {
        return raw
            .split(';')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .map(|s| Author { name: s.to_string() })
            .collect();
    }

    // Detect APA-style "Lastname, I., Lastname, I., & Lastname, I."
    // by checking whether any ", "-split token looks like an initial ("A.", "A. B.").
    let tokens: Vec<&str> = raw.split(", ").collect();
    let has_initials = tokens.iter().any(|t| {
        is_initial_token(t.trim_start_matches("& ").trim())
    });

    if has_initials {
        return split_apa_style(&tokens);
    }

    // Plain comma-separated: "Alice Smith, Bob Jones"
    raw.split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| Author { name: s.to_string() })
        .collect()
}

/// Returns true when `s` looks like author initials: "A.", "A. B.", "AB.", etc.
fn is_initial_token(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }
    s.split_whitespace().all(|word| {
        let w = word.trim_end_matches('.');
        !w.is_empty() && w.len() <= 3 && w.chars().all(|c| c.is_uppercase())
    })
}

/// Parse APA-format author list where authors are "Lastname, I." entries
/// separated by ", ", with optional "& " before the last one.
///
/// "Snazzleton, A., Frumpleston, B., & Zibbitova, C."
///   → ["Snazzleton, A.", "Frumpleston, B.", "Zibbitova, C."]
fn split_apa_style(tokens: &[&str]) -> Vec<Author> {
    let mut authors: Vec<Author> = Vec::new();
    let mut pending: Option<String> = None;

    for token in tokens {
        let token = token.trim_start_matches("& ").trim();
        if is_initial_token(token) {
            // This is an initial — attach it to the pending last name.
            if let Some(ref mut name) = pending {
                name.push_str(", ");
                name.push_str(token);
            }
            // Flush: the "Lastname, Initial." pair is now complete.
            if let Some(name) = pending.take() {
                authors.push(Author { name });
            }
        } else {
            // This is a last name (or full name). Flush any preceding incomplete entry.
            if let Some(name) = pending.take() {
                authors.push(Author { name });
            }
            pending = Some(token.to_string());
        }
    }
    // Flush whatever is left (e.g. a name with no initials after it).
    if let Some(name) = pending {
        authors.push(Author { name });
    }
    authors
}

// ── Source 2: "Author" style paragraphs ──────────────────────────────────────

fn from_body_style(parsed: &ParsedXml) -> Vec<Author> {
    parsed
        .paragraphs
        .iter()
        .filter(|p| {
            p.style
                .as_deref()
                .map(|s| AUTHOR_STYLES.iter().any(|a| s.eq_ignore_ascii_case(a)))
                .unwrap_or(false)
                && !p.text.is_empty()
        })
        .map(|p| Author { name: p.text.clone() })
        .collect()
}

// ── Source 3: Positional heuristic ───────────────────────────────────────────

/// Look at up to POSITIONAL_WINDOW paragraphs that immediately follow the title
/// paragraph and precede the first heading or abstract-style paragraph.
/// Short, non-sentence paragraphs in that window are treated as author names.
fn from_position(parsed: &ParsedXml, known_title: Option<&str>) -> Vec<Author> {
    let paras = &parsed.paragraphs;

    // Find where the title paragraph is
    let title_idx = known_title.and_then(|title| {
        paras
            .iter()
            .position(|p| p.text.trim() == title.trim())
    });

    let start = match title_idx {
        Some(i) => i + 1,
        // No known title — try from the very beginning but limit window tightly
        None => 0,
    };

    let mut authors = Vec::new();

    for para in paras.iter().skip(start).take(POSITIONAL_WINDOW) {
        let text = para.text.trim();

        // Stop at the first heading or abstract-like paragraph
        if is_heading_style(&para.style) || is_abstract_heading(text) {
            break;
        }

        // Skip empty paragraphs (common spacers in DOCX front matter)
        if text.is_empty() {
            continue;
        }

        // Stop if the paragraph looks like body text (long or sentence-shaped)
        if text.len() > MAX_NAME_LEN || looks_like_body_text(text) {
            break;
        }

        // Apply the same APA-aware split as dc:creator so that a line like
        // "Smith, A., Jones, B., & Brown, C." yields 3 authors, not 1.
        authors.extend(split_creator(text));
    }

    authors
}

fn is_heading_style(style: &Option<String>) -> bool {
    style.as_deref().map(|s| {
        let sl = s.to_lowercase();
        sl.starts_with("heading") || sl == "1" || sl == "2" || sl == "3"
    }).unwrap_or(false)
}

fn is_abstract_heading(text: &str) -> bool {
    text.eq_ignore_ascii_case("abstract")
        || text.eq_ignore_ascii_case("summary")
        || text.eq_ignore_ascii_case("introduction")
}

/// Returns true when the text looks like a sentence (body text) rather than a name.
fn looks_like_body_text(text: &str) -> bool {
    let word_count = text.split_whitespace().count();

    // More than 7 words — almost certainly a sentence, not a name
    if word_count > 7 {
        return true;
    }
    // Contains ". " in the middle — multiple sentences, UNLESS every ". "
    // is preceded by a single uppercase letter (author initial, e.g. "A., ").
    if text.contains(". ") {
        let has_sentence_break = text.match_indices(". ").any(|(i, _)| {
            // Walk backwards to find the character(s) before the period.
            let before = text[..i].trim_end_matches(',').trim_end();
            let last_word = before.split_whitespace().last().unwrap_or("");
            // If the word before "." is a 1-3 char uppercase token it's an initial.
            let is_initial = !last_word.is_empty()
                && last_word.len() <= 3
                && last_word.chars().all(|c| c.is_uppercase());
            !is_initial
        });
        if has_sentence_break {
            return true;
        }
    }
    // Ends with "." and has more than 5 words — likely a sentence,
    // UNLESS the last word is an initial (e.g. "Smith, A. B., Jones, C.").
    if text.ends_with('.') && word_count > 5 {
        let before_dot = text.trim_end_matches('.');
        let last_word = before_dot.split_whitespace().last().unwrap_or("");
        let w = last_word.trim_end_matches('.');
        let ends_with_initial =
            !w.is_empty() && w.len() <= 3 && w.chars().all(|c| c.is_uppercase());
        if !ends_with_initial {
            return true;
        }
    }
    // Starts with a lowercase letter — continuation text
    if text.chars().next().map(|c| c.is_lowercase()).unwrap_or(false) {
        return true;
    }
    // Starts with a very common sentence-opening word (not a name prefix)
    let first_word = text.split_whitespace().next().unwrap_or("");
    matches!(
        first_word.to_lowercase().as_str(),
        "the" | "a" | "an" | "this" | "in" | "of" | "for" | "we" | "our"
    )
}

fn local_name(name: &[u8]) -> &[u8] {
    name.splitn(2, |&b| b == b':').last().unwrap_or(name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::raw_xml::{Paragraph, ParsedXml};

    fn parsed(paras: Vec<Paragraph>) -> ParsedXml {
        ParsedXml { paragraphs: paras }
    }

    fn para(style: Option<&str>, text: &str) -> Paragraph {
        Paragraph {
            style: style.map(String::from),
            text: text.to_string(),
        }
    }

    fn core_xml(creator: &str) -> String {
        // Escape XML special characters so the test XML is always well-formed.
        let escaped = creator
            .replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;");
        format!(
            r#"<?xml version="1.0"?>
<cp:coreProperties xmlns:cp="http://schemas.openxmlformats.org/package/2006/metadata/core-properties"
                   xmlns:dc="http://purl.org/dc/elements/1.1/">
  <dc:creator>{escaped}</dc:creator>
</cp:coreProperties>"#
        )
    }

    #[test]
    fn test_authors_from_core_xml_semicolon() {
        let mut w = vec![];
        let authors = extract_authors(Some(&core_xml("Alice Smith; Bob Jones")), &parsed(vec![]), None, &mut w);
        assert_eq!(authors.len(), 2);
        assert_eq!(authors[0].name, "Alice Smith");
        assert_eq!(authors[1].name, "Bob Jones");
        assert!(w.is_empty());
    }

    #[test]
    fn test_authors_from_core_xml_single() {
        let mut w = vec![];
        let authors = extract_authors(Some(&core_xml("Jane Doe")), &parsed(vec![]), None, &mut w);
        assert_eq!(authors.len(), 1);
        assert_eq!(authors[0].name, "Jane Doe");
    }

    #[test]
    fn test_tool_generated_creator_is_skipped() {
        let mut w = vec![];
        // "python-docx" should be filtered out
        let authors = extract_authors(Some(&core_xml("python-docx")), &parsed(vec![]), None, &mut w);
        assert!(authors.is_empty() || w.iter().any(|m: &String| m.contains("author")));
    }

    #[test]
    fn test_microsoft_creator_is_skipped() {
        let p = parsed(vec![para(Some("Author"), "Real Author")]);
        let mut w = vec![];
        let authors = extract_authors(Some(&core_xml("Microsoft")), &p, None, &mut w);
        // Should fall through to body style, finding "Real Author"
        assert_eq!(authors.len(), 1);
        assert_eq!(authors[0].name, "Real Author");
    }

    #[test]
    fn test_authors_fallback_to_body_style() {
        let p = parsed(vec![
            para(Some("Author"), "Dr. Smith"),
            para(Some("Normal"), "Introduction text"),
        ]);
        let mut w = vec![];
        let authors = extract_authors(None, &p, None, &mut w);
        assert_eq!(authors.len(), 1);
        assert_eq!(authors[0].name, "Dr. Smith");
        assert!(w.is_empty());
    }

    #[test]
    fn test_positional_author_after_title() {
        let p = parsed(vec![
            para(Some("Title"), "My Paper Title"),
            para(Some("Normal"), "Jane Smith"),
            para(Some("Normal"), "John Doe"),
            para(Some("Heading1"), "Abstract"),
            para(Some("Normal"), "This paper investigates..."),
        ]);
        let mut w = vec![];
        let authors = extract_authors(None, &p, Some("My Paper Title"), &mut w);
        assert_eq!(authors.len(), 2);
        assert_eq!(authors[0].name, "Jane Smith");
        assert_eq!(authors[1].name, "John Doe");
        assert!(w.iter().any(|m: &String| m.contains("inferred")));
    }

    #[test]
    fn test_positional_stops_at_body_text() {
        let p = parsed(vec![
            para(Some("Title"), "My Paper"),
            para(Some("Normal"), "Jane Smith"),
            // Long sentence — should stop here
            para(Some("Normal"), "This study examines the effects of climate change on polar ecosystems."),
            para(Some("Normal"), "Bob Jones"),
        ]);
        let mut w = vec![];
        let authors = extract_authors(None, &p, Some("My Paper"), &mut w);
        assert_eq!(authors.len(), 1);
        assert_eq!(authors[0].name, "Jane Smith");
    }

    #[test]
    fn test_no_authors_adds_warning() {
        let p = parsed(vec![para(Some("Normal"), "Just body text that is very long and looks like a sentence indeed.")]);
        let mut w = vec![];
        let authors = extract_authors(None, &p, None, &mut w);
        assert!(authors.is_empty());
        assert!(!w.is_empty());
    }

    #[test]
    fn test_core_xml_takes_priority_over_body_style() {
        let p = parsed(vec![para(Some("Author"), "Style Author")]);
        let mut w = vec![];
        let authors = extract_authors(Some(&core_xml("Metadata Author")), &p, None, &mut w);
        assert_eq!(authors.len(), 1);
        assert_eq!(authors[0].name, "Metadata Author");
    }

    #[test]
    fn test_split_creator_apa_direct() {
        // Test split_creator directly (bypasses XML parsing).
        let authors = split_creator("Snazzleton, A., Frumpleston, B., & Zibbitova, C.");
        assert_eq!(authors.len(), 3, "direct split_creator got: {:?}", authors.iter().map(|a| &a.name).collect::<Vec<_>>());
        assert_eq!(authors[0].name, "Snazzleton, A.");
        assert_eq!(authors[1].name, "Frumpleston, B.");
        assert_eq!(authors[2].name, "Zibbitova, C.");
    }

    #[test]
    fn test_apa_style_creator_three_authors() {
        // "Lastname, I., Lastname, I., & Lastname, I." — end-to-end through XML.
        // Uses &amp; in XML so the reader can decode it back to &.
        let xml = core_xml("Snazzleton, A., Frumpleston, B., & Zibbitova, C.");
        let mut w = vec![];
        let authors = extract_authors(Some(&xml), &parsed(vec![]), None, &mut w);
        assert_eq!(authors.len(), 3, "expected 3 authors, got: {:?}", authors.iter().map(|a| &a.name).collect::<Vec<_>>());
        assert_eq!(authors[0].name, "Snazzleton, A.");
        assert_eq!(authors[1].name, "Frumpleston, B.");
        assert_eq!(authors[2].name, "Zibbitova, C.");
    }

    #[test]
    fn test_apa_style_two_initials_per_author() {
        // "Smith, A. B., Jones, C. D." — each author has two initials.
        let xml = core_xml("Smith, A. B., Jones, C. D.");
        let mut w = vec![];
        let authors = extract_authors(Some(&xml), &parsed(vec![]), None, &mut w);
        assert_eq!(authors.len(), 2, "got: {:?}", authors);
        assert_eq!(authors[0].name, "Smith, A. B.");
        assert_eq!(authors[1].name, "Jones, C. D.");
    }

    #[test]
    fn test_positional_author_with_initials_not_rejected() {
        // A single paragraph "Snazzleton, A., Frumpleston, B., & Zibbitova, C."
        // should not be thrown away by looks_like_body_text.
        let p = parsed(vec![
            para(Some("Title"), "Review of Something"),
            para(Some("Normal"), "Snazzleton, A., Frumpleston, B., & Zibbitova, C."),
            para(Some("Heading1"), "Abstract"),
        ]);
        let mut w = vec![];
        let authors = extract_authors(None, &p, Some("Review of Something"), &mut w);
        // The APA line is split into 3 individual authors.
        assert_eq!(authors.len(), 3, "expected 3 authors, got: {:?}", authors.iter().map(|a| &a.name).collect::<Vec<_>>());
        assert_eq!(authors[0].name, "Snazzleton, A.");
        assert_eq!(authors[1].name, "Frumpleston, B.");
        assert_eq!(authors[2].name, "Zibbitova, C.");
    }

    #[test]
    fn test_real_name_containing_tool_substring_not_filtered() {
        // "Python Analytics" contains "python" as first token — would be filtered.
        // But "Pythonia Smith" does NOT contain "python" as a whole token.
        let mut w = vec![];
        let authors = extract_authors(Some(&core_xml("Pythonia Smith")), &parsed(vec![]), None, &mut w);
        assert_eq!(authors.len(), 1, "should not filter 'Pythonia Smith'");
        assert_eq!(authors[0].name, "Pythonia Smith");
    }

    #[test]
    fn test_python_docx_hyphenated_is_filtered() {
        // "python-docx" splits into tokens ["python", "docx"] — both are signatures.
        let mut w = vec![];
        let authors = extract_authors(Some(&core_xml("python-docx")), &parsed(vec![]), None, &mut w);
        assert!(authors.is_empty(), "python-docx must be filtered");
    }
}
