use crate::vector::{VectorClient, VectorClientError};
use async_trait::async_trait;
use data_structures::intermediate::Chunk;
use point_id::PointIdOptions::Uuid as PointUuid;
use qdrant_client::{
    Qdrant, QdrantError,
    qdrant::{
        CollectionExistsRequest, CountPointsBuilder, CreateCollectionBuilder, Distance,
        GetCollectionInfoResponse, GetPointsBuilder, NamedVectors, PointId, PointStruct,
        QueryPointsBuilder, RetrievedPoint, ScrollPointsBuilder, SparseVectorConfig,
        SparseVectorParamsBuilder, UpsertPointsBuilder, Value, VectorParamsBuilder,
        VectorParamsMap, Vectors, VectorsConfig, point_id, vector_output, vectors, vectors_config,
        vectors_output::VectorsOptions,
    },
};
use serde::Deserialize;
use serde_json;
use std::collections::HashMap;
use tracing::{info, instrument};
use uuid::Uuid;

#[derive(thiserror::Error, Debug)]
pub enum QdrantClientError {
    #[error("Internal Qdrant error: {0}")]
    Internal(#[from] QdrantError),
    #[error("Internal Qdrant error: {0}")]
    Other(String),
}

impl From<QdrantError> for VectorClientError {
    fn from(value: QdrantError) -> Self {
        VectorClientError::Internal(value.to_string())
    }
}

pub struct QdrantClient {
    client: Qdrant,
    embedding_size: u64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct QdrantConfig {
    pub url: String,
    pub embedding_size: u64,
    pub run: String,
}

impl QdrantClient {
    const COLLECTION_NAME_CHUNK: &'static str = "chunk";
    const EMBEDDING_NAME_DENSE: &'static str = "dense";
    const EMBEDDING_NAME_SPARSE: &'static str = "sparse";

    pub async fn new(config: QdrantConfig) -> Result<Self, VectorClientError> {
        info!(
            "New QdrantClient. url={}, size={}",
            config.url, config.embedding_size
        );

        let client = Qdrant::from_url(&config.url).build()?;

        let collection_exists = client
            .collection_exists(CollectionExistsRequest {
                collection_name: Self::COLLECTION_NAME_CHUNK.to_string(),
            })
            .await?;

        if !collection_exists {
            let dense_vector_config = VectorsConfig {
                config: Some(vectors_config::Config::ParamsMap(VectorParamsMap {
                    map: HashMap::from([(
                        Self::EMBEDDING_NAME_DENSE.to_string(),
                        VectorParamsBuilder::new(config.embedding_size, Distance::Cosine).into(),
                    )]),
                })),
            };

            let sparse_vector_config = SparseVectorConfig {
                map: HashMap::from([(
                    Self::EMBEDDING_NAME_SPARSE.to_string(),
                    SparseVectorParamsBuilder::default().into(),
                )]),
            };

            let builder = CreateCollectionBuilder::new(Self::COLLECTION_NAME_CHUNK)
                .vectors_config(dense_vector_config)
                .sparse_vectors_config(sparse_vector_config);

            client.create_collection(builder).await?;
        }

        // Ensure collection matches given dimensions
        let info = client.collection_info(Self::COLLECTION_NAME_CHUNK).await?;
        let size = from_collection_info_get_size(info.clone());
        let n = from_collection_info_get_n(info);
        info!("Collection: size={size}, n={n}");

        if size != config.embedding_size {
            return Err(VectorClientError::InvalidVectorDimension(format!(
                "Config embedding size is {} but existing collection embedding size is {}. Delete the existing collection or update the configuration.",
                config.embedding_size, size
            )));
        };

        Ok(Self {
            client,
            embedding_size: config.embedding_size,
        })
    }

    pub async fn analytics(&self) -> Result<(), QdrantClientError> {
        let collections_list = self.client.list_collections().await?;

        for collection in collections_list.collections {
            let count = self
                .client
                .count(CountPointsBuilder::new(&collection.name).exact(true))
                .await?;

            let count_result = count
                .result
                .ok_or_else(|| QdrantClientError::Other("Count result is empty".to_string()))?;
            println!(
                "Collection: {}, count: {}",
                collection.name, count_result.count,
            );
        }

        /*
        let mut next_offset: Option<PointId> = None;

        loop {
            println!(".");
            let mut builder = ScrollPointsBuilder::new(Self::COLLECTION_NAME_CHUNK)
                .with_payload(true)
                .with_vectors(true);

            if let Some(offset) = next_offset {
                builder = builder.offset(offset);
            }

            let scroll = self.client.scroll(builder.clone()).await?;

            for result in scroll.result {
                let id = result
                    .id
                    .ok_or_else(|| QdrantClientError::Other("Point ID is missing".to_string()))?;
                let point_id_options = id.point_id_options.ok_or_else(|| {
                    QdrantClientError::Other("Point ID options are missing".to_string())
                })?;
                let my_string = match point_id_options {
                    point_id::PointIdOptions::Num(n) => n.to_string(),
                    point_id::PointIdOptions::Uuid(u) => u.to_string(),
                };
                println!("\n  {}", my_string);

                let payload = result.payload;
                for (s, v) in payload {
                    println!(
                        "    {s:>15} {}",
                        v.to_string().chars().take(100).collect::<String>()
                    );
                }

                if let Some(vectors) = result.vectors {
                    if let Some(vector) = vectors.vectors_options {
                        match vector {
                            VectorsOptions::Vector(v) => {
                                println!("Vector {:?}", v.data);
                            }
                            VectorsOptions::Vectors(v) => {
                                for (name, v2) in v.vectors {
                                    println!(
                                        "              Named  {name} {:?}",
                                        v2.data.iter().take(5).collect::<Vec<&f32>>()
                                    );
                                }
                            }
                        }
                    }
                }
            }

            next_offset = scroll.next_page_offset;
            if next_offset.is_none() {
                break;
            }
        }
        */

        Ok(())
    }

    fn validate_embedding_size(&self, size: usize) -> Result<(), VectorClientError> {
        if size != self.embedding_size as usize {
            return Err(VectorClientError::InvalidVectorDimension(format!(
                "Expected embedding size {} but got {}",
                self.embedding_size, size
            )));
        }

        Ok(())
    }
}

#[async_trait]
impl VectorClient for QdrantClient {
    async fn store_chunk(&self, chunk: Chunk) -> Result<(), VectorClientError> {
        self.validate_embedding_size(chunk.dense_embedding.len())?;

        let id = chunk.to_uuid();
        let point_id: PointId = id.to_string().into();

        let mut payload: HashMap<String, Value> = HashMap::new();
        payload.insert(
            "league_year_team_idx".to_string(),
            chunk.league_year_team_idx.into(),
        );
        let league_str = serde_json::to_string(&chunk.league)
            .map_err(|e| VectorClientError::Internal(e.to_string()))?;
        payload.insert("league".to_string(), league_str.into());
        payload.insert("year".to_string(), (chunk.year as i64).into());
        let team_str = serde_json::to_string(&chunk.team)
            .map_err(|e| VectorClientError::Internal(e.to_string()))?;
        payload.insert("team".to_string(), team_str.into());
        payload.insert(
            "paragraph_sequence_id".to_string(),
            (chunk.paragraph_sequence_id as i64).into(),
        );
        payload.insert(
            "chunk_sequence_id".to_string(),
            (chunk.chunk_sequence_id as i64).into(),
        );
        payload.insert("idx_begin".to_string(), (chunk.idx_begin as i64).into());
        payload.insert("idx_end".to_string(), (chunk.idx_end as i64).into());
        payload.insert("text".to_string(), chunk.text.into());

        let point = PointStruct {
            id: Some(point_id),
            vectors: Some(Vectors {
                vectors_options: Some(vectors::VectorsOptions::Vectors(NamedVectors {
                    vectors: HashMap::from([
                        (
                            Self::EMBEDDING_NAME_DENSE.to_string(),
                            chunk.dense_embedding.into(),
                        ),
                        (
                            Self::EMBEDDING_NAME_SPARSE.to_string(),
                            chunk.sparse_embedding.into(),
                        ),
                    ]),
                })),
            }),
            payload,
        };

        self.client
            .upsert_points(
                UpsertPointsBuilder::new(Self::COLLECTION_NAME_CHUNK, vec![point]).wait(true),
            )
            .await?;

        Ok(())
    }

    #[instrument(name = "vector", skip(self))]
    async fn get_all_chunks(&self) -> Result<Vec<Chunk>, VectorClientError> {
        info!("Retrieving all chunks from Qdrant");
        let count_response = self
            .client
            .count(CountPointsBuilder::new(Self::COLLECTION_NAME_CHUNK).exact(true))
            .await?;

        let total_count = count_response
            .result
            .ok_or_else(|| VectorClientError::Internal("Count result missing".to_string()))?
            .count;

        info!("Total chunks to retrieve: {}", total_count);
        if total_count == 0 {
            return Ok(vec![]);
        }

        let scroll_response = self
            .client
            .scroll(
                ScrollPointsBuilder::new(Self::COLLECTION_NAME_CHUNK)
                    .with_payload(true)
                    .with_vectors(true)
                    .limit(total_count as u32),
            )
            .await?;

        let mut chunks = Vec::with_capacity(scroll_response.result.len());
        for retrieved_point in scroll_response.result {
            chunks.push(retrieved_point.into_chunk()?);
        }

        Ok(chunks)
    }

    async fn get_chunk_by_id(&self, id: Uuid) -> Result<Chunk, VectorClientError> {
        let point_id = PointId {
            point_id_options: Some(PointUuid(id.to_string())),
        };

        let retrieved_points = self
            .client
            .get_points(
                GetPointsBuilder::new(Self::COLLECTION_NAME_CHUNK, vec![point_id])
                    .with_vectors(true)
                    .with_payload(true),
            )
            .await?
            .result;

        let Some(point) = retrieved_points.first() else {
            return Err(VectorClientError::NotFound(format!(
                "Chunk with ID {} not found",
                id
            )));
        };

        Ok(point.clone().into_chunk()?)
    }

    async fn search_chunks_by_embedding(
        &self,
        embedding: Vec<f32>,
        limit: u64,
    ) -> Result<Vec<Chunk>, VectorClientError> {
        self.validate_embedding_size(embedding.len())?;

        let query = QueryPointsBuilder::new(Self::COLLECTION_NAME_CHUNK)
            .query(embedding)
            .using(Self::EMBEDDING_NAME_DENSE)
            .limit(limit)
            .with_payload(true);

        let response = self.client.query(query).await?;

        // println!("response: {response:?}");

        for point in &response.result {
            println!("\n");
            println!("score: {}", point.score);
            println!(
                "lyti: {}",
                from_payload_get_string(&point.payload, "league_year_team_idx").unwrap()
            );
            println!(
                "text: {}",
                from_payload_get_string(&point.payload, "text").unwrap()
            );
        }

        response
            .result
            .into_iter()
            .map(|point| point.payload.into_chunk())
            .collect()
    }
}

pub trait IntoChunk {
    fn into_chunk(self) -> Result<Chunk, VectorClientError>;
}

impl IntoChunk for RetrievedPoint {
    fn into_chunk(self) -> Result<Chunk, VectorClientError> {
        let dense_embedding = from_point_get_dense_vector(&self).ok_or_else(|| {
            VectorClientError::FieldMissing(QdrantClient::EMBEDDING_NAME_DENSE.to_string())
        })?;

        let sparse_embedding = from_point_get_sparse_vector(&self).ok_or_else(|| {
            VectorClientError::FieldMissing(QdrantClient::EMBEDDING_NAME_SPARSE.to_string())
        })?;

        let payload = self.payload;

        let mut chunk: Chunk = payload.into_chunk()?;

        chunk.dense_embedding = dense_embedding;
        chunk.sparse_embedding = sparse_embedding;

        Ok(chunk)
    }
}

impl IntoChunk for HashMap<String, Value> {
    fn into_chunk(self) -> Result<Chunk, VectorClientError> {
        let league_year_team_idx = from_payload_get_string(&self, "league_year_team_idx")
            .ok_or_else(|| VectorClientError::FieldMissing("league_year_team_idx".to_string()))?;

        let league_str = from_payload_get_string(&self, "league")
            .ok_or_else(|| VectorClientError::FieldMissing("league".to_string()))?;
        let league: data_structures::file::League =
            serde_json::from_str(&league_str).map_err(|e| {
                VectorClientError::Internal(format!("Failed to deserialize League: {}", e))
            })?;

        let year = from_payload_get_i64(&self, "year")
            .map(|i| i as u32)
            .ok_or_else(|| VectorClientError::FieldMissing("year".to_string()))?;

        let team_str = from_payload_get_string(&self, "team")
            .ok_or_else(|| VectorClientError::FieldMissing("team".to_string()))?;
        let team: data_structures::file::TeamName =
            serde_json::from_str(&team_str).map_err(|e| {
                VectorClientError::Internal(format!("Failed to deserialize TeamName: {}", e))
            })?;

        let paragraph_sequence_id = from_payload_get_i64(&self, "paragraph_sequence_id")
            .map(|i| i as u32)
            .ok_or_else(|| VectorClientError::FieldMissing("paragraph_sequence_id".to_string()))?;

        let chunk_sequence_id = from_payload_get_i64(&self, "chunk_sequence_id")
            .map(|i| i as u32)
            .ok_or_else(|| VectorClientError::FieldMissing("chunk_sequence_id".to_string()))?;

        let idx_begin = from_payload_get_i64(&self, "idx_begin")
            .map(|i| i as u32)
            .ok_or_else(|| VectorClientError::FieldMissing("idx_begin".to_string()))?;

        let idx_end = from_payload_get_i64(&self, "idx_end")
            .map(|i| i as u32)
            .ok_or_else(|| VectorClientError::FieldMissing("idx_end".to_string()))?;

        let text = from_payload_get_string(&self, "text")
            .ok_or_else(|| VectorClientError::FieldMissing("text".to_string()))?;

        Ok(Chunk {
            dense_embedding: vec![],
            sparse_embedding: HashMap::new(),
            league_year_team_idx,
            league,
            year,
            team,
            paragraph_sequence_id,
            chunk_sequence_id,
            idx_begin,
            idx_end,
            text,
        })
    }
}

fn from_payload_get_string(payload: &HashMap<String, Value>, key: &str) -> Option<String> {
    payload
        .get(key)
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}

fn from_payload_get_i64(payload: &HashMap<String, Value>, key: &str) -> Option<i64> {
    payload.get(key).and_then(|v| v.as_integer())
}

fn from_point_get_dense_vector(p: &RetrievedPoint) -> Option<Vec<f32>> {
    let v = match p.vectors.as_ref()?.vectors_options.as_ref()? {
        VectorsOptions::Vectors(v) => v,
        _ => return None,
    };
    let vector = v.vectors.get(QdrantClient::EMBEDDING_NAME_DENSE)?.clone();

    if let vector_output::Vector::Dense(v) = vector.into_vector() {
        Some(v.data)
    } else {
        None
    }
}

fn from_point_get_sparse_vector(p: &RetrievedPoint) -> Option<HashMap<u32, f32>> {
    let v = match p.vectors.as_ref()?.vectors_options.as_ref()? {
        VectorsOptions::Vectors(v) => v,
        _ => return None,
    };
    let vector = v.vectors.get(QdrantClient::EMBEDDING_NAME_SPARSE)?.clone();

    if let vector_output::Vector::Sparse(v) = vector.into_vector() {
        let values = v.values;
        let indices = v.indices;
        Some(indices.into_iter().zip(values.into_iter()).collect())
    } else {
        None
    }
}

fn from_collection_info_get_size(info: GetCollectionInfoResponse) -> u64 {
    let params = info.result.unwrap().config.unwrap().params.unwrap();
    let config = params.vectors_config.unwrap().config.unwrap();
    let dimension = match config {
        vectors_config::Config::Params(_vector_params) => {
            todo!()
        }
        vectors_config::Config::ParamsMap(vector_params_map) => {
            let params = vector_params_map
                .map
                .get(QdrantClient::EMBEDDING_NAME_DENSE)
                .unwrap();
            params.size
        }
    };
    dimension
}

fn from_collection_info_get_n(info: GetCollectionInfoResponse) -> u64 {
    info.result.unwrap().points_count.unwrap()
}

#[cfg(test)]
mod tests {

    use std::collections::HashMap;
    use std::time::Duration;

    use crate::vector::{QdrantClient, QdrantConfig, VectorClient};
    use data_structures::file::{League, TeamName};
    use data_structures::intermediate::Chunk;
    use testcontainers::ImageExt;
    use testcontainers::core::IntoContainerPort;
    use testcontainers::{GenericImage, runners::AsyncRunner};
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_create_client() -> Result<(), Box<dyn std::error::Error>> {
        let client = QdrantClient::new(QdrantConfig {
            url: "http://localhost:6334".to_string(),
            embedding_size: 1536,
            run: "test_run".to_string(),
        })
        .await;

        assert!(client.is_ok());

        Ok(())
    }

    #[tokio::test]
    async fn test_analyze() -> Result<(), anyhow::Error> {
        let client = QdrantClient::new(QdrantConfig {
            url: "http://localhost:6334".to_string(),
            embedding_size: 1536,
            run: "test_run".to_string(),
        })
        .await;

        assert!(client.is_ok());

        let client = client.unwrap();

        client.analytics().await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_store_and_retrieve() -> Result<(), anyhow::Error> {
        let _image = GenericImage::new("qdrant/qdrant", "v1.16")
            .with_exposed_port(6333.tcp())
            .with_exposed_port(6334.tcp())
            .with_mapped_port(7333, 6333.tcp())
            .with_mapped_port(7334, 6334.tcp())
            .start()
            .await
            .expect("Failed to start Qdrant");

        sleep(Duration::from_secs(2)).await;

        let client = QdrantClient::new(QdrantConfig {
            url: "http://localhost:7334".to_string(),
            embedding_size: 3,
            run: "test_run".to_string(),
        })
        .await;

        assert!(client.is_ok());

        let client = client.unwrap();

        // Create chunk and store in database
        let chunk = Chunk {
            dense_embedding: vec![0.0; 3],
            sparse_embedding: HashMap::from([(1, 1.0), (2, 2.0)]),
            league_year_team_idx: "test_league__1998__test_team__0".to_string(),
            league: League::try_from("test_league").unwrap(),
            year: 1998,
            team: TeamName::new("test_team"),
            paragraph_sequence_id: 0,
            chunk_sequence_id: 0,
            idx_begin: 0,
            idx_end: 0,
            text: "test_text".to_string(),
        };

        let id = chunk.to_uuid();

        client.store_chunk(chunk.clone()).await?;

        client.analytics().await?;

        let retrieved_chunk = client.get_chunk_by_id(id).await?;

        assert_eq!(
            chunk.league_year_team_idx,
            retrieved_chunk.league_year_team_idx
        );

        Ok(())
    }
}
