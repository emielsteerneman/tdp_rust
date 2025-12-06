use crate::vector::{VectorClient, VectorClientError};
use async_trait::async_trait;
use data_structures::{intermediate::Chunk, mock::MockVector};
use qdrant_client::{
    Qdrant, QdrantError,
    qdrant::{
        CountPointsBuilder, PointId, PointStruct, RetrievedPoint, ScrollPointsBuilder,
        point_id::PointIdOptions, vectors_output::VectorsOptions,
    },
};
use uuid::Uuid;

impl From<QdrantError> for VectorClientError {
    fn from(value: QdrantError) -> Self {
        VectorClientError::Internal(value.to_string())
    }
}

pub struct QdrantClient {
    client: Qdrant,
}

#[derive(thiserror::Error, Debug)]
pub enum QdrantClientError {
    #[error("Internal Qdrant error: {0}")]
    Internal(#[from] QdrantError),
    #[error("Internal Qdrant error: {0}")]
    Other(String),
}

impl QdrantClient {
    pub fn new() -> Result<Self, QdrantClientError> {
        let client = Qdrant::from_url("http://localhost:6334").build()?;

        Ok(Self { client })
    }

    pub async fn analytics(&self) -> Result<(), QdrantClientError> {
        let collections_list = self.client.list_collections().await.unwrap();

        for collection in collections_list.collections {
            let count = self
                .client
                .count(CountPointsBuilder::new(&collection.name).exact(true))
                .await?;

            println!(
                "Collection: {}, count: {}",
                collection.name,
                count.result.unwrap().count
            );
        }

        let mut next_offset = PointIdOptions::Num(0);

        loop {
            println!(".");
            let builder = ScrollPointsBuilder::new("paragraph")
                .with_payload(true)
                .with_vectors(true)
                .offset(next_offset);

            let scroll = self.client.scroll(builder.clone()).await.unwrap();

            for result in scroll.result {
                let my_string = match result.id.unwrap().point_id_options.unwrap() {
                    PointIdOptions::Num(n) => n.to_string(),
                    PointIdOptions::Uuid(u) => u.to_string(),
                };
                println!("\n  {my_string}");

                let payload = result.payload;
                for (s, v) in payload {
                    println!(
                        "    {s:>15} {}",
                        v.to_string().chars().take(100).collect::<String>()
                    );
                }

                let vector = result.vectors.unwrap().vectors_options.unwrap();
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

            match scroll.next_page_offset {
                Some(PointId {
                    point_id_options: Some(offset),
                }) => next_offset = offset,
                _ => break,
            }
        }

        Ok(())
    }
}

#[async_trait]
impl VectorClient for QdrantClient {
    const COLLECTION_NAME_PARAGRAPH: &'static str = "paragraph";
    const COLLECTION_NAME_MOCK: &'static str = "mock";

    async fn get_first_paragraph(&self) -> Result<(String, Vec<f32>), VectorClientError> {
        todo!()
    }

    async fn get_first_mock(&self) -> Result<MockVector, VectorClientError> {
        let next_offset = PointIdOptions::Num(0);

        let builder = ScrollPointsBuilder::new(Self::COLLECTION_NAME_MOCK)
            .with_payload(true)
            .with_vectors(true)
            .offset(next_offset);

        let scroll = self.client.scroll(builder.clone()).await.unwrap();

        let Some(first) = scroll.result.into_iter().next() else {
            return Err(VectorClientError::Empty);
        };

        first.into_mock()
    }

    async fn get_all_mock(&self) -> Result<Vec<MockVector>, VectorClientError> {
        let count = self
            .client
            .count(CountPointsBuilder::new(Self::COLLECTION_NAME_MOCK).exact(true))
            .await?;

        let count = count.result.unwrap().count;

        println!(
            "Collection: {}, count: {}",
            Self::COLLECTION_NAME_MOCK,
            count
        );

        if count == 0 {
            return Err(VectorClientError::Empty);
        }

        let builder = ScrollPointsBuilder::new(Self::COLLECTION_NAME_MOCK)
            .with_payload(true)
            .with_vectors(true)
            .limit(count as u32);

        let scroll = match self.client.scroll(builder.clone()).await {
            Ok(s) => s,
            Err(e) => return Err(e.into()),
        };

        let mut output = Vec::<MockVector>::new();

        for result in scroll.result {
            let my_string = match result.id.clone().unwrap().point_id_options.unwrap() {
                PointIdOptions::Num(n) => n.to_string(),
                PointIdOptions::Uuid(u) => u.to_string(),
            };
            println!("\n  id={my_string}");

            output.push(result.into_mock()?);
        }

        Ok(output)
    }

    /*
    async fn store_chunk(&self, chunk: Chunk) -> Result<(), VectorClientError> {
        let ps = PointStruct {
            id: todo!(),
            payload: todo!(),
            vectors: todo!(),
        };
        Ok(())
    }

    async fn get_first_chunk(&self) -> Result<Chunk, VectorClientError> {
        Ok(Chunk {
            sentences: todo!(),
            start: todo!(),
            end: todo!(),
            text: todo!(),
            metadata: todo!(),
            embedding: todo!(),
        })
    }

    async fn get_all_chunks(&self) -> Result<Vec<Chunk>, VectorClientError> {
        Ok(vec![Chunk {
            sentences: todo!(),
            start: todo!(),
            end: todo!(),
            text: todo!(),
            metadata: todo!(),
            embedding: todo!(),
        }])
    }*/
}

pub trait IntoMockVector {
    fn into_mock(self) -> Result<MockVector, VectorClientError>;
}

impl IntoMockVector for RetrievedPoint {
    fn into_mock(self) -> Result<MockVector, VectorClientError> {
        // Retrieve text from payload
        let Some(text) = get_string_from_point(&self, "text") else {
            return Err(VectorClientError::FieldMissing("text".to_string()));
        };

        // Retrieve dense vector
        let Some(vector) = get_dense_vector_from_point(self) else {
            return Err(VectorClientError::FieldMissing("dense".to_string()));
        };

        Ok(MockVector { text, vector })
    }
}

pub fn get_string_from_point(p: &RetrievedPoint, key: &str) -> Option<String> {
    p.payload
        .get(key)
        .and_then(|v| v.as_str())
        .map(|v| v.to_string())
}

pub fn get_dense_vector_from_point(p: RetrievedPoint) -> Option<Vec<f32>> {
    let v = match p.vectors?.vectors_options? {
        VectorsOptions::Vectors(v) => v,
        _ => return None,
    };
    let vector = v.vectors.get("dense")?.clone();
    Some(vector.data)
}

pub fn get_hash(s: &str) -> Uuid {
    static ZERO_NAMESPACE: Uuid = Uuid::from_bytes([0u8; 16]);
    Uuid::new_v5(&ZERO_NAMESPACE, s.as_bytes())
}

#[cfg(test)]
mod tests {

    use crate::vector::QdrantClient;

    #[tokio::test]
    async fn test_analyze() -> Result<(), anyhow::Error> {
        let client = QdrantClient::new();
        assert!(client.is_ok());

        let client = client.unwrap();

        client.analytics().await?;

        Ok(())
    }
}
