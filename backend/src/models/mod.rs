use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub title: Option<String>,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Author {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section {
    pub heading: Option<String>,
    pub level: u8,
    pub body: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reference {
    pub id: String,
    pub raw_text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentMetadata {
    pub title: Option<String>,
    pub author_count: usize,
    pub section_count: usize,
    pub reference_count: usize,
    pub has_abstract: bool,
}
