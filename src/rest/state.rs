use crate::pipeline::Pipeline;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub pipeline: Arc<Pipeline>,
}
