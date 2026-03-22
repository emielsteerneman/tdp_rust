use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;

use crate::{Event, EventListener, EventListenerError, EventSource};

// ---------------------------------------------------------------------------
// Config
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, Clone)]
pub struct TelegramConfig {
    pub bot_token: String,
    pub chat_id: String,
}

// ---------------------------------------------------------------------------
// TelegramListener
// ---------------------------------------------------------------------------

pub struct TelegramListener {
    client: Client,
    bot_token: String,
    chat_id: String,
}

impl TelegramListener {
    pub fn new(config: &TelegramConfig) -> Self {
        Self {
            client: Client::new(),
            bot_token: config.bot_token.clone(),
            chat_id: config.chat_id.clone(),
        }
    }

    pub fn format_message(&self, source: &EventSource, event: &Event) -> Option<String> {
        let src = source.as_str();

        match event {
            Event::Search(e) => {
                let mut msg = format!(
                    "[{src}] Search: '{}' ({}, {} results)",
                    e.query, e.search_type, e.result_count
                );
                if let Some(ref f) = e.league_filter {
                    msg.push_str(&format!("\n  league: {f}"));
                }
                if let Some(ref f) = e.year_filter {
                    msg.push_str(&format!("\n  year: {f}"));
                }
                if let Some(ref f) = e.team_filter {
                    msg.push_str(&format!("\n  team: {f}"));
                }
                if let Some(ref f) = e.content_type_filter {
                    msg.push_str(&format!("\n  content_type: {f}"));
                }
                Some(msg)
            }
            Event::GetAbstract(e) => {
                Some(format!("[{src}] Get abstract: {}", e.paper))
            }
            Event::GetTableOfContents(e) => {
                Some(format!("[{src}] Get table of contents: {}", e.paper))
            }
            Event::GetSection(e) => {
                Some(format!(
                    "[{src}] Get section: {} seq={} children={} ({} items)",
                    e.paper, e.content_seq, e.include_children, e.items_returned
                ))
            }
            Event::GetParagraph(e) => {
                Some(format!("[{src}] Get paragraph: {} seq={}", e.paper, e.content_seq))
            }
            Event::GetTable(e) => {
                Some(format!("[{src}] Get table: {} seq={}", e.paper, e.content_seq))
            }
            Event::GetImage(e) => {
                Some(format!("[{src}] Get image: {} seq={}", e.paper, e.content_seq))
            }
            Event::GetTdpContents(e) => {
                Some(format!(
                    "[{src}] Get TDP contents: {} / {} / {}",
                    e.league, e.year, e.team
                ))
            }
            Event::PaperOpen(e) => {
                let referrer = e.referrer.as_deref().unwrap_or("direct");
                Some(format!("[{src}] Paper opened: {} (from {referrer})", e.paper_id))
            }
            Event::Suggestion(e) => Some(format!("[{src}] Suggestion: {}", e.message)),
            // Noisy events — skip
            Event::ListLeagues(_)
            | Event::ListYears(_)
            | Event::ListTeams(_)
            | Event::ListPapers(_)
            | Event::HttpRequest(_) => None,
        }
    }
}

// ---------------------------------------------------------------------------
// EventListener impl
// ---------------------------------------------------------------------------

#[async_trait]
impl EventListener for TelegramListener {
    async fn on_event(
        &self,
        source: &EventSource,
        event: &Event,
    ) -> Result<(), EventListenerError> {
        let message = match self.format_message(source, event) {
            Some(msg) => msg,
            None => return Ok(()),
        };

        let url = format!(
            "https://api.telegram.org/bot{}/sendMessage",
            self.bot_token
        );

        let response = self
            .client
            .post(&url)
            .json(&serde_json::json!({
                "chat_id": self.chat_id,
                "text": message,
            }))
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(EventListenerError::Other(format!(
                "Telegram API returned {status}: {body}"
            )));
        }

        Ok(())
    }

    fn name(&self) -> &str {
        "telegram"
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::*;

    fn make_listener() -> TelegramListener {
        TelegramListener::new(&TelegramConfig {
            bot_token: "test-token".into(),
            chat_id: "12345".into(),
        })
    }

    #[test]
    fn format_search_event() {
        let listener = make_listener();
        let event = Event::Search(SearchEvent {
            query: "robot navigation".into(),
            search_type: "hybrid".into(),
            result_count: 7,
            league_filter: Some("soccer_smallsize".into()),
            year_filter: None,
            team_filter: None,
            content_type_filter: None,
        });

        let msg = listener
            .format_message(&EventSource::Web, &event)
            .unwrap();
        assert!(msg.contains("[web]"));
        assert!(msg.contains("robot navigation"));
        assert!(msg.contains("hybrid"));
        assert!(msg.contains("7 results"));
        assert!(msg.contains("league: soccer_smallsize"));
        assert!(!msg.contains("year:"));
    }

    #[test]
    fn format_search_no_filters() {
        let listener = make_listener();
        let event = Event::Search(SearchEvent {
            query: "test".into(),
            search_type: "keyword".into(),
            result_count: 3,
            league_filter: None,
            year_filter: None,
            team_filter: None,
            content_type_filter: None,
        });

        let msg = listener
            .format_message(&EventSource::Mcp, &event)
            .unwrap();
        assert!(msg.contains("[mcp]"));
        assert!(msg.contains("'test'"));
        assert!(!msg.contains("\n"));
    }

    #[test]
    fn format_get_abstract() {
        let listener = make_listener();
        let event = Event::GetAbstract(GetAbstractEvent {
            paper: "soccer_smallsize__2024__RoboTeam__0".into(),
        });

        let msg = listener
            .format_message(&EventSource::Web, &event)
            .unwrap();
        assert!(msg.contains("Get abstract"));
        assert!(msg.contains("soccer_smallsize__2024__RoboTeam__0"));
    }

    #[test]
    fn format_get_section() {
        let listener = make_listener();
        let event = Event::GetSection(GetSectionEvent {
            paper: "paper1".into(),
            content_seq: 5,
            include_children: true,
            items_returned: 3,
        });

        let msg = listener
            .format_message(&EventSource::Mcp, &event)
            .unwrap();
        assert!(msg.contains("seq=5"));
        assert!(msg.contains("children=true"));
        assert!(msg.contains("3 items"));
    }

    #[test]
    fn format_paper_open() {
        let listener = make_listener();
        let event = Event::PaperOpen(PaperOpenEvent {
            paper_id: "my_paper".into(),
            referrer: Some("https://google.com".into()),
        });

        let msg = listener
            .format_message(&EventSource::Web, &event)
            .unwrap();
        assert!(msg.contains("Paper opened"));
        assert!(msg.contains("my_paper"));
        assert!(msg.contains("from https://google.com"));
    }

    #[test]
    fn format_paper_open_no_referrer() {
        let listener = make_listener();
        let event = Event::PaperOpen(PaperOpenEvent {
            paper_id: "my_paper".into(),
            referrer: None,
        });

        let msg = listener
            .format_message(&EventSource::Web, &event)
            .unwrap();
        assert!(msg.contains("from direct"));
    }

    #[test]
    fn format_get_tdp_contents() {
        let listener = make_listener();
        let event = Event::GetTdpContents(GetTdpContentsEvent {
            league: "soccer_smallsize".into(),
            year: "2024".into(),
            team: "RoboTeam".into(),
        });

        let msg = listener
            .format_message(&EventSource::Web, &event)
            .unwrap();
        assert!(msg.contains("Get TDP contents"));
        assert!(msg.contains("soccer_smallsize / 2024 / RoboTeam"));
    }

    #[test]
    fn skipped_events_return_none() {
        let listener = make_listener();

        let skipped = vec![
            Event::ListLeagues(ListLeaguesEvent { result_count: 5 }),
            Event::ListYears(ListYearsEvent {
                league: None,
                team: None,
                result_count: 3,
            }),
            Event::ListTeams(ListTeamsEvent {
                hint: None,
                result_count: 2,
            }),
            Event::ListPapers(ListPapersEvent {
                league: None,
                year: None,
                team: None,
                result_count: 10,
            }),
            Event::HttpRequest(HttpRequestEvent {
                method: "GET".into(),
                path: "/".into(),
                status: 200,
                duration_ms: 50,
                ip: None,
                user_agent: "test".into(),
            }),
        ];

        for event in skipped {
            assert!(
                listener
                    .format_message(&EventSource::Web, &event)
                    .is_none(),
                "expected None for {:?}",
                event.event_type()
            );
        }
    }
}
