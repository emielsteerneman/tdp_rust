use data_processing::search::Searcher;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub searcher: Arc<Searcher>,
}

impl AppState {
    pub fn new(searcher: Arc<Searcher>) -> Self {
        Self { searcher }
    }
}
