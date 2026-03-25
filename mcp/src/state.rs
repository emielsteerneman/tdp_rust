use data_access::metadata::MetadataClient;
use data_access::teams::TeamRegistryClient;
use data_processing::search::Searcher;
use event_processing::dispatcher::EventDispatcher;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub metadata_client: Arc<dyn MetadataClient + Send + Sync>,
    pub searcher: Arc<Searcher>,
    pub dispatcher: Arc<EventDispatcher>,
    pub team_registry: Option<Arc<dyn TeamRegistryClient + Send + Sync>>,
}

impl AppState {
    pub fn new(
        metadata_client: Arc<dyn MetadataClient + Send + Sync>,
        searcher: Arc<Searcher>,
        dispatcher: Arc<EventDispatcher>,
        team_registry: Option<Arc<dyn TeamRegistryClient + Send + Sync>>,
    ) -> Self {
        Self {
            metadata_client,
            searcher,
            dispatcher,
            team_registry,
        }
    }
}
