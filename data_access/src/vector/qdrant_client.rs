use crate::vector::{VectorClient, VectorClientError};
use async_trait::async_trait;
use data_structures::intermediate::Chunk;
use point_id::PointIdOptions::Uuid as PointUuid;
use qdrant_client::{
    Qdrant, QdrantError,
    qdrant::{
        CountPointsBuilder, GetPointsBuilder, PointId, PointStruct, RetrievedPoint,
        ScrollPointsBuilder, UpsertPointsBuilder, Value, Vectors, point_id, vectors,
        vectors_output::VectorsOptions,
    },
};
use serde::Deserialize;
use serde_json;
use std::collections::HashMap;
use tracing::{info, instrument};
use uuid::Uuid;

impl From<QdrantError> for VectorClientError {
    fn from(value: QdrantError) -> Self {
        VectorClientError::Internal(value.to_string())
    }
}

pub struct QdrantClient {
    client: Qdrant,
}

#[derive(Debug, Deserialize, Clone)]
pub struct QdrantConfig {
    pub url: String,
}

#[derive(thiserror::Error, Debug)]
pub enum QdrantClientError {
    #[error("Internal Qdrant error: {0}")]
    Internal(#[from] QdrantError),
    #[error("Internal Qdrant error: {0}")]
    Other(String),
}

impl QdrantClient {
    pub fn new(config: QdrantConfig) -> Result<Self, QdrantClientError> {
        let client = Qdrant::from_url(&config.url).build()?;

        Ok(Self { client })
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

        let mut next_offset: Option<PointId> = None;

        loop {
            println!(".");
            let mut builder = ScrollPointsBuilder::new("paragraph")
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

        Ok(())
    }
}

#[async_trait]
impl VectorClient for QdrantClient {
    const COLLECTION_NAME_CHUNK: &'static str = "chunk";
    const COLLECTION_NAME_MOCK: &'static str = "mock";

    async fn store_chunk(&self, chunk: Chunk) -> Result<(), VectorClientError> {
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
                vectors_options: Some(vectors::VectorsOptions::Vector(chunk.embedding.into())),
            }),
            payload,
        };

        self.client
            .upsert_points(UpsertPointsBuilder::new(
                Self::COLLECTION_NAME_CHUNK,
                vec![point],
            ))
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
            .get_points(GetPointsBuilder::new(
                Self::COLLECTION_NAME_CHUNK,
                vec![point_id],
            ))
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
}

pub trait IntoChunk {
    fn into_chunk(self) -> Result<Chunk, VectorClientError>;
}

impl IntoChunk for RetrievedPoint {
    fn into_chunk(self) -> Result<Chunk, VectorClientError> {
        let embedding = get_dense_vector_from_point(&self)
            .ok_or_else(|| VectorClientError::FieldMissing("embedding".to_string()))?;

        let payload = self.payload;

        let league_year_team_idx = get_string_from_payload(&payload, "league_year_team_idx")
            .ok_or_else(|| VectorClientError::FieldMissing("league_year_team_idx".to_string()))?;

        let league_str = get_string_from_payload(&payload, "league")
            .ok_or_else(|| VectorClientError::FieldMissing("league".to_string()))?;
        let league: data_structures::file::League =
            serde_json::from_str(&league_str).map_err(|e| {
                VectorClientError::Internal(format!("Failed to deserialize League: {}", e))
            })?;

        let year = get_i64_from_payload(&payload, "year")
            .map(|i| i as u32)
            .ok_or_else(|| VectorClientError::FieldMissing("year".to_string()))?;

        let team_str = get_string_from_payload(&payload, "team")
            .ok_or_else(|| VectorClientError::FieldMissing("team".to_string()))?;
        let team: data_structures::file::TeamName =
            serde_json::from_str(&team_str).map_err(|e| {
                VectorClientError::Internal(format!("Failed to deserialize TeamName: {}", e))
            })?;

        let paragraph_sequence_id = get_i64_from_payload(&payload, "paragraph_sequence_id")
            .map(|i| i as u32)
            .ok_or_else(|| VectorClientError::FieldMissing("paragraph_sequence_id".to_string()))?;

        let chunk_sequence_id = get_i64_from_payload(&payload, "chunk_sequence_id")
            .map(|i| i as u32)
            .ok_or_else(|| VectorClientError::FieldMissing("chunk_sequence_id".to_string()))?;

        let idx_begin = get_i64_from_payload(&payload, "idx_begin")
            .map(|i| i as u32)
            .ok_or_else(|| VectorClientError::FieldMissing("idx_begin".to_string()))?;

        let idx_end = get_i64_from_payload(&payload, "idx_end")
            .map(|i| i as u32)
            .ok_or_else(|| VectorClientError::FieldMissing("idx_end".to_string()))?;

        let text = get_string_from_payload(&payload, "text")
            .ok_or_else(|| VectorClientError::FieldMissing("text".to_string()))?;

        Ok(Chunk {
            embedding,
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

pub fn get_string_from_payload(payload: &HashMap<String, Value>, key: &str) -> Option<String> {
    payload
        .get(key)
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}

pub fn get_i64_from_payload(payload: &HashMap<String, Value>, key: &str) -> Option<i64> {
    payload.get(key).and_then(|v| v.as_integer())
}

pub fn get_dense_vector_from_point(p: &RetrievedPoint) -> Option<Vec<f32>> {
    let v = match p.vectors.as_ref()?.vectors_options.as_ref()? {
        VectorsOptions::Vectors(v) => v,
        _ => return None,
    };
    let vector = v.vectors.get("dense")?.clone();
    Some(vector.data)
}

#[cfg(test)]
mod tests {

    use std::time::Duration;

    use crate::vector::{QdrantClient, QdrantConfig, VectorClient};
    use data_structures::file::{League, TeamName};
    use data_structures::intermediate::Chunk;
    use testcontainers::ImageExt;
    use testcontainers::core::IntoContainerPort;
    use testcontainers::{GenericImage, runners::AsyncRunner};
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_analyze() -> Result<(), anyhow::Error> {
        let client = QdrantClient::new(QdrantConfig {
            url: "http://localhost:6334".to_string(),
        });
        assert!(client.is_ok());

        let client = client.unwrap();

        client.analytics().await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_store_and_retrieve() -> Result<(), anyhow::Error> {
        let image = GenericImage::new("qdrant/qdrant", "v1.16")
            .with_exposed_port(6333.tcp())
            .with_exposed_port(6334.tcp())
            .with_mapped_port(7333, 6333.tcp())
            .with_mapped_port(7334, 6334.tcp())
            .start()
            .await
            .expect("Failed to start Qdrant");

        // sleep 5 seconds
        sleep(Duration::from_secs(5)).await;

        let client = QdrantClient::new(QdrantConfig {
            url: "http://localhost:7334".to_string(),
        });
        assert!(client.is_ok());

        let client = client.unwrap();

        // Create chunk and store in database
        let chunk = Chunk {
            embedding: vec![0.0; 1536],
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
