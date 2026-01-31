use std::{collections::HashMap, pin::Pin};

mod sqlite_client;
use data_structures::{IDF, file::TDPName, paper::TDP};
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

pub trait MetadataClient {
    fn store_idf<'a>(
        &'a self,
        map: IDF,
    ) -> Pin<Box<dyn Future<Output = Result<(), MetadataClientError>> + Send + 'a>>;

    fn load_idf<'a>(
        &'a self,
    ) -> Pin<
        Box<
            dyn Future<Output = Result<HashMap<String, (u32, f32)>, MetadataClientError>>
                + Send
                + 'a,
        >,
    >;

    fn store_tdps<'a>(
        &'a self,
        tdps: Vec<TDP>,
    ) -> Pin<Box<dyn Future<Output = Result<(), MetadataClientError>> + Send + 'a>>;

    fn load_tdps<'a>(
        &'a self,
        tdps: Vec<TDP>,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<TDPName>, MetadataClientError>> + Send + 'a>>;

    /* What else do I want to store here?
        The basic League Year Team
        * Leagues. Hardcode the leagues, together with their descriptions? Or at least hardcode the description to give to the AI.
        * All years for which papers are available
        * Teams, RoboTeam_Twente. Or maybe as RoboTeam Twente? Since that matches better with what is in papers. If the AI
        search for RoboTeam_Twente it won't find anything.

        Maybe number of papers, number of chunks? Papers per league?
        Information: Group by league, year, team? Not needed for now

        Maybe at some point, specific team data? Their github repos, websites, other stuff

        // table team, year, league, paper. Then index on all, simply retrieve the index?

    */
}
