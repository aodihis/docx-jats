use quick_xml::{
    events::{BytesDecl, BytesEnd, BytesStart, BytesText, Event},
    Writer,
};
use std::io::Cursor;

use crate::{error::AppError, models::Document};

/// Generate a simplified JATS XML string from a `Document`.
pub fn generate_jats(doc: &Document) -> Result<String, AppError> {
    let mut writer = Writer::new_with_indent(Cursor::new(Vec::new()), b' ', 2);

    // <?xml version="1.0" encoding="UTF-8"?>
    writer
        .write_event(Event::Decl(BytesDecl::new("1.0", Some("UTF-8"), None)))?;

    // <article article-type="research-article" ...>
    let mut article = BytesStart::new("article");
    article.push_attribute(("article-type", "research-article"));
    article.push_attribute(("xmlns:xlink", "http://www.w3.org/1999/xlink"));
    writer.write_event(Event::Start(article))?;

    write_front(&mut writer, doc)?;
    write_body(&mut writer, doc)?;
    write_back(&mut writer, doc)?;

    writer.write_event(Event::End(BytesEnd::new("article")))?;

    let xml_bytes = writer.into_inner().into_inner();
    String::from_utf8(xml_bytes).map_err(|e| AppError::InvalidDocx(e.to_string()))
}

fn write_front(w: &mut Writer<Cursor<Vec<u8>>>, doc: &Document) -> Result<(), AppError> {
    w.write_event(Event::Start(BytesStart::new("front")))?;
    w.write_event(Event::Start(BytesStart::new("article-meta")))?;

    // <title-group>
    w.write_event(Event::Start(BytesStart::new("title-group")))?;
    w.write_event(Event::Start(BytesStart::new("article-title")))?;
    if let Some(title) = &doc.title {
        w.write_event(Event::Text(BytesText::new(title)))?;
    }
    w.write_event(Event::End(BytesEnd::new("article-title")))?;
    if let Some(subtitle) = &doc.subtitle {
        w.write_event(Event::Start(BytesStart::new("subtitle")))?;
        w.write_event(Event::Text(BytesText::new(subtitle)))?;
        w.write_event(Event::End(BytesEnd::new("subtitle")))?;
    }
    w.write_event(Event::End(BytesEnd::new("title-group")))?;

    // <contrib-group>
    if !doc.authors.is_empty() {
        w.write_event(Event::Start(BytesStart::new("contrib-group")))?;
        for author in &doc.authors {
            let mut contrib = BytesStart::new("contrib");
            contrib.push_attribute(("contrib-type", "author"));
            w.write_event(Event::Start(contrib))?;
            w.write_event(Event::Start(BytesStart::new("string-name")))?;
            w.write_event(Event::Text(BytesText::new(&author.name)))?;
            w.write_event(Event::End(BytesEnd::new("string-name")))?;
            w.write_event(Event::End(BytesEnd::new("contrib")))?;
        }
        w.write_event(Event::End(BytesEnd::new("contrib-group")))?;
    }

    // <abstract>
    if let Some(abstract_text) = &doc.abstract_text {
        w.write_event(Event::Start(BytesStart::new("abstract")))?;
        w.write_event(Event::Start(BytesStart::new("p")))?;
        w.write_event(Event::Text(BytesText::new(abstract_text)))?;
        w.write_event(Event::End(BytesEnd::new("p")))?;
        w.write_event(Event::End(BytesEnd::new("abstract")))?;
    }

    w.write_event(Event::End(BytesEnd::new("article-meta")))?;
    w.write_event(Event::End(BytesEnd::new("front")))?;
    Ok(())
}

fn write_body(w: &mut Writer<Cursor<Vec<u8>>>, doc: &Document) -> Result<(), AppError> {
    w.write_event(Event::Start(BytesStart::new("body")))?;

    for section in &doc.sections {
        let mut sec = BytesStart::new("sec");
        sec.push_attribute(("sec-type", level_to_name(section.level)));
        w.write_event(Event::Start(sec))?;

        if let Some(heading) = &section.heading {
            w.write_event(Event::Start(BytesStart::new("title")))?;
            w.write_event(Event::Text(BytesText::new(heading)))?;
            w.write_event(Event::End(BytesEnd::new("title")))?;
        }

        for para in &section.body {
            w.write_event(Event::Start(BytesStart::new("p")))?;
            w.write_event(Event::Text(BytesText::new(para)))?;
            w.write_event(Event::End(BytesEnd::new("p")))?;
        }

        w.write_event(Event::End(BytesEnd::new("sec")))?;
    }

    w.write_event(Event::End(BytesEnd::new("body")))?;
    Ok(())
}

fn write_back(w: &mut Writer<Cursor<Vec<u8>>>, doc: &Document) -> Result<(), AppError> {
    if doc.references.is_empty() {
        return Ok(());
    }

    w.write_event(Event::Start(BytesStart::new("back")))?;
    w.write_event(Event::Start(BytesStart::new("ref-list")))?;

    for reference in &doc.references {
        let mut ref_el = BytesStart::new("ref");
        ref_el.push_attribute(("id", reference.id.as_str()));
        w.write_event(Event::Start(ref_el))?;

        w.write_event(Event::Start(BytesStart::new("mixed-citation")))?;
        w.write_event(Event::Text(BytesText::new(&reference.raw_text)))?;
        w.write_event(Event::End(BytesEnd::new("mixed-citation")))?;

        w.write_event(Event::End(BytesEnd::new("ref")))?;
    }

    w.write_event(Event::End(BytesEnd::new("ref-list")))?;
    w.write_event(Event::End(BytesEnd::new("back")))?;
    Ok(())
}

fn level_to_name(level: u8) -> &'static str {
    match level {
        1 => "section",
        2 => "subsection",
        _ => "subsubsection",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Document, Reference, Section};

    fn minimal_doc() -> Document {
        Document {
            title: Some("Test Title".into()),
            subtitle: None,
            authors: vec![],
            abstract_text: Some("Test abstract.".into()),
            sections: vec![Section {
                heading: Some("Introduction".into()),
                level: 1,
                body: vec!["Body text here.".into()],
            }],
            references: vec![],
            warnings: vec![],
        }
    }

    #[test]
    fn test_generates_article_root() {
        let doc = minimal_doc();
        let xml = generate_jats(&doc).unwrap();
        assert!(xml.contains("<article"));
        assert!(xml.contains("</article>"));
    }

    #[test]
    fn test_includes_abstract() {
        let doc = minimal_doc();
        let xml = generate_jats(&doc).unwrap();
        assert!(xml.contains("<abstract>"));
        assert!(xml.contains("Test abstract."));
    }

    #[test]
    fn test_includes_title() {
        let doc = minimal_doc();
        let xml = generate_jats(&doc).unwrap();
        assert!(xml.contains("<article-title>"));
        assert!(xml.contains("Test Title"));
    }

    #[test]
    fn test_no_back_when_no_refs() {
        let doc = minimal_doc();
        let xml = generate_jats(&doc).unwrap();
        assert!(!xml.contains("<back>"));
    }

    #[test]
    fn test_ref_list_rendered() {
        let mut doc = minimal_doc();
        doc.references.push(Reference {
            id: "ref-1".into(),
            raw_text: "Smith (2020).".into(),
        });
        let xml = generate_jats(&doc).unwrap();
        assert!(xml.contains("<ref-list>"));
        assert!(xml.contains(r#"id="ref-1""#));
        assert!(xml.contains("Smith (2020)."));
    }

    #[test]
    fn test_subtitle_emitted_inside_title_group() {
        // JATS allows <subtitle> inside <title-group> — verify it is generated.
        let mut doc = minimal_doc();
        doc.subtitle = Some("A Descriptive Subtitle".into());
        let xml = generate_jats(&doc).unwrap();
        assert!(xml.contains("<subtitle>"), "expected <subtitle> element in XML");
        assert!(xml.contains("A Descriptive Subtitle"));
        // Must appear inside <title-group>
        let title_group_pos = xml.find("<title-group>").expect("<title-group> missing");
        let title_group_end = xml.find("</title-group>").expect("</title-group> missing");
        let subtitle_pos = xml.find("<subtitle>").expect("<subtitle> missing");
        assert!(
            subtitle_pos > title_group_pos && subtitle_pos < title_group_end,
            "<subtitle> must be inside <title-group>"
        );
    }

    #[test]
    fn test_no_subtitle_element_when_absent() {
        let doc = minimal_doc(); // subtitle: None
        let xml = generate_jats(&doc).unwrap();
        assert!(!xml.contains("<subtitle>"), "<subtitle> must not appear when subtitle is None");
    }
}
