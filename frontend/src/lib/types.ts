export interface Author {
  name: string;
}

export interface Section {
  heading: string | null;
  level: number;
  body: string[];
}

export interface Reference {
  id: string;
  raw_text: string;
}

export interface DocumentContent {
  title: string | null;
  authors: Author[];
  abstract_text: string | null;
  sections: Section[];
  references: Reference[];
}

export interface DocumentMetadata {
  title: string | null;
  author_count: number;
  section_count: number;
  reference_count: number;
  has_abstract: boolean;
}

export interface ConvertResponse {
  success: boolean;
  xml: string;
  warnings: string[];
  metadata: DocumentMetadata;
  document: DocumentContent;
}

export interface ConvertError {
  success: false;
  error: string;
}

export interface RegenerateResponse {
  success: boolean;
  xml: string;
  warnings: string[];
}
