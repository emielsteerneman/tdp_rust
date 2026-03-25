use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use hmac::{Hmac, Mac};
use rusqlite::{Connection, params};
use serde::Deserialize;
use sha2::Sha256;
use subtle::ConstantTimeEq;
use crate::teams::{TeamMetadataEntry, TeamRegistryClient, TeamRegistryError};

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, Deserialize, Clone)]
pub struct TeamsSqliteConfig {
    pub filename: String,
    pub master_password: Option<String>,
}

pub struct TeamsSqliteClient {
    conn: Arc<Mutex<Connection>>,
    salt: String,
}

impl TeamsSqliteClient {
    pub fn new(config: TeamsSqliteConfig) -> Self {
        let conn = Connection::open(&config.filename)
            .expect("Failed to open SQLite database");

        conn.execute_batch("PRAGMA journal_mode=WAL;")
            .expect("Failed to enable WAL mode");

        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS config (
                key   TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS team_metadata (
                id         INTEGER PRIMARY KEY AUTOINCREMENT,
                team_name  TEXT NOT NULL,
                key        TEXT NOT NULL,
                value      TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_team_metadata_team_name
                ON team_metadata(team_name);",
        )
        .expect("Failed to create tables");

        // Load or generate salt
        let salt: String = {
            let existing: Option<String> = conn
                .query_row(
                    "SELECT value FROM config WHERE key = 'salt'",
                    [],
                    |row| row.get(0),
                )
                .ok();

            if let Some(s) = existing {
                s
            } else {
                let bytes: [u8; 32] = rand::random();
                let s = hex::encode(bytes);
                conn.execute(
                    "INSERT INTO config (key, value) VALUES ('salt', ?1)",
                    params![s],
                )
                .expect("Failed to store salt");
                s
            }
        };

        // If master_password provided and no master_hash stored yet, store it
        if let Some(ref pw) = config.master_password {
            let has_hash: bool = conn
                .query_row(
                    "SELECT COUNT(*) FROM config WHERE key = 'master_hash'",
                    [],
                    |row| row.get::<_, i64>(0),
                )
                .unwrap_or(0)
                > 0;

            if !has_hash {
                let hash = Self::compute_hmac(&salt, pw);
                conn.execute(
                    "INSERT INTO config (key, value) VALUES ('master_hash', ?1)",
                    params![hash],
                )
                .expect("Failed to store master_hash");
            }
        }

        TeamsSqliteClient {
            conn: Arc::new(Mutex::new(conn)),
            salt,
        }
    }

    /// HMAC-SHA256(salt, message) returned as a lowercase hex string.
    pub fn compute_hmac(salt: &str, message: &str) -> String {
        let mut mac = HmacSha256::new_from_slice(salt.as_bytes())
            .expect("HMAC accepts any key size");
        mac.update(message.as_bytes());
        hex::encode(mac.finalize().into_bytes())
    }

    /// First 16 characters of HMAC(salt, team_name).
    pub fn compute_team_code(salt: &str, team_name: &str) -> String {
        Self::compute_hmac(salt, team_name)[..16].to_string()
    }

    /// Constant-time equality check.
    pub fn constant_time_eq(a: &str, b: &str) -> bool {
        a.as_bytes().ct_eq(b.as_bytes()).into()
    }
}

impl TeamRegistryClient for TeamsSqliteClient {
    fn get_team_metadata<'a>(
        &'a self,
        team_name: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<TeamMetadataEntry>, TeamRegistryError>> + Send + 'a>>
    {
        Box::pin(async move {
            let conn = self.conn.lock().map_err(|e| {
                TeamRegistryError::Internal(format!("Lock poisoned: {e}"))
            })?;

            let mut stmt = conn
                .prepare(
                    "SELECT key, value, updated_at FROM team_metadata \
                     WHERE team_name = ?1 ORDER BY id",
                )
                .map_err(|e| TeamRegistryError::Internal(e.to_string()))?;

            let entries: Vec<TeamMetadataEntry> = stmt
                .query_map(params![team_name], |row| {
                    Ok(TeamMetadataEntry {
                        key: row.get(0)?,
                        value: row.get(1)?,
                        updated_at: row.get(2)?,
                    })
                })
                .map_err(|e| TeamRegistryError::Internal(e.to_string()))?
                .collect::<Result<_, _>>()
                .map_err(|e| TeamRegistryError::Internal(e.to_string()))?;

            Ok(entries)
        })
    }

    fn set_team_metadata<'a>(
        &'a self,
        team_name: &'a str,
        entries: Vec<(String, String)>,
    ) -> Pin<Box<dyn Future<Output = Result<(), TeamRegistryError>> + Send + 'a>>
    {
        Box::pin(async move {
            let conn = self.conn.lock().map_err(|e| {
                TeamRegistryError::Internal(format!("Lock poisoned: {e}"))
            })?;

            let tx = conn.unchecked_transaction().map_err(|e| {
                TeamRegistryError::Internal(e.to_string())
            })?;

            tx.execute(
                "DELETE FROM team_metadata WHERE team_name = ?1",
                params![team_name],
            )
            .map_err(|e| TeamRegistryError::Internal(e.to_string()))?;

            let now = chrono::Utc::now().to_rfc3339();

            for (key, value) in &entries {
                tx.execute(
                    "INSERT INTO team_metadata (team_name, key, value, updated_at) \
                     VALUES (?1, ?2, ?3, ?4)",
                    params![team_name, key, value, now],
                )
                .map_err(|e| {
                    TeamRegistryError::Internal(e.to_string())
                })?;
            }

            tx.commit().map_err(|e| TeamRegistryError::Internal(e.to_string()))?;

            Ok(())
        })
    }

    fn verify_code<'a>(
        &'a self,
        team_name: &'a str,
        code: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<bool, TeamRegistryError>> + Send + 'a>>
    {
        Box::pin(async move {
            // Check team code first
            let expected_team_code = Self::compute_team_code(&self.salt, team_name);
            if Self::constant_time_eq(&expected_team_code, code) {
                return Ok(true);
            }

            // Check master password: stored master_hash = HMAC(salt, master_password)
            // Submitted code is the raw master password, so compute HMAC of it and compare
            let conn = self.conn.lock().map_err(|e| {
                TeamRegistryError::Internal(format!("Lock poisoned: {e}"))
            })?;

            let master_hash: Option<String> = conn
                .query_row(
                    "SELECT value FROM config WHERE key = 'master_hash'",
                    [],
                    |row| row.get(0),
                )
                .ok();

            if let Some(stored_hash) = master_hash {
                let submitted_hash = Self::compute_hmac(&self.salt, code);
                if Self::constant_time_eq(&stored_hash, &submitted_hash) {
                    return Ok(true);
                }
            }

            Ok(false)
        })
    }

    fn generate_team_code<'a>(
        &'a self,
        team_name: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<String, TeamRegistryError>> + Send + 'a>>
    {
        Box::pin(async move {
            Ok(Self::compute_team_code(&self.salt, team_name))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_client() -> TeamsSqliteClient {
        TeamsSqliteClient::new(TeamsSqliteConfig {
            filename: ":memory:".to_string(),
            master_password: Some("test-master-pw".to_string()),
        })
    }

    #[tokio::test]
    async fn test_get_empty_team() {
        let client = test_client();
        let result = client.get_team_metadata("UnknownTeam").await.unwrap();
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn test_set_and_get_metadata() {
        let client = test_client();
        client
            .set_team_metadata(
                "TeamA",
                vec![
                    ("github".to_string(), "https://github.com/a".to_string()),
                    ("website".to_string(), "https://example.com".to_string()),
                ],
            )
            .await
            .unwrap();

        let entries = client.get_team_metadata("TeamA").await.unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].key, "github");
        assert_eq!(entries[0].value, "https://github.com/a");
        assert_eq!(entries[1].key, "website");
        assert_eq!(entries[1].value, "https://example.com");
    }

    #[tokio::test]
    async fn test_set_replaces_existing() {
        let client = test_client();
        client
            .set_team_metadata("TeamA", vec![("github".to_string(), "old".to_string())])
            .await
            .unwrap();
        client
            .set_team_metadata("TeamA", vec![("github".to_string(), "new".to_string())])
            .await
            .unwrap();

        let entries = client.get_team_metadata("TeamA").await.unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].value, "new");
    }

    #[tokio::test]
    async fn test_set_empty_clears_entries() {
        let client = test_client();
        client
            .set_team_metadata("TeamA", vec![("github".to_string(), "val".to_string())])
            .await
            .unwrap();
        client
            .set_team_metadata("TeamA", vec![])
            .await
            .unwrap();

        let entries = client.get_team_metadata("TeamA").await.unwrap();
        assert!(entries.is_empty());
    }

    #[tokio::test]
    async fn test_multiple_values_same_key() {
        let client = test_client();
        client
            .set_team_metadata(
                "TeamA",
                vec![
                    ("github".to_string(), "https://github.com/a".to_string()),
                    ("github".to_string(), "https://github.com/b".to_string()),
                ],
            )
            .await
            .unwrap();

        let entries = client.get_team_metadata("TeamA").await.unwrap();
        assert_eq!(entries.len(), 2);
        assert!(entries.iter().all(|e| e.key == "github"));
    }

    #[tokio::test]
    async fn test_teams_are_isolated() {
        let client = test_client();
        client
            .set_team_metadata("TeamA", vec![("key".to_string(), "valueA".to_string())])
            .await
            .unwrap();
        client
            .set_team_metadata("TeamB", vec![("key".to_string(), "valueB".to_string())])
            .await
            .unwrap();

        let a = client.get_team_metadata("TeamA").await.unwrap();
        let b = client.get_team_metadata("TeamB").await.unwrap();
        assert_eq!(a.len(), 1);
        assert_eq!(a[0].value, "valueA");
        assert_eq!(b.len(), 1);
        assert_eq!(b[0].value, "valueB");
    }

    #[tokio::test]
    async fn test_verify_team_code() {
        let client = test_client();
        let code = client.generate_team_code("TeamA").await.unwrap();

        assert!(client.verify_code("TeamA", &code).await.unwrap());
        assert!(!client.verify_code("TeamA", "wrongcode1234567").await.unwrap());
        assert!(!client.verify_code("TeamB", &code).await.unwrap());
    }

    #[tokio::test]
    async fn test_verify_master_password() {
        let client = test_client();
        // master password should work for any team
        assert!(client.verify_code("TeamA", "test-master-pw").await.unwrap());
        assert!(client.verify_code("TeamB", "test-master-pw").await.unwrap());
        assert!(client.verify_code("SomeOtherTeam", "test-master-pw").await.unwrap());
    }

    #[tokio::test]
    async fn test_verify_wrong_master_password() {
        let client = test_client();
        assert!(!client.verify_code("TeamA", "wrong-password").await.unwrap());
    }

    #[tokio::test]
    async fn test_generate_team_code_deterministic() {
        let client = test_client();
        let code1 = client.generate_team_code("TeamA").await.unwrap();
        let code2 = client.generate_team_code("TeamA").await.unwrap();
        assert_eq!(code1, code2);
        assert_eq!(code1.len(), 16);
    }

    #[tokio::test]
    async fn test_different_teams_different_codes() {
        let client = test_client();
        let code_a = client.generate_team_code("TeamA").await.unwrap();
        let code_b = client.generate_team_code("TeamB").await.unwrap();
        assert_ne!(code_a, code_b);
    }

    #[tokio::test]
    async fn test_salt_persists_across_instances() {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let db_path_str = db_path.to_str().unwrap().to_string();

        let code1 = {
            let client = TeamsSqliteClient::new(TeamsSqliteConfig {
                filename: db_path_str.clone(),
                master_password: None,
            });
            client.generate_team_code("TeamA").await.unwrap()
        };

        let code2 = {
            let client = TeamsSqliteClient::new(TeamsSqliteConfig {
                filename: db_path_str,
                master_password: None,
            });
            client.generate_team_code("TeamA").await.unwrap()
        };

        assert_eq!(code1, code2);
    }
}
