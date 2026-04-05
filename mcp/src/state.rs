use data_access::metadata::MetadataClient;
use data_access::registry::RegistryClient;
use data_processing::search::Searcher;
use event_processing::dispatcher::EventDispatcher;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub metadata_client: Arc<dyn MetadataClient + Send + Sync>,
    pub searcher: Arc<Searcher>,
    pub dispatcher: Arc<EventDispatcher>,
    pub registry: Option<Arc<dyn RegistryClient + Send + Sync>>,
    pub website_url: Option<String>,
}

impl AppState {
    pub fn new(
        metadata_client: Arc<dyn MetadataClient + Send + Sync>,
        searcher: Arc<Searcher>,
        dispatcher: Arc<EventDispatcher>,
        registry: Option<Arc<dyn RegistryClient + Send + Sync>>,
        website_url: Option<String>,
    ) -> Self {
        Self {
            metadata_client,
            searcher,
            dispatcher,
            registry,
            website_url,
        }
    }
}
