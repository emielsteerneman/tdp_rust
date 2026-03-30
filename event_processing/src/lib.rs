pub mod dispatcher;
pub mod listeners;

use async_trait::async_trait;
use serde::Serialize;
use thiserror::Error;

// ---------------------------------------------------------------------------
// EventSource
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EventSource {
    Web,
    Mcp,
}

impl EventSource {
    pub fn as_str(&self) -> &'static str {
        match self {
            EventSource::Web => "web",
            EventSource::Mcp => "mcp",
        }
    }
}

// ---------------------------------------------------------------------------
// Event structs
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize)]
pub struct SearchEvent {
    pub query: String,
    pub search_type: String,
    pub result_count: usize,
    pub league_filter: Option<String>,
    pub year_filter: Option<String>,
    pub team_filter: Option<String>,
    pub content_type_filter: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ListLeaguesEvent {
    pub result_count: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct ListYearsEvent {
    pub league: Option<String>,
    pub team: Option<String>,
    pub result_count: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct ListTeamsEvent {
    pub hint: Option<String>,
    pub result_count: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct ListPapersEvent {
    pub league: Option<String>,
    pub year: Option<String>,
    pub team: Option<String>,
    pub result_count: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct GetAbstractEvent {
    pub paper: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct GetPaperInfoEvent {
    pub paper: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct GetTableOfContentsEvent {
    pub paper: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct GetSectionEvent {
    pub paper: String,
    pub content_seq: u32,
    pub include_children: bool,
    pub items_returned: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct GetParagraphEvent {
    pub paper: String,
    pub content_seq: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct GetTableEvent {
    pub paper: String,
    pub content_seq: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct GetImageEvent {
    pub paper: String,
    pub content_seq: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct GetTdpContentsEvent {
    pub league: String,
    pub year: String,
    pub team: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct HttpRequestEvent {
    pub method: String,
    pub path: String,
    pub status: u16,
    pub duration_ms: u64,
    pub ip: Option<String>,
    pub user_agent: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct PaperOpenEvent {
    pub paper_id: String,
    pub referrer: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PdfOpenEvent {
    pub paper_id: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct SuggestionEvent {
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct GetTeamInfoEvent {
    pub team: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct UpdateTeamInfoEvent {
    pub team: String,
    pub entries: Vec<(String, String)>,
}

// ---------------------------------------------------------------------------
// Event enum
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Event {
    Search(SearchEvent),
    ListLeagues(ListLeaguesEvent),
    ListYears(ListYearsEvent),
    ListTeams(ListTeamsEvent),
    ListPapers(ListPapersEvent),
    GetAbstract(GetAbstractEvent),
    GetPaperInfo(GetPaperInfoEvent),
    GetTableOfContents(GetTableOfContentsEvent),
    GetSection(GetSectionEvent),
    GetParagraph(GetParagraphEvent),
    GetTable(GetTableEvent),
    GetImage(GetImageEvent),
    GetTdpContents(GetTdpContentsEvent),
    HttpRequest(HttpRequestEvent),
    PaperOpen(PaperOpenEvent),
    PdfOpen(PdfOpenEvent),
    Suggestion(SuggestionEvent),
    GetTeamInfo(GetTeamInfoEvent),
    UpdateTeamInfo(UpdateTeamInfoEvent),
}

impl Event {
    pub fn event_type(&self) -> &'static str {
        match self {
            Event::Search(_) => "search",
            Event::ListLeagues(_) => "list_leagues",
            Event::ListYears(_) => "list_years",
            Event::ListTeams(_) => "list_teams",
            Event::ListPapers(_) => "list_papers",
            Event::GetAbstract(_) => "get_abstract",
            Event::GetPaperInfo(_) => "get_paper_info",
            Event::GetTableOfContents(_) => "get_table_of_contents",
            Event::GetSection(_) => "get_section",
            Event::GetParagraph(_) => "get_paragraph",
            Event::GetTable(_) => "get_table",
            Event::GetImage(_) => "get_image",
            Event::GetTdpContents(_) => "get_tdp_contents",
            Event::HttpRequest(_) => "http_request",
            Event::PaperOpen(_) => "paper_open",
            Event::PdfOpen(_) => "pdf_open",
            Event::Suggestion(_) => "suggestion",
            Event::GetTeamInfo(_) => "get_team_info",
            Event::UpdateTeamInfo(_) => "update_team_info",
        }
    }
}

// ---------------------------------------------------------------------------
// EventListenerError
// ---------------------------------------------------------------------------

#[derive(Debug, Error)]
pub enum EventListenerError {
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("{0}")]
    Other(String),
}

// ---------------------------------------------------------------------------
// EventListener trait
// ---------------------------------------------------------------------------

#[async_trait]
pub trait EventListener: Send + Sync {
    async fn on_event(&self, source: &EventSource, event: &Event) -> Result<(), EventListenerError>;
    fn name(&self) -> &str;
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_type_strings() {
        let cases: Vec<(Event, &str)> = vec![
            (Event::Search(SearchEvent {
                query: "test".into(),
                search_type: "hybrid".into(),
                result_count: 5,
                league_filter: None,
                year_filter: None,
                team_filter: None,
                content_type_filter: None,
            }), "search"),
            (Event::ListLeagues(ListLeaguesEvent { result_count: 3 }), "list_leagues"),
            (Event::ListYears(ListYearsEvent { league: None, team: None, result_count: 2 }), "list_years"),
            (Event::ListTeams(ListTeamsEvent { hint: None, result_count: 1 }), "list_teams"),
            (Event::ListPapers(ListPapersEvent { league: None, year: None, team: None, result_count: 0 }), "list_papers"),
            (Event::GetAbstract(GetAbstractEvent { paper: "p".into() }), "get_abstract"),
            (Event::GetPaperInfo(GetPaperInfoEvent { paper: "p".into() }), "get_paper_info"),
            (Event::GetTableOfContents(GetTableOfContentsEvent { paper: "p".into() }), "get_table_of_contents"),
            (Event::GetSection(GetSectionEvent { paper: "p".into(), content_seq: 1, include_children: false, items_returned: 0 }), "get_section"),
            (Event::GetParagraph(GetParagraphEvent { paper: "p".into(), content_seq: 1 }), "get_paragraph"),
            (Event::GetTable(GetTableEvent { paper: "p".into(), content_seq: 1 }), "get_table"),
            (Event::GetImage(GetImageEvent { paper: "p".into(), content_seq: 1 }), "get_image"),
            (Event::GetTdpContents(GetTdpContentsEvent { league: "l".into(), year: "y".into(), team: "t".into() }), "get_tdp_contents"),
            (Event::HttpRequest(HttpRequestEvent { method: "GET".into(), path: "/".into(), status: 200, duration_ms: 10, ip: None, user_agent: "ua".into() }), "http_request"),
            (Event::PaperOpen(PaperOpenEvent { paper_id: "p".into(), referrer: None }), "paper_open"),
            (Event::PdfOpen(PdfOpenEvent { paper_id: "p".into() }), "pdf_open"),
            (Event::Suggestion(SuggestionEvent { message: "test suggestion".into() }), "suggestion"),
            (Event::GetTeamInfo(GetTeamInfoEvent { team: "t".into() }), "get_team_info"),
            (Event::UpdateTeamInfo(UpdateTeamInfoEvent { team: "t".into(), entries: vec![] }), "update_team_info"),
        ];

        for (event, expected) in cases {
            assert_eq!(event.event_type(), expected);
        }
    }

    #[test]
    fn test_event_source_as_str() {
        assert_eq!(EventSource::Web.as_str(), "web");
        assert_eq!(EventSource::Mcp.as_str(), "mcp");
    }

    #[test]
    fn test_event_serialization() {
        let event = Event::Search(SearchEvent {
            query: "robot navigation".into(),
            search_type: "hybrid".into(),
            result_count: 7,
            league_filter: Some("soccer_smallsize".into()),
            year_filter: None,
            team_filter: None,
            content_type_filter: None,
        });

        let json = serde_json::to_value(&event).unwrap();
        assert_eq!(json["type"], "search");
        assert_eq!(json["query"], "robot navigation");
        assert_eq!(json["result_count"], 7);
        assert_eq!(json["league_filter"], "soccer_smallsize");
        assert!(json["year_filter"].is_null());
    }

    #[test]
    fn test_event_serialization_all_variants() {
        // Ensure all variants serialize without error
        let events = vec![
            Event::ListLeagues(ListLeaguesEvent { result_count: 5 }),
            Event::GetAbstract(GetAbstractEvent { paper: "test_paper".into() }),
            Event::GetPaperInfo(GetPaperInfoEvent { paper: "test_paper".into() }),
            Event::PaperOpen(PaperOpenEvent { paper_id: "id".into(), referrer: Some("https://example.com".into()) }),
            Event::PdfOpen(PdfOpenEvent { paper_id: "test_paper".into() }),
            Event::Suggestion(SuggestionEvent { message: "improve search".into() }),
        ];

        for event in events {
            let json_str = serde_json::to_string(&event).unwrap();
            assert!(!json_str.is_empty());
            // Verify it round-trips through Value
            let val: serde_json::Value = serde_json::from_str(&json_str).unwrap();
            assert!(val.get("type").is_some());
        }
    }
}
