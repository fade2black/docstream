use axum::{
    Router,
    routing::{get, post},
};

use super::{handlers, state::AppState};

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(handlers::health))
        .route("/search", post(handlers::search))
        .route("/ingest", post(handlers::ingest_document))
        .with_state(state)
}
