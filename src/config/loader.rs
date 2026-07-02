#[derive(Debug, serde::Deserialize)]
pub struct EmbedderConfig {
    pub provider: String,
    pub endpoint: String,
    pub model: String,
    pub api_key: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
pub struct QdrantConfig {
    /// Qdrant server URL, e.g. "http://localhost:6334" or Qdrant Cloud endpoint
    pub url: String,

    /// Optional API key for Qdrant Cloud
    pub api_key: String,

    /// Collection name to store embeddings in
    pub collection_name: String,

    /// Vector dimension — must match the embedder's output size
    pub vector_size: u64,
}

#[derive(serde::Deserialize)]
pub struct AppConfig {
    pub embedder: EmbedderConfig,
    pub qdrant: QdrantConfig,
}

#[derive(Debug)]
pub enum ConfigError {
    UnableToLoadFromFlle(String),
}

impl AppConfig {
    pub fn from_env() -> anyhow::Result<Self> {
        // Load .env if present (local dev only)
        dotenvy::dotenv().ok();

        let embedder = EmbedderConfig {
            provider: std::env::var("EMBEDDER_PROVIDER")?,
            endpoint: std::env::var("EMBEDDER_ENDPOINT")?,
            model: std::env::var("EMBEDDER_MODEL")?,
            api_key: std::env::var("EMBEDDER_API_KEY").ok(),
        };

        let qdrant = QdrantConfig {
            api_key: std::env::var("QDRANT_API_KEY")?,
            url: std::env::var("QDRANT_URL")?,
            vector_size: std::env::var("QDRANT_VECTOR_SIZE")?.parse()?,
            collection_name: std::env::var("QDRANT_COLLECTION_NAME")?,
        };

        Ok(Self { embedder, qdrant })
    }
}
