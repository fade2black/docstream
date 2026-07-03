use crate::config::loader::QdrantConfig;
use crate::core::Chunk;
use crate::store::{ChunkMetadata, SearchResult, VectorStore, VectorStoreError};
use qdrant_client::Payload;
use qdrant_client::Qdrant;
use qdrant_client::qdrant::point_id::PointIdOptions;
use qdrant_client::qdrant::{
    CreateCollectionBuilder, Distance, PointId, PointStruct, QueryPointsBuilder, ScoredPoint,
    UpsertPointsBuilder, VectorInput, VectorParamsBuilder,
};
use std::collections::HashMap;
use tracing::info;
use uuid::Uuid;

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

    fn map_search_result(point: ScoredPoint) -> Result<SearchResult, VectorStoreError> {
        Ok(SearchResult {
            chunk_id: Self::parse_chunk_id(point.id)?,
            score: point.score,
            metadata: Self::parse_metadata(point.payload)?,
        })
    }

    fn parse_chunk_id(id: Option<PointId>) -> Result<Uuid, VectorStoreError> {
        match id.and_then(|id| id.point_id_options) {
            Some(PointIdOptions::Uuid(s)) => {
                Uuid::parse_str(&s).map_err(|_| VectorStoreError::InvalidId(s))
            }
            _ => Err(VectorStoreError::MissingId),
        }
    }

    fn parse_metadata(
        payload: HashMap<String, qdrant_client::qdrant::Value>,
    ) -> Result<ChunkMetadata, VectorStoreError> {
        let json = serde_json::Value::Object(
            payload
                .into_iter()
                .map(|(k, v)| (k, v.into_json()))
                .collect(),
        );

        serde_json::from_value(json).map_err(|e| VectorStoreError::SearchError(e.to_string()))
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

    async fn search(
        &self,
        vec: &[f32],
        top_k: usize,
    ) -> Result<Vec<SearchResult>, VectorStoreError> {
        let query = QueryPointsBuilder::new(&self.collection_name)
            .query(VectorInput::new_dense(vec.to_vec()))
            .limit(top_k as u64)
            .with_payload(true);

        let response = self
            .client
            .query(query)
            .await
            .map_err(|e| VectorStoreError::SearchError(e.to_string()))?;

        response
            .result
            .into_iter()
            .map(Self::map_search_result)
            .collect()
    }
}
