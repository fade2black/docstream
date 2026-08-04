use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct SearchRequest {
    pub query: String,
    pub top_k: usize,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SearchResponse {
    pub results: Vec<SearchResult>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SearchResult {
    pub chunk_id: String,
    pub score: f32,
    pub text: String,
    pub document_id: String,
}

#[derive(Deserialize)]
pub struct IngestDocumentRequest {
    pub doc_ref: String,
}
