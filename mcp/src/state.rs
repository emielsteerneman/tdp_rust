use data_access::metadata::MetadataClient;
use data_processing::search::Searcher;
use event_processing::dispatcher::EventDispatcher;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub metadata_client: Arc<dyn MetadataClient + Send + Sync>,
    pub searcher: Arc<Searcher>,
    pub dispatcher: Arc<EventDispatcher>,
}

impl AppState {
    pub fn new(
        metadata_client: Arc<dyn MetadataClient + Send + Sync>,
        searcher: Arc<Searcher>,
        dispatcher: Arc<EventDispatcher>,
    ) -> Self {
        Self {
            metadata_client,
            searcher,
            dispatcher,
        }
    }
}
