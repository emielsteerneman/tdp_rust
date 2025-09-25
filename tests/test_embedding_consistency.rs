use data_access::{embed::FastembedClient, vector::QdrantClient};

#[test]
fn test_embedding_consistency() {
    let _vector_client = QdrantClient::new();
    let _embed_client = FastembedClient::new();
}
