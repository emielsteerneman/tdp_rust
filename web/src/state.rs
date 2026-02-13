use data_access::activity::ActivityClient;
use data_access::metadata::MetadataClient;
use data_processing::search::Searcher;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub metadata_client: Arc<dyn MetadataClient + Send + Sync>,
    pub searcher: Arc<Searcher>,
    pub activity_client: Option<Arc<dyn ActivityClient + Send + Sync>>,
}

impl AppState {
    pub fn new(
        metadata_client: Arc<dyn MetadataClient + Send + Sync>,
        searcher: Arc<Searcher>,
        activity_client: Option<Arc<dyn ActivityClient + Send + Sync>>,
    ) -> Self {
        Self {
            metadata_client,
            searcher,
            activity_client,
        }
    }
}
