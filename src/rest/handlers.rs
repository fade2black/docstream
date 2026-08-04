use crate::core::DocumentJob;
use crate::rest::dto::{IngestDocumentRequest, SearchRequest, SearchResponse, SearchResult};
use crate::rest::state::AppState;
use axum::response::IntoResponse;
use tracing::{error, info};
use uuid::Uuid;

use axum::{Json, extract::State, http::StatusCode};

pub async fn health() -> impl IntoResponse {
    (StatusCode::OK, "OK")
}

pub async fn search(
    State(state): State<AppState>,
    Json(req): Json<SearchRequest>,
) -> impl IntoResponse {
    match state.pipeline.search(&req.query, req.top_k).await {
        Ok(results) => {
            let dto: Vec<SearchResult> = results
                .into_iter()
                .map(|r| SearchResult {
                    chunk_id: r.chunk_id.to_string(),
                    score: r.score,
                    text: r.metadata.text,
                    document_id: r.metadata.document_id,
                })
                .collect();

            (StatusCode::OK, Json(SearchResponse { results: dto }))
        }
        Err(err) => {
            error!("search error: {}", err);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(SearchResponse { results: vec![] }),
            )
        }
    }
}

pub async fn ingest_document(
    State(state): State<AppState>,
    Json(req): Json<IngestDocumentRequest>,
) -> impl IntoResponse {
    let doc_id = Uuid::new_v4();

    let job = DocumentJob {
        doc_id,
        doc_ref: req.doc_ref,
    };

    match state.pipeline.ingest(job).await {
        Ok(_) => {
            info!("doc_id={} queued for ingestion", doc_id);
            StatusCode::ACCEPTED
        }
        Err(err) => {
            error!("ingest_document error: {}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}
