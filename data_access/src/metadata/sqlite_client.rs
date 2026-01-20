use serde::Deserialize;

use crate::metadata::MetadataClient;

pub struct SqliteClient {
    config: SqliteConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SqliteConfig {
    pub filename: String,
}

impl SqliteClient {
    pub fn new(config: SqliteConfig) -> Self {
        let client = Self { config }
        
        client.ensure_database();

        client
    }

    fn ensure_database(&self) {
        todo!()
    }
}

impl MetadataClient for SqliteClient {
    fn store_idf<'a>(
        &'a self,
        map: std::collections::HashMap<String, f32>,
    ) -> std::pin::Pin<Box<dyn Future<Output = Result<(), super::MetadataClientError>> + Send + 'a>>
    {
        todo!()
    }

    fn load_idf<'a>(
        &'a self,
        run: String,
    ) -> std::pin::Pin<
        Box<
            dyn Future<
                    Output = Result<
                        std::collections::HashMap<String, f32>,
                        super::MetadataClientError,
                    >,
                > + Send
                + 'a,
        >,
    > {
        todo!()
    }
}
