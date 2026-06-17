use crate::core::Embedding;
use crate::embedder::{Embedder, EmbedderError};
use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};

pub struct EmbeddingConfig {
    pub model: EmbeddingModel,
    pub max_length: usize,
    pub show_download_progress: bool,
}

impl Default for EmbeddingConfig {
    fn default() -> Self {
        Self {
            model: EmbeddingModel::AllMiniLML6V2,
            max_length: 512,
            show_download_progress: true,
        }
    }
}

impl EmbeddingConfig {
    pub fn builder() -> EmbeddingConfigBuilder {
        EmbeddingConfigBuilder::new()
    }
}

pub struct EmbeddingConfigBuilder {
    config: EmbeddingConfig,
}

impl EmbeddingConfigBuilder {
    pub fn new() -> Self {
        Self {
            config: EmbeddingConfig::default(),
        }
    }

    pub fn model(mut self, model: EmbeddingModel) -> Self {
        self.config.model = model;
        self
    }

    pub fn max_length(mut self, max_length: usize) -> Self {
        self.config.max_length = max_length;
        self
    }

    pub fn show_download_progress(mut self, show: bool) -> Self {
        self.config.show_download_progress = show;
        self
    }

    pub fn build(self) -> EmbeddingConfig {
        self.config
    }
}

pub struct FastEmbedder {
    model: TextEmbedding,
    model_name: String,
}

impl FastEmbedder {
    pub fn new(config: EmbeddingConfig) -> Result<Self, EmbedderError> {
        let model_name = format!("{:?}", config.model);

        let model = TextEmbedding::try_new(
            InitOptions::new(config.model)
                .with_max_length(config.max_length)
                .with_show_download_progress(config.show_download_progress),
        )
        .map_err(|e| EmbedderError::ModelUnavailable(e.to_string()))?;

        Ok(Self { model, model_name })
    }
}

#[async_trait::async_trait]
impl Embedder for FastEmbedder {
    async fn embed(
        &mut self,
        chunk_id: uuid::Uuid,
        text: &str,
    ) -> Result<Embedding, EmbedderError> {
        if text.trim().is_empty() {
            return Err(EmbedderError::EmptyText);
        }

        let vectors = self
            .model
            .embed(vec![text], None)
            .map_err(|e| EmbedderError::ApiError(e.to_string()))?;

        let vector = vectors
            .into_iter()
            .next()
            .ok_or_else(|| EmbedderError::ApiError("no vector returned".to_string()))?;

        Ok(Embedding {
            chunk_id,
            vector,
            model_name: self.model_name.clone(),
        })
    }
}
