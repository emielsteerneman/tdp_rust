use data_access::{embed::EmbedClient, metadata::MetadataClient, vector::VectorClient};
use data_structures::IDF;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    #[allow(dead_code)]
    pub embed_client: Arc<dyn EmbedClient + Send + Sync>,
    pub vector_client: Arc<dyn VectorClient + Send + Sync>,
    #[allow(dead_code)]
    pub metadata_client: Arc<dyn MetadataClient + Send + Sync>,
    pub idf_map: Arc<IDF>,
}

impl AppState {
    pub fn new(
        embed_client: Arc<dyn EmbedClient + Send + Sync>,
        vector_client: Arc<dyn VectorClient + Send + Sync>,
        metadata_client: Arc<dyn MetadataClient + Send + Sync>,
        idf_map: IDF,
    ) -> Self {
        Self {
            embed_client,
            vector_client,
            metadata_client,
            idf_map: Arc::new(idf_map),
        }
    }
}
