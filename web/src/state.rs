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
    pub tdps_markdown_root: String,
    pub tdps_pdf_root: String,
    pub registry: Option<Arc<dyn RegistryClient + Send + Sync>>,
}

impl AppState {
    pub fn new(
        metadata_client: Arc<dyn MetadataClient + Send + Sync>,
        searcher: Arc<Searcher>,
        dispatcher: Arc<EventDispatcher>,
        tdps_markdown_root: String,
        tdps_pdf_root: String,
        registry: Option<Arc<dyn RegistryClient + Send + Sync>>,
    ) -> Self {
        Self {
            metadata_client,
            searcher,
            dispatcher,
            tdps_markdown_root,
            tdps_pdf_root,
            registry,
        }
    }
}
