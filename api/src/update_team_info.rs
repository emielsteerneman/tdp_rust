use std::sync::Arc;

use data_access::teams::TeamRegistryClient;
use data_structures::file::TeamName;
use event_processing::dispatcher::EventDispatcher;
use event_processing::{Event, EventSource, UpdateTeamInfoEvent};
use serde::Deserialize;

use crate::error::ApiError;

#[derive(Debug, Deserialize)]
pub struct UpdateTeamInfoArgs {
    pub code: String,
    pub entries: Vec<UpdateEntry>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateEntry {
    pub key: String,
    pub value: String,
}

pub async fn update_team_info(
    team_registry: Arc<dyn TeamRegistryClient + Send + Sync>,
    team: &str,
    args: UpdateTeamInfoArgs,
    dispatcher: &EventDispatcher,
    source: EventSource,
) -> Result<String, ApiError> {
    let team_name = TeamName::new(team.trim());

    // Validate entries
    if args.entries.len() > 50 {
        return Err(ApiError::Argument(
            "entries".to_string(),
            "Maximum 50 entries per team".to_string(),
        ));
    }

    for entry in &args.entries {
        if entry.key.len() > 64 {
            return Err(ApiError::Argument(
                "key".to_string(),
                format!("Key '{}' exceeds 64 character limit", entry.key),
            ));
        }
        if !entry.key.chars().all(|c| c.is_alphanumeric() || c == '_') {
            return Err(ApiError::Argument(
                "key".to_string(),
                format!(
                    "Key '{}' contains invalid characters (alphanumeric and underscores only)",
                    entry.key
                ),
            ));
        }
        if entry.value.len() > 2048 {
            return Err(ApiError::Argument(
                "value".to_string(),
                format!(
                    "Value for key '{}' exceeds 2048 character limit",
                    entry.key
                ),
            ));
        }
    }

    // Verify auth
    let authorized = team_registry
        .verify_code(&team_name.name, &args.code)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    if !authorized {
        return Err(ApiError::Forbidden("Invalid team code".to_string()));
    }

    let entries_tuples: Vec<(String, String)> = args
        .entries
        .into_iter()
        .map(|e| (e.key, e.value))
        .collect();

    let entries_for_event = entries_tuples.clone();

    team_registry
        .set_team_metadata(&team_name.name, entries_tuples)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    dispatcher.dispatch(
        source,
        Event::UpdateTeamInfo(UpdateTeamInfoEvent {
            team: team_name.name,
            entries: entries_for_event,
        }),
    );

    Ok("Team metadata updated successfully".to_string())
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use data_access::teams::{TeamRegistryClient, TeamsSqliteClient, TeamsSqliteConfig};
    use event_processing::dispatcher::EventDispatcher;
    use event_processing::EventSource;

    use super::*;

    fn make_registry(master_pw: &str) -> Arc<dyn TeamRegistryClient + Send + Sync> {
        Arc::new(TeamsSqliteClient::new(TeamsSqliteConfig {
            filename: ":memory:".to_string(),
            master_password: Some(master_pw.to_string()),
        }))
    }

    fn make_dispatcher() -> EventDispatcher {
        EventDispatcher::new()
    }

    fn make_entries(n: usize) -> Vec<UpdateEntry> {
        (0..n)
            .map(|i| UpdateEntry {
                key: format!("key_{i}"),
                value: format!("value_{i}"),
            })
            .collect()
    }

    #[tokio::test]
    async fn test_validation_too_many_entries() {
        let registry = make_registry("master");
        let dispatcher = make_dispatcher();

        let args = UpdateTeamInfoArgs {
            code: "any".to_string(),
            entries: make_entries(51),
        };

        let result = update_team_info(registry, "TeamA", args, &dispatcher, EventSource::Web).await;

        match result {
            Err(ApiError::Argument(field, _)) => assert_eq!(field, "entries"),
            other => panic!("Expected Argument error on 'entries', got {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_validation_key_too_long() {
        let registry = make_registry("master");
        let dispatcher = make_dispatcher();

        let long_key = "a".repeat(65);
        let args = UpdateTeamInfoArgs {
            code: "any".to_string(),
            entries: vec![UpdateEntry {
                key: long_key,
                value: "value".to_string(),
            }],
        };

        let result = update_team_info(registry, "TeamA", args, &dispatcher, EventSource::Web).await;

        match result {
            Err(ApiError::Argument(field, _)) => assert_eq!(field, "key"),
            other => panic!("Expected Argument error on 'key', got {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_validation_key_invalid_chars() {
        let registry = make_registry("master");
        let dispatcher = make_dispatcher();

        let args = UpdateTeamInfoArgs {
            code: "any".to_string(),
            entries: vec![UpdateEntry {
                key: "invalid-key".to_string(),
                value: "value".to_string(),
            }],
        };

        let result = update_team_info(registry, "TeamA", args, &dispatcher, EventSource::Web).await;

        match result {
            Err(ApiError::Argument(field, _)) => assert_eq!(field, "key"),
            other => panic!("Expected Argument error on 'key', got {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_validation_value_too_long() {
        let registry = make_registry("master");
        let dispatcher = make_dispatcher();

        let long_value = "a".repeat(2049);
        let args = UpdateTeamInfoArgs {
            code: "any".to_string(),
            entries: vec![UpdateEntry {
                key: "valid_key".to_string(),
                value: long_value,
            }],
        };

        let result = update_team_info(registry, "TeamA", args, &dispatcher, EventSource::Web).await;

        match result {
            Err(ApiError::Argument(field, _)) => assert_eq!(field, "value"),
            other => panic!("Expected Argument error on 'value', got {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_auth_failure() {
        let registry = make_registry("master");
        let dispatcher = make_dispatcher();

        let args = UpdateTeamInfoArgs {
            code: "wrong_code".to_string(),
            entries: vec![UpdateEntry {
                key: "valid_key".to_string(),
                value: "value".to_string(),
            }],
        };

        let result = update_team_info(registry, "TeamA", args, &dispatcher, EventSource::Web).await;

        match result {
            Err(ApiError::Forbidden(_)) => {}
            other => panic!("Expected Forbidden error, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_successful_update_with_master_password() {
        let master_pw = "super_secret_master";
        let registry = make_registry(master_pw);
        let dispatcher = make_dispatcher();

        let args = UpdateTeamInfoArgs {
            code: master_pw.to_string(),
            entries: vec![
                UpdateEntry {
                    key: "github".to_string(),
                    value: "https://github.com/team".to_string(),
                },
                UpdateEntry {
                    key: "website".to_string(),
                    value: "https://team.example.com".to_string(),
                },
            ],
        };

        let result =
            update_team_info(registry.clone(), "TeamA", args, &dispatcher, EventSource::Web).await;

        assert!(result.is_ok(), "Expected Ok, got {:?}", result);

        let entries = registry.get_team_metadata("TeamA").await.unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].key, "github");
        assert_eq!(entries[1].key, "website");
    }
}
