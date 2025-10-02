// #[cfg(test)]
use std::collections::HashMap;

use crate::vector::{VectorClient, VectorClientError};
use anyhow::bail;
use async_trait::async_trait;
use data_structures::mock::MockVector;
use qdrant_client::{
    Qdrant, QdrantError,
    qdrant::{
        CountPointsBuilder, PointId, ScrollPointsBuilder, point_id::PointIdOptions,
        vectors_output::VectorsOptions,
    },
};

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
                            println!("Named  {name} {:?}", v2.data);
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
    const COLLECTION_NAME_PARAGRAPHS: &'static str = "paragraph";

    async fn get_first_paragraph(&self) -> Result<(String, Vec<f32>), VectorClientError> {
        todo!()
    }

    async fn get_first_mock(&self) -> Result<MockVector, VectorClientError> {
        let next_offset = PointIdOptions::Num(0);

        let builder = ScrollPointsBuilder::new("paragraph")
            .with_payload(true)
            .with_vectors(true)
            .offset(next_offset);

        let scroll = self.client.scroll(builder.clone()).await.unwrap();

        let Some(first) = scroll.result.into_iter().next() else {
            return Err(VectorClientError::Empty);
        };

        // Retrieve text from payload
        let text = match first.payload.get("text") {
            Some(v) => v.to_string(),
            None => return Err(VectorClientError::FieldMissing("text".to_string())),
        };

        // Retrieve dense vector
        let Some(vec) = first.vectors else {
            return Err(VectorClientError::FieldMissing("vectors".to_string()));
        };

        let vector = match vec.vectors_options {
            Some(VectorsOptions::Vectors(v)) => v.vectors.get("dense").cloned(),
            _ => return Err(VectorClientError::FieldMissing("named vectors".to_string())),
        };

        let Some(vector) = vector else {
            return Err(VectorClientError::FieldMissing("dense".to_string()));
        };

        // Return
        Ok(MockVector {
            text,
            vector: vector.data,
        })
    }

    async fn get_all_mock(&self) -> Result<Vec<MockVector>, VectorClientError> {
        let count = self
            .client
            .count(CountPointsBuilder::new("mock_data").exact(true))
            .await?;

        let count = count.result.unwrap().count;

        println!("Collection: mock_data, count: {}", count);

        if count == 0 {
            return Err(VectorClientError::Empty);
        }

        let builder = ScrollPointsBuilder::new("mock_data")
            .with_payload(true)
            .with_vectors(true)
            .limit(count as u32);

        let scroll = match self.client.scroll(builder.clone()).await {
            Ok(s) => s,
            Err(e) => return Err(e.into()),
        };

        let mut output = Vec::<MockVector>::new();

        for result in scroll.result {
            let my_string = match result.id.unwrap().point_id_options.unwrap() {
                PointIdOptions::Num(n) => n.to_string(),
                PointIdOptions::Uuid(u) => u.to_string(),
            };
            println!("\n  id={my_string}");

            // Retrieve text from payload
            let text = match result.payload.get("text") {
                Some(v) => v.to_string(),
                None => return Err(VectorClientError::FieldMissing("text".to_string())),
            };

            // Retrieve dense vector
            let Some(vec) = result.vectors else {
                return Err(VectorClientError::FieldMissing("vectors".to_string()));
            };

            let vector = match vec.vectors_options {
                Some(VectorsOptions::Vectors(v)) => v.vectors.get("dense").cloned(),
                _ => return Err(VectorClientError::FieldMissing("named vectors".to_string())),
            };

            let Some(vector) = vector else {
                return Err(VectorClientError::FieldMissing("dense".to_string()));
            };

            output.push(MockVector {
                text,
                vector: vector.data,
            });
        }

        Ok(output)
    }
}

#[cfg(test)]
mod tests {

    use crate::vector::QdrantClient;

    #[tokio::test]
    async fn test_initialization() -> Result<(), anyhow::Error> {
        let client = QdrantClient::new();
        assert!(client.is_ok());

        let client = client.unwrap();

        client.analytics().await?;

        Ok(())
    }
}
