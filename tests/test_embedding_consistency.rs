use std::error::Error;

use data_access::{
    embed::{EmbedClient, FastembedClient},
    vector::QdrantClient,
};
use ndarray::Array1;

#[tokio::test]
async fn test_embedding_consistency() -> Result<(), Box<dyn Error>> {
    let _vector_client = QdrantClient::new()?;
    let _embed_client = FastembedClient::new()?;

    let (text, vector) = _vector_client.get_first().await?;

    let vector2 = _embed_client.embed_string(&text)?;

    println!("\n\n{}", text);
    println!("\n\n{:?}", vector);
    println!("\n\n{:?}", vector2);

    let v1 = Array1::from(vector);
    let v2 = Array1::from(vector2);

    let dist = v1.cos

    Ok(())
}
