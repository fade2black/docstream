use crate::embedder::{Embedder, EmbedderConfig, EmbedderError};

#[derive(Debug)]
pub struct LocalEmbedder {
    pub config: EmbedderConfig,
}
impl LocalEmbedder {
    pub fn new(config: EmbedderConfig) -> Self {
        Self { config }
    }
}

#[async_trait::async_trait]
impl Embedder for LocalEmbedder {
    fn endpoint(&self) -> &str {
        &self.config.endpoint
    }

    fn build_request(&self, text: &str) -> Result<serde_json::Value, EmbedderError> {
        Ok(serde_json::json!({
            "input": text,
            "model": self.config.model,
        }))
    }

    fn parse_response(&self, json: &serde_json::Value) -> Result<Vec<f32>, EmbedderError> {
        let arr = json["data"][0]["embedding"]
            .as_array()
            .ok_or_else(|| EmbedderError::ApiError("missing embedding".into()))?;

        let mut vec = Vec::with_capacity(arr.len());
        for val in arr {
            let num = val
                .as_f64()
                .ok_or_else(|| EmbedderError::ApiError("invalid embedding value".into()))?;
            vec.push(num as f32);
        }

        Ok(vec)
    }
}
