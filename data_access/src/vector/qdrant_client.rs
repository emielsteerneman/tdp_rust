use crate::vector::{VectorClient, VectorClientError};
use async_trait::async_trait;
use data_structures::{
    file::{League, TeamName},
    filter::Filter,
    intermediate::Chunk,
};
use point_id::PointIdOptions::Uuid as PointUuid;
use qdrant_client::{
    Qdrant, QdrantError,
    qdrant::{
        CollectionExistsRequest, Condition, CountPointsBuilder, CreateCollectionBuilder, Distance,
        Fusion, GetCollectionInfoResponse, GetPointsBuilder, NamedVectors, PointId, PointStruct,
        PrefetchQueryBuilder, Query, QueryPointsBuilder, RetrievedPoint, ScrollPointsBuilder,
        SparseVector, SparseVectorConfig, SparseVectorParamsBuilder, UpsertPointsBuilder, Value,
        Vector, VectorParamsBuilder, VectorParamsMap, Vectors, VectorsConfig, point_id,
        vector_output, vectors, vectors_config, vectors_output::VectorsOptions,
    },
};
use serde::Deserialize;
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

    const KEY_LEAGUE: &'static str = "league";
    const KEY_YEAR: &'static str = "year";
    const KEY_TEAM: &'static str = "team";
    const KEY_LYTI: &'static str = "lyti";

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

        // League Year Team
        payload.insert(Self::KEY_LEAGUE.into(), chunk.league.name_pretty.into());
        payload.insert(Self::KEY_YEAR.into(), (chunk.year as i64).into());
        payload.insert(Self::KEY_TEAM.into(), chunk.team.name_pretty.into());
        payload.insert(Self::KEY_LYTI.into(), chunk.league_year_team_idx.into());
        // Structure
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
        // Text
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
                        (Self::EMBEDDING_NAME_SPARSE.to_string(), {
                            let mut pairs: Vec<_> =
                                chunk.sparse_embedding.clone().into_iter().collect();
                            // TODO Figure out if this sort_by_key is actually needed.
                            // It's not needed for HashMap equality, but maybe Qdrant needs it?
                            pairs.sort_by_key(|(k, _)| *k);
                            let (indices, values): (Vec<_>, Vec<_>) = pairs.into_iter().unzip();
                            Vector::from(SparseVector { indices, values })
                        }),
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

    async fn search_chunks(
        &self,
        dense: Option<Vec<f32>>,
        sparse: Option<HashMap<u32, f32>>,
        limit: u64,
        filter: Option<Filter>,
    ) -> Result<Vec<(Chunk, f32)>, VectorClientError> {
        if let Some(ref d) = dense {
            self.validate_embedding_size(d.len())?;
        }

        let mut query_builder = QueryPointsBuilder::new(Self::COLLECTION_NAME_CHUNK)
            .limit(limit)
            .with_payload(true);

        if let Some(f) = filter {
            let mut conditions = Vec::new();

            if let Some(leagues) = f.leagues {
                if !leagues.is_empty() {
                    info!("Adding league filter {:?}", leagues);
                    conditions.push(Condition::matches(
                        Self::KEY_LEAGUE,
                        leagues.into_iter().collect::<Vec<String>>(),
                    ));
                }
            }

            if let Some(years) = f.years {
                if !years.is_empty() {
                    info!("Adding year filter {:?}", years);
                    conditions.push(Condition::matches(
                        Self::KEY_YEAR,
                        years.into_iter().map(|y| y as i64).collect::<Vec<i64>>(),
                    ));
                }
            }

            if let Some(teams) = f.teams {
                if !teams.is_empty() {
                    info!("Adding team filter {:?}", teams);
                    conditions.push(Condition::matches(
                        Self::KEY_TEAM,
                        teams.into_iter().collect::<Vec<String>>(),
                    ));
                }
            }

            if let Some(indexes) = f.league_year_team_indexes {
                if !indexes.is_empty() {
                    info!("Adding lity filter {:?}", indexes);
                    conditions.push(Condition::matches(
                        Self::KEY_LYTI,
                        indexes.into_iter().collect::<Vec<String>>(),
                    ));
                }
            }

            if !conditions.is_empty() {
                query_builder =
                    query_builder.filter(qdrant_client::qdrant::Filter::must(conditions));
            }
        }

        match (dense, sparse) {
            (Some(dense_vector), Some(sparse_vector)) => {
                let sparse_vector: Vec<(u32, f32)> = sparse_vector.into_iter().collect();

                query_builder = query_builder.add_prefetch(
                    PrefetchQueryBuilder::default()
                        .query(Query::new_nearest(dense_vector))
                        .using(Self::EMBEDDING_NAME_DENSE)
                        .limit(limit),
                );

                query_builder = query_builder.add_prefetch(
                    PrefetchQueryBuilder::default()
                        .query(sparse_vector)
                        .using(Self::EMBEDDING_NAME_SPARSE)
                        .limit(limit),
                );

                query_builder = query_builder.query(Query::new_fusion(Fusion::Rrf));
            }
            (Some(d), None) => {
                query_builder = query_builder.query(d).using(Self::EMBEDDING_NAME_DENSE);
            }
            (None, Some(s)) => {
                let sparse_vector: Vec<(u32, f32)> = s.into_iter().collect();
                query_builder = query_builder
                    .query(sparse_vector)
                    .using(Self::EMBEDDING_NAME_SPARSE);
            }
            (None, None) => return Err(VectorClientError::Empty),
        };

        let response = self.client.query(query_builder).await?;

        response
            .result
            .into_iter()
            .map(|point| point.payload.into_chunk().map(|c| (c, point.score)))
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
        // League Year Team Index
        let league_str = from_payload_get_string(&self, QdrantClient::KEY_LEAGUE)
            .ok_or_else(|| VectorClientError::FieldMissing(QdrantClient::KEY_LEAGUE.to_string()))?;
        let league: League = league_str.as_str().try_into().map_err(|e| {
            VectorClientError::Internal(format!("Failed to deserialize League: {}", e))
        })?;

        let year = from_payload_get_i64(&self, QdrantClient::KEY_YEAR)
            .map(|i| i as u32)
            .ok_or_else(|| VectorClientError::FieldMissing(QdrantClient::KEY_YEAR.to_string()))?;

        let team_str = from_payload_get_string(&self, QdrantClient::KEY_TEAM)
            .ok_or_else(|| VectorClientError::FieldMissing(QdrantClient::KEY_TEAM.to_string()))?;
        let team = TeamName::new(&team_str);

        let league_year_team_idx = from_payload_get_string(&self, QdrantClient::KEY_LYTI)
            .ok_or_else(|| VectorClientError::FieldMissing(QdrantClient::KEY_LYTI.to_string()))?;

        // Structure
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

        // Text
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
    use data_structures::filter::Filter;
    use data_structures::intermediate::Chunk;
    use testcontainers::ImageExt;
    use testcontainers::core::IntoContainerPort;
    use testcontainers::{GenericImage, runners::AsyncRunner};
    use tokio::time::sleep;

    fn normalize(vec: Vec<f32>) -> Vec<f32> {
        let len_squared: f32 = vec.iter().map(|f| f * f).sum();
        let len = len_squared.sqrt();
        vec.into_iter().map(|f| f / len).collect()
    }

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

        let dense_embedding = normalize(vec![1.0, 3.0, 2.0]);
        let sparse_embedding = HashMap::from([(1, 1.0), (3, 3.0), (2, 2.0)]);

        // Create chunk and store in database
        let chunk = Chunk {
            dense_embedding: dense_embedding.clone(),
            sparse_embedding: sparse_embedding.clone(),
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
        assert_eq!(dense_embedding, retrieved_chunk.dense_embedding);
        assert_eq!(sparse_embedding, retrieved_chunk.sparse_embedding);

        Ok(())
    }

    #[tokio::test]
    async fn test_store_and_retrieve_with_filter() -> Result<(), anyhow::Error> {
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

        let dense_embedding = normalize(vec![1.0, 3.0, 2.0]);
        let sparse_embedding = HashMap::from([(1, 1.0), (3, 3.0), (2, 2.0)]);

        // Create chunk and store in database
        let chunk_1 = Chunk {
            dense_embedding: dense_embedding.clone(),
            sparse_embedding: sparse_embedding.clone(),
            league_year_team_idx: "test_league_1__1998__test_team_1__0".to_string(),
            league: League::try_from("test_league_1").unwrap(),
            year: 1998,
            team: TeamName::new("test_team_1"),
            paragraph_sequence_id: 0,
            chunk_sequence_id: 0,
            idx_begin: 0,
            idx_end: 0,
            text: "test_text_1".to_string(),
        };

        // Create chunk and store in database
        let chunk_2_1 = Chunk {
            dense_embedding: dense_embedding.clone(),
            sparse_embedding: sparse_embedding.clone(),
            league_year_team_idx: "test_league_2__2008__test_team_2__0".to_string(),
            league: League::try_from("test_league_2").unwrap(),
            year: 2008,
            team: TeamName::new("test_team_2"),
            paragraph_sequence_id: 0,
            chunk_sequence_id: 0,
            idx_begin: 0,
            idx_end: 0,
            text: "test_text_2".to_string(),
        };

        let chunk_2_2 = Chunk {
            dense_embedding: dense_embedding.clone(),
            sparse_embedding: sparse_embedding.clone(),
            league_year_team_idx: "test_league_2__2008__test_team_2__1".to_string(),
            league: League::try_from("test_league_2").unwrap(),
            year: 2008,
            team: TeamName::new("test_team_2"),
            paragraph_sequence_id: 0,
            chunk_sequence_id: 0,
            idx_begin: 0,
            idx_end: 0,
            text: "test_text_2".to_string(),
        };

        let id_1 = chunk_1.to_uuid();
        let id_2_1 = chunk_2_1.to_uuid();
        let id_2_2 = chunk_2_2.to_uuid();

        client.store_chunk(chunk_1.clone()).await?;
        client.store_chunk(chunk_2_1.clone()).await?;
        client.store_chunk(chunk_2_2.clone()).await?;

        client.analytics().await?;

        // Test retrieval by ID
        let retrieved_chunk = client.get_chunk_by_id(id_1).await?;
        assert_eq!(
            chunk_1.league_year_team_idx,
            retrieved_chunk.league_year_team_idx
        );
        let retrieved_chunk = client.get_chunk_by_id(id_2_1).await?;
        assert_eq!(
            chunk_2_1.league_year_team_idx,
            retrieved_chunk.league_year_team_idx
        );
        let retrieved_chunk = client.get_chunk_by_id(id_2_2).await?;
        assert_eq!(
            chunk_2_2.league_year_team_idx,
            retrieved_chunk.league_year_team_idx
        );

        // Test retrieval by filter
        let get = async |filter: Filter| {
            client
                .search_chunks(
                    Some(dense_embedding.clone()),
                    Some(sparse_embedding.clone()),
                    5,
                    Some(filter),
                )
                .await
        };

        //// TEST 1
        // Test League filter
        let mut filter = Filter::default();
        filter.add_league(chunk_1.league);
        let chunks = get(filter).await?;
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].0.to_uuid(), id_1);

        // Test Year filter
        let mut filter = Filter::default();
        filter.add_year(chunk_1.year);
        let chunks = get(filter).await?;
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].0.to_uuid(), id_1);

        // Test Team filter
        let mut filter = Filter::default();
        filter.add_team(chunk_1.team);
        let chunks = get(filter).await?;
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].0.to_uuid(), id_1);

        // Test LeagueYearTeamIndex filter
        let mut filter = Filter::default();
        filter.add_league_year_team_index(chunk_1.league_year_team_idx);
        let chunks = get(filter).await?;
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].0.to_uuid(), id_1);

        //// TEST 2. 2_1 and 2_2 should both be retrieved (Except for lyti)
        // Test League filter
        let mut filter = Filter::default();
        filter.add_league(chunk_2_1.league);
        let chunks = get(filter).await?;
        assert_eq!(chunks.len(), 2);

        // Test Year filter
        let mut filter = Filter::default();
        filter.add_year(chunk_2_1.year);
        let chunks = get(filter).await?;
        assert_eq!(chunks.len(), 2);

        // Test Team filter
        let mut filter = Filter::default();
        filter.add_team(chunk_2_1.team);
        let chunks = get(filter).await?;
        assert_eq!(chunks.len(), 2);

        Ok(())
    }
}
