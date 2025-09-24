use qdrant_client::{
    Qdrant, QdrantError,
    qdrant::{CountPointsBuilder, PointId, ScrollPointsBuilder, point_id::PointIdOptions},
};

use crate::vector::VectorClient;

pub struct QdrantClient {
    client: Qdrant,
}

impl QdrantClient {
    pub fn new() -> Result<Self, QdrantError> {
        let client = Qdrant::from_url("http://localhost:6334").build()?;

        Ok(Self { client })
    }

    pub async fn analytics(self) -> Result<(), QdrantError> {
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
                println!("  {my_string}");

                let payload = result.payload;
                for (s, v) in payload {
                    println!(
                        "    {s:>30} {}",
                        v.to_string().chars().take(100).collect::<String>()
                    );
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
