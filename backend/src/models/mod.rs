use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub title: Option<String>,
    pub subtitle: Option<String>,
    pub authors: Vec<Author>,
    pub abstract_text: Option<String>,
    pub sections: Vec<Section>,
    pub references: Vec<Reference>,
    pub warnings: Vec<String>,
}

impl Document {
    pub fn new() -> Self {
        Self {
            title: None,
            subtitle: None,
            authors: Vec::new(),
            abstract_text: None,
            sections: Vec::new(),
            references: Vec::new(),
            warnings: Vec::new(),
        }
    }

    pub fn add_warning(&mut self, msg: impl Into<String>) {
        self.warnings.push(msg.into());
    }

    pub fn metadata(&self) -> DocumentMetadata {
        DocumentMetadata {
            title: self.title.clone(),
            author_count: self.authors.len(),
            section_count: self.sections.len(),
            reference_count: self.references.len(),
            has_abstract: self.abstract_text.is_some(),
        }
    }
}

impl Default for Document {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Author {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Section {
    pub heading: Option<String>,
    pub level: u8,
    pub body: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Reference {
    pub id: String,
    pub raw_text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DocumentContent {
    pub title: Option<String>,
    pub subtitle: Option<String>,
    pub authors: Vec<Author>,
    pub abstract_text: Option<String>,
    pub sections: Vec<Section>,
    pub references: Vec<Reference>,
}

impl From<&Document> for DocumentContent {
    fn from(doc: &Document) -> Self {
        Self {
            title: doc.title.clone(),
            subtitle: doc.subtitle.clone(),
            authors: doc.authors.clone(),
            abstract_text: doc.abstract_text.clone(),
            sections: doc.sections.clone(),
            references: doc.references.clone(),
        }
    }
}

impl From<DocumentContent> for Document {
    fn from(content: DocumentContent) -> Self {
        Self {
            title: content.title,
            subtitle: content.subtitle,
            authors: content.authors,
            abstract_text: content.abstract_text,
            sections: content.sections,
            references: content.references,
            warnings: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DocumentMetadata {
    pub title: Option<String>,
    pub author_count: usize,
    pub section_count: usize,
    pub reference_count: usize,
    pub has_abstract: bool,
}
