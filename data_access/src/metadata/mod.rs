use std::pin::Pin;
mod sqlite_client;
use data_structures::{
    IDF,
    content::{ContentItem, MarkdownTDP, TocEntry},
    file::{League, TDPName, TeamName},
};
use mockall::automock;
pub use sqlite_client::{SqliteClient, SqliteConfig};

#[derive(thiserror::Error, Debug)]
pub enum MetadataClientError {
    #[error("Internal error: {0}")]
    Internal(String),
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("No vectors present")]
    Empty,
    #[error("Field missing: {0}")]
    FieldMissing(String),
    #[error("Invalid vector dimension: {0}")]
    InvalidVectorDimension(String),
}
#[automock]
pub trait MetadataClient: Send + Sync {
    fn store_idf<'a>(
        &'a self,
        idf: IDF,
    ) -> Pin<Box<dyn Future<Output = Result<(), MetadataClientError>> + Send + 'a>>;

    fn load_idf<'a>(
        &'a self,
    ) -> Pin<Box<dyn Future<Output = Result<IDF, MetadataClientError>> + Send + 'a>>;

    fn load_tdps<'a>(
        &'a self,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<TDPName>, MetadataClientError>> + Send + 'a>>;

    fn load_teams<'a>(
        &'a self,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<TeamName>, MetadataClientError>> + Send + 'a>>;

    fn load_leagues<'a>(
        &'a self,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<League>, MetadataClientError>> + Send + 'a>>;

    fn get_tdp_markdown<'a>(
        &'a self,
        tdp_name: TDPName,
    ) -> Pin<Box<dyn Future<Output = Result<String, MetadataClientError>> + Send + 'a>>;

    fn print_analytics<'a>(
        &'a self,
    ) -> Pin<Box<dyn Future<Output = Result<(), MetadataClientError>> + Send + 'a>>;

    fn store_paper<'a>(
        &'a self,
        tdp: MarkdownTDP,
    ) -> Pin<Box<dyn Future<Output = Result<(), MetadataClientError>> + Send + 'a>>;

    fn load_toc<'a>(
        &'a self,
        lyti: String,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<TocEntry>, MetadataClientError>> + Send + 'a>>;

    fn load_content_item<'a>(
        &'a self,
        lyti: String,
        content_seq: u32,
    ) -> Pin<Box<dyn Future<Output = Result<ContentItem, MetadataClientError>> + Send + 'a>>;

    fn load_paper_abstract<'a>(
        &'a self,
        lyti: String,
    ) -> Pin<Box<dyn Future<Output = Result<String, MetadataClientError>> + Send + 'a>>;

}
