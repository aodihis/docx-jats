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
}

export interface ConvertError {
  success: false;
  error: string;
}
