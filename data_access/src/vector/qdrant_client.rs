use crate::vector::VectorClient;
use qdrant_client::{
    Qdrant, QdrantError,
    qdrant::{
        CountPointsBuilder, PointId, ScrollPointsBuilder, point_id::PointIdOptions,
        vectors_output::VectorsOptions,
    },
};

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

    pub async fn get_first(self) -> Result<Vec<f32>, QdrantClientError> {
        let mut next_offset = PointIdOptions::Num(0);

        let builder = ScrollPointsBuilder::new("paragraph")
            .with_payload(true)
            .with_vectors(true)
            .offset(next_offset);

        let scroll = self.client.scroll(builder.clone()).await.unwrap();

        let Some(first) = scroll.result.into_iter().next() else {
            return Err(QdrantClientError::Other("Weird".to_string()));
        };

        let Some(vec) = first.vectors else {
            return Err(QdrantClientError::Other("Weird".to_string()));
        };

        let dense = match vec.vectors_options {
            Some(VectorsOptions::Vectors(v)) => v.vectors.get("dense").cloned(),
            _ => None,
        };

        let Some(dense) = dense else {
            return Err(QdrantClientError::Other(
                "Dense vector not found".to_string(),
            ));
        };

        Ok(dense.data)
    }

    pub async fn analytics(self) -> Result<(), QdrantClientError> {
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

impl VectorClient for QdrantClient {}

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
