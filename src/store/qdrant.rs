use crate::config::loader::QdrantConfig;
use crate::core::Chunk;
use crate::store::{VectorStore, VectorStoreError};
use qdrant_client::Payload;
use qdrant_client::Qdrant;
use qdrant_client::qdrant::{
    CreateCollectionBuilder, Distance, PointStruct, UpsertPointsBuilder, VectorParamsBuilder,
};
use std::collections::HashMap;
use tracing::info;

pub struct QdrantStore {
    client: Qdrant,
    collection_name: String,
}

impl QdrantStore {
    pub async fn new(config: QdrantConfig) -> Result<Self, VectorStoreError> {
        info!("Connecting to Qdrant at {}", config.url);

        let client = Qdrant::from_url(&config.url)
            .api_key(config.api_key)
            .build()
            .map_err(|e| VectorStoreError::Unavailable(e.to_string()))?;

        // Create collection if it doesn't exist
        let exists = client
            .collection_exists(&config.collection_name)
            .await
            .map_err(|e| VectorStoreError::Unavailable(e.to_string()))?;

        if !exists {
            client
                .create_collection(
                    CreateCollectionBuilder::new(&config.collection_name).vectors_config(
                        VectorParamsBuilder::new(config.vector_size, Distance::Cosine),
                    ),
                )
                .await
                .map_err(|e| VectorStoreError::Unavailable(e.to_string()))?;
        }

        info!(
            "Connected to Qdrant, collection '{}'",
            config.collection_name,
        );

        Ok(Self {
            client,
            collection_name: config.collection_name,
        })
    }
}

#[async_trait::async_trait]
impl VectorStore for QdrantStore {
    async fn insert(&self, vec: &[f32], chunk: &Chunk) -> Result<(), VectorStoreError> {
        let mut fields = HashMap::new();

        fields.insert("text".to_string(), serde_json::json!(chunk.text));
        fields.insert(
            "document_id".to_string(),
            serde_json::json!(chunk.doc_id.to_string()),
        );
        let payload: Payload = fields.into();

        let point = PointStruct::new(chunk.id.to_string(), vec.to_vec(), payload);

        self.client
            .upsert_points(UpsertPointsBuilder::new(&self.collection_name, vec![point]))
            .await
            .map_err(|e| VectorStoreError::StoreError(e.to_string()))?;

        Ok(())
    }
}
