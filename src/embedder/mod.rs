pub mod local_embedder;

#[async_trait::async_trait]
pub trait Embedder: Send + Sync {
    /// Provider-specific endpoint
    fn endpoint(&self) -> &str;

    /// Provider-specific JSON request
    fn build_request(&self, text: &str) -> Result<serde_json::Value, EmbedderError>;

    /// Provider-specific JSON response parser
    fn parse_response(&self, json: &serde_json::Value) -> Result<Vec<f32>, EmbedderError>;

    /// Provider-specific HTTP client
    fn client(&self) -> &reqwest::Client;

    /// Generic embedding logic (same for all providers)
    async fn embed(&self, text: &str) -> Result<Vec<f32>, EmbedderError> {
        if text.trim().is_empty() {
            return Err(EmbedderError::EmptyText);
        }

        let body = self.build_request(text)?;

        let client = self.client();
        let resp = client
            .post(self.endpoint())
            .json(&body)
            .send()
            .await
            .map_err(|e| EmbedderError::ApiError(e.to_string()))?;

        let json: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| EmbedderError::ApiError(e.to_string()))?;

        self.parse_response(&json)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum EmbedderError {
    #[error("embedding API error: {0}")]
    ApiError(String),

    #[error("empty text provided")]
    EmptyText,

    #[error("model not available: {0}")]
    ModelUnavailable(String),
}
