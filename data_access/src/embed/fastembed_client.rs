use crate::embed::EmbedClient;

pub struct FastembedClient {}

// https://crates.io/crates/fastembed
impl FastembedClient {
    pub fn new() -> Self {
        Self {}
    }
}

impl EmbedClient for FastembedClient {}
