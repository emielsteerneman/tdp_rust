use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;

use rusqlite::{Connection, params};
use serde::Deserialize;

use crate::metadata::{MetadataClient, MetadataClientError};

pub struct SqliteClient {
    config: SqliteConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SqliteConfig {
    pub filename: String,
    pub run: String,
}

impl SqliteClient {
    pub fn new(config: SqliteConfig) -> Self {
        let client = Self { config };

        client.ensure_database();

        client
    }

    fn ensure_database(&self) {
        let conn = Connection::open(&self.config.filename).expect("Failed to open SQLite database");

        conn.execute(
            "CREATE TABLE IF NOT EXISTS idf_index (
                word TEXT NOT NULL,
                run TEXT NOT NULL,
                idx INTEGER NOT NULL,
                idf REAL NOT NULL,
                UNIQUE(word, run)
            )",
            [],
        )
        .expect("Failed to create idf_index table");

        conn.execute("CREATE INDEX IF NOT EXISTS idx_run ON idf_index (run)", [])
            .expect("Failed to create index on run");
    }
}

impl MetadataClient for SqliteClient {
    fn store_idf<'a>(
        &'a self,
        map: HashMap<String, (u32, f32)>,
    ) -> Pin<Box<dyn Future<Output = Result<(), MetadataClientError>> + Send + 'a>> {
        let filename = self.config.filename.clone();
        let run = self.config.run.clone();

        Box::pin(async move {
            tokio::task::spawn_blocking(move || {
                let mut conn = Connection::open(&filename)
                    .map_err(|e| MetadataClientError::Internal(e.to_string()))?;

                let tx = conn
                    .transaction()
                    .map_err(|e| MetadataClientError::Internal(e.to_string()))?;

                {
                    // Clear existing entries for this run_id to ensure overwrite
                    tx.execute("DELETE FROM idf_index WHERE run = ?1", params![run])
                        .map_err(|e| MetadataClientError::Internal(e.to_string()))?;

                    let mut stmt = tx
                        .prepare(
                            "INSERT INTO idf_index (word, run, idx, idf) VALUES (?1, ?2, ?3, ?4)",
                        )
                        .map_err(|e| MetadataClientError::Internal(e.to_string()))?;

                    for (word, (idx, idf)) in map.iter() {
                        stmt.execute(params![word, run, idx, idf])
                            .map_err(|e| MetadataClientError::Internal(e.to_string()))?;
                    }
                }

                tx.commit()
                    .map_err(|e| MetadataClientError::Internal(e.to_string()))?;

                Ok(())
            })
            .await
            .map_err(|e| MetadataClientError::Internal(e.to_string()))?
        })
    }

    fn load_idf<'a>(
        &'a self,
        run: String,
    ) -> Pin<
        Box<
            dyn Future<Output = Result<HashMap<String, (u32, f32)>, MetadataClientError>>
                + Send
                + 'a,
        >,
    > {
        let filename = self.config.filename.clone();

        Box::pin(async move {
            tokio::task::spawn_blocking(move || {
                let conn = Connection::open(&filename)
                    .map_err(|e| MetadataClientError::Internal(e.to_string()))?;

                let mut stmt = conn
                    .prepare("SELECT word, idx, idf FROM idf_index WHERE run = ?1")
                    .map_err(|e| MetadataClientError::Internal(e.to_string()))?;

                let rows = stmt
                    .query_map(params![run], |row| {
                        Ok((
                            row.get::<_, String>(0)?,
                            row.get::<_, u32>(1)?,
                            row.get::<_, f32>(2)?,
                        ))
                    })
                    .map_err(|e| MetadataClientError::Internal(e.to_string()))?;

                let mut map = HashMap::new();
                for row in rows {
                    let (word, idx, idf) =
                        row.map_err(|e| MetadataClientError::Internal(e.to_string()))?;
                    map.insert(word, (idx, idf));
                }

                Ok(map)
            })
            .await
            .map_err(|e| MetadataClientError::Internal(e.to_string()))?
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[tokio::test]
    async fn test_sqlite_client_lifecycle() {
        // Generate a unique filename using timestamp
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let db_filename = format!("test_idf_{}.db", timestamp);

        let run_1 = "run_1";
        let run_2 = "run_2";

        // Setup client 1
        let config_1 = SqliteConfig {
            filename: db_filename.clone(),
            run: run_1.to_string(),
        };
        let client_1 = SqliteClient::new(config_1);

        // 1. Store map for run_1
        let mut map_1 = HashMap::new();
        map_1.insert("apple".to_string(), (1, 1.0));
        map_1.insert("banana".to_string(), (2, 2.0));

        client_1
            .store_idf(map_1.clone())
            .await
            .expect("Failed to store map 1");

        // 2. Load map for run_1
        let loaded_map_1 = client_1
            .load_idf(run_1.to_string())
            .await
            .expect("Failed to load map 1");
        assert_eq!(
            map_1, loaded_map_1,
            "Loaded map 1 should match stored map 1"
        );

        // 3. Store map for run_2
        let config_2 = SqliteConfig {
            filename: db_filename.clone(),
            run: run_2.to_string(),
        };
        let client_2 = SqliteClient::new(config_2);

        let mut map_2 = HashMap::new();
        map_2.insert("cherry".to_string(), (3, 3.0));

        client_2
            .store_idf(map_2.clone())
            .await
            .expect("Failed to store map 2");

        // 4. Load map for run_2 and verify run_1 is untouched
        let loaded_map_2 = client_2
            .load_idf(run_2.to_string())
            .await
            .expect("Failed to load map 2");
        assert_eq!(
            map_2, loaded_map_2,
            "Loaded map 2 should match stored map 2"
        );

        let loaded_map_1_again = client_1
            .load_idf(run_1.to_string())
            .await
            .expect("Failed to reload map 1");
        assert_eq!(
            map_1, loaded_map_1_again,
            "Map 1 should persist after storing map 2"
        );

        // 5. Overwrite run_1
        let mut map_1_new = HashMap::new();
        map_1_new.insert("apple".to_string(), (1, 1.5)); // Updated value
        map_1_new.insert("date".to_string(), (4, 4.0)); // New value
        // "banana" is removed

        client_1
            .store_idf(map_1_new.clone())
            .await
            .expect("Failed to overwrite map 1");

        // 6. Verify overwrite
        let loaded_map_1_new = client_1
            .load_idf(run_1.to_string())
            .await
            .expect("Failed to load overwritten map 1");
        assert_eq!(
            map_1_new, loaded_map_1_new,
            "Map 1 should match the new map after overwrite"
        );

        // Ensure "banana" is gone
        assert!(
            !loaded_map_1_new.contains_key("banana"),
            "Old keys should be removed on overwrite"
        );

        // 7. Cleanup
        fs::remove_file(&db_filename).expect("Failed to delete database file");
    }

    #[tokio::test]
    async fn test_read_existing_db() -> Result<(), Box<dyn std::error::Error>> {
        let db_filename = "../my_sqlite.db";

        // Check if file exists
        match std::fs::exists(db_filename) {
            Ok(true) => {}
            _ => return Err("Database file does not exist".into()),
        };

        let conn = Connection::open(&db_filename)
            .map_err(|e| MetadataClientError::Internal(e.to_string()))?;

        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM idf_index", [], |row| row.get(0))
            .map_err(|e| MetadataClientError::Internal(e.to_string()))?;

        println!("Number of entries in {db_filename} : {count}");
        Ok(())
    }
}
