use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};

use data_structures::IDF;
use rusqlite::{Connection, params};
use serde::Deserialize;
use tracing::info;

use crate::metadata::{MetadataClient, MetadataClientError};

pub struct SqliteClient {
    config: SqliteConfig,
    conn: Arc<Mutex<Connection>>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SqliteConfig {
    pub filename: String,
    pub run: String,
}

impl SqliteClient {
    pub fn new(config: SqliteConfig) -> Self {
        let conn = Connection::open(&config.filename).expect("Failed to open SQLite database");

        // Enable WAL mode for better concurrency
        conn.query_row("PRAGMA journal_mode=WAL;", [], |_| Ok(()))
            .expect("Failed to set WAL mode");

        let client = Self {
            config,
            conn: Arc::new(Mutex::new(conn)),
        };

        client.ensure_database_idf();
        client.ensure_database_tdp();

        client
    }

    fn ensure_database_idf(&self) {
        let conn = self.conn.lock().unwrap();

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
        .expect("Failed to create table idf_index");

        conn.execute("CREATE INDEX IF NOT EXISTS idx_run ON idf_index (run)", [])
            .expect("Failed to create index on idf_index (run)");
    }

    fn ensure_database_tdp(&self) {
        let conn = self.conn.lock().unwrap();

        conn.execute(
            "CREATE TABLE IF NOT EXISTS tdp (
                run TEXT NOT NULL,
                league VARCHAR(50) NOT NULL,
                year INTEGER NOT NULL,
                team VARCHAR(100) NOT NULL,
                idx INTEGER NOT NULL,
                lyti VARCHAR(100) NOT NULL PRIMARY KEY
            )",
            [],
        )
        .expect("Failed to create table tdp");

        conn.execute("CREATE INDEX IF NOT EXISTS tdp_run ON tdp (run)", [])
            .expect("Failed to create index on tdp (run)");

        conn.execute("CREATE INDEX IF NOT EXISTS tdp_league ON tdp (league)", [])
            .expect("Failed to create index on tdp (league)");

        conn.execute("CREATE INDEX IF NOT EXISTS tdp_year ON tdp (year)", [])
            .expect("Failed to create index on tdp (year)");

        conn.execute("CREATE INDEX IF NOT EXISTS tdp_team ON tdp (team)", [])
            .expect("Failed to create index on tdp (team)");
    }
}

impl MetadataClient for SqliteClient {
    fn store_idf<'a>(
        &'a self,
        map: IDF,
    ) -> Pin<Box<dyn Future<Output = Result<(), MetadataClientError>> + Send + 'a>> {
        let conn = self.conn.clone();
        let run = self.config.run.clone();

        Box::pin(async move {
            tokio::task::spawn_blocking(move || {
                let mut conn = conn.lock().unwrap();

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
    ) -> Pin<Box<dyn Future<Output = Result<IDF, MetadataClientError>> + Send + 'a>> {
        let conn = self.conn.clone();
        let run = self.config.run.clone();

        Box::pin(async move {
            tokio::task::spawn_blocking(move || {
                let conn = conn.lock().unwrap();

                let mut stmt = conn
                    .prepare("SELECT word, idx, idf FROM idf_index WHERE run = ?1")
                    .map_err(|e| MetadataClientError::Internal(e.to_string()))?;

                info!("Retrieving IDF from sqlite database..");
                let rows = stmt
                    .query_map(params![run], |row| {
                        Ok((
                            row.get::<_, String>(0)?,
                            row.get::<_, u32>(1)?,
                            row.get::<_, f32>(2)?,
                        ))
                    })
                    .map_err(|e| MetadataClientError::Internal(e.to_string()))?;

                let mut map = IDF::new();
                for row in rows {
                    let (word, idx, idf) =
                        row.map_err(|e| MetadataClientError::Internal(e.to_string()))?;
                    map.insert(word, (idx, idf));
                }

                info!("Retrieved IDF with {} rows", map.len());

                Ok(map)
            })
            .await
            .map_err(|e| MetadataClientError::Internal(e.to_string()))?
        })
    }

    fn store_tdps<'a>(
        &'a self,
        tdps: Vec<data_structures::paper::TDP>,
    ) -> Pin<Box<dyn Future<Output = Result<(), MetadataClientError>> + Send + 'a>> {
        let conn = self.conn.clone();
        let run = self.config.run.clone();

        Box::pin(async move {
            tokio::task::spawn_blocking(move || {
                let mut conn = conn.lock().unwrap();

                let tx = conn
                    .transaction()
                    .map_err(|e| MetadataClientError::Internal(e.to_string()))?;

                {
                    // Clear existing entries for this run_id to ensure overwrite
                    tx.execute("DELETE FROM tdp WHERE run = ?1", params![run])
                        .map_err(|e| MetadataClientError::Internal(e.to_string()))?;

                    let mut stmt = tx
                        .prepare("INSERT INTO tdp (run, league, year, team, idx, lyti) VALUES (?1, ?2, ?3, ?4, ?5, ?6)")
                        .map_err(|e| MetadataClientError::Internal(e.to_string()))?;

                    for tdp in tdps.into_iter() {
                        let lyti = tdp.name.get_filename();
                        let league = tdp.name.league.name_pretty;
                        let year = tdp.name.year;
                        let team = tdp.name.team_name.name_pretty;
                        let idx = tdp.name.index;
                        stmt.execute(params![run, league, year, team, idx, lyti])
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

    fn load_tdps<'a>(
        &'a self,
    ) -> Pin<
        Box<
            dyn Future<Output = Result<Vec<data_structures::file::TDPName>, MetadataClientError>>
                + Send
                + 'a,
        >,
    > {
        let conn = self.conn.clone();
        let run = self.config.run.clone();

        Box::pin(async move {
            tokio::task::spawn_blocking(move || {
                let conn = conn.lock().unwrap();

                let mut stmt = conn
                    .prepare("SELECT lyti FROM tdp WHERE run = ?1")
                    .map_err(|e| MetadataClientError::Internal(e.to_string()))?;

                let rows = stmt
                    .query_map(params![run], |row| {
                        let lyti: String = row.get(0)?;
                        Ok(lyti)
                    })
                    .map_err(|e| MetadataClientError::Internal(e.to_string()))?;

                let mut results = Vec::new();
                for row in rows {
                    let lyti = row.map_err(|e| MetadataClientError::Internal(e.to_string()))?;
                    let tdp_name = data_structures::file::TDPName::try_from(lyti.as_str())
                        .map_err(|e| MetadataClientError::Internal(e.to_string()))?;
                    results.push(tdp_name);
                }

                Ok(results)
            })
            .await
            .map_err(|e| MetadataClientError::Internal(e.to_string()))?
        })
    }

    fn load_teams<'a>(
        &'a self,
    ) -> Pin<
        Box<
            dyn Future<Output = Result<Vec<data_structures::file::TeamName>, MetadataClientError>>
                + Send
                + 'a,
        >,
    > {
        let conn = self.conn.clone();
        let run = self.config.run.clone();

        Box::pin(async move {
            tokio::task::spawn_blocking(move || {
                let conn = conn.lock().unwrap();

                let mut stmt = conn
                    .prepare("SELECT DISTINCT team FROM tdp WHERE run = ?1")
                    .map_err(|e| MetadataClientError::Internal(e.to_string()))?;

                let rows = stmt
                    .query_map(params![run], |row| {
                        let team: String = row.get(0)?;
                        Ok(team)
                    })
                    .map_err(|e| MetadataClientError::Internal(e.to_string()))?;

                let mut results = Vec::new();
                for row in rows {
                    let team_str = row.map_err(|e| MetadataClientError::Internal(e.to_string()))?;
                    results.push(data_structures::file::TeamName::from_pretty(&team_str));
                }

                Ok(results)
            })
            .await
            .map_err(|e| MetadataClientError::Internal(e.to_string()))?
        })
    }

    fn load_leagues<'a>(
        &'a self,
    ) -> Pin<
        Box<
            dyn Future<Output = Result<Vec<data_structures::file::League>, MetadataClientError>>
                + Send
                + 'a,
        >,
    > {
        let conn = self.conn.clone();
        let run = self.config.run.clone();

        Box::pin(async move {
            tokio::task::spawn_blocking(move || {
                let conn = conn.lock().unwrap();

                let mut stmt = conn
                    .prepare("SELECT DISTINCT league FROM tdp WHERE run = ?1")
                    .map_err(|e| MetadataClientError::Internal(e.to_string()))?;

                let rows = stmt
                    .query_map(params![run], |row| {
                        let league: String = row.get(0)?;
                        Ok(league)
                    })
                    .map_err(|e| MetadataClientError::Internal(e.to_string()))?;

                let mut results = Vec::new();
                for row in rows {
                    let league_str =
                        row.map_err(|e| MetadataClientError::Internal(e.to_string()))?;
                    let league = data_structures::file::League::try_from(league_str.as_str())
                        .map_err(|e| MetadataClientError::Internal(e.to_string()))?;
                    results.push(league);
                }

                Ok(results)
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
        let mut map_1 = IDF::new();
        map_1.insert("apple".to_string(), (1, 1.0));
        map_1.insert("banana".to_string(), (2, 2.0));

        client_1
            .store_idf(map_1.clone())
            .await
            .expect("Failed to store map 1");

        // 2. Load map for run_1
        let loaded_map_1 = client_1.load_idf().await.expect("Failed to load map 1");
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

        let mut map_2 = IDF::new();
        map_2.insert("cherry".to_string(), (3, 3.0));

        client_2
            .store_idf(map_2.clone())
            .await
            .expect("Failed to store map 2");

        // 4. Load map for run_2 and verify run_1 is untouched
        let loaded_map_2 = client_2.load_idf().await.expect("Failed to load map 2");
        assert_eq!(
            map_2, loaded_map_2,
            "Loaded map 2 should match stored map 2"
        );

        let loaded_map_1_again = client_1.load_idf().await.expect("Failed to reload map 1");
        assert_eq!(
            map_1, loaded_map_1_again,
            "Map 1 should persist after storing map 2"
        );

        // 5. Overwrite run_1
        let mut map_1_new = IDF::new();
        map_1_new.insert("apple".to_string(), (1, 1.5)); // Updated value
        map_1_new.insert("date".to_string(), (4, 4.0)); // New value
        // "banana" is removed

        client_1
            .store_idf(map_1_new.clone())
            .await
            .expect("Failed to overwrite map 1");

        // 6. Verify overwrite
        let loaded_map_1_new = client_1
            .load_idf()
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

        let config = SqliteConfig {
            filename: db_filename.to_string(),
            run: "my_run".to_string(),
        };
        let client = SqliteClient::new(config);

        let idfs = client.load_idf().await?;
        println!("Number of entries in {db_filename} (IDF): {}", idfs.len());

        let tdps = client.load_tdps().await?;
        println!("Number of TDPs: {}", tdps.len());
        for tdp in tdps.iter().take(5) {
            println!("  {}", tdp.get_filename());
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_load_teams_and_leagues() {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let db_filename = format!("test_teams_{}.db", timestamp);
        let run = "test_run";

        let config = SqliteConfig {
            filename: db_filename.clone(),
            run: run.to_string(),
        };
        let client = SqliteClient::new(config);

        // Prepare dummy TDPs
        let league1 = data_structures::file::League::try_from("soccer_smallsize").unwrap();
        let league2 = data_structures::file::League::try_from("soccer_midsize").unwrap();
        let team1 = data_structures::file::TeamName::new("RoboTeam Twente");
        let team2 = data_structures::file::TeamName::new("Tigers Mannheim");

        let tdp1 = data_structures::paper::TDP {
            name: data_structures::file::TDPName::new(
                league1.clone(),
                2019,
                team1.clone(),
                Some(1),
            ),
            structure: data_structures::paper::TDPStructure { paragraphs: vec![] },
        };
        let tdp2 = data_structures::paper::TDP {
            name: data_structures::file::TDPName::new(
                league1.clone(),
                2019,
                team2.clone(),
                Some(1),
            ),
            structure: data_structures::paper::TDPStructure { paragraphs: vec![] },
        };
        let tdp3 = data_structures::paper::TDP {
            name: data_structures::file::TDPName::new(
                league2.clone(),
                2020,
                team1.clone(),
                Some(1),
            ),
            structure: data_structures::paper::TDPStructure { paragraphs: vec![] },
        };

        client
            .store_tdps(vec![tdp1, tdp2, tdp3])
            .await
            .expect("Failed to store TDPs");

        // Test load_teams
        let teams = client.load_teams().await.expect("Failed to load teams");
        assert_eq!(teams.len(), 2);
        let team_names: Vec<String> = teams.iter().map(|t| t.name_pretty.clone()).collect();
        assert!(team_names.contains(&"RoboTeam Twente".to_string()));
        assert!(team_names.contains(&"Tigers Mannheim".to_string()));

        // Test load_leagues
        let leagues = client.load_leagues().await.expect("Failed to load leagues");
        assert_eq!(leagues.len(), 2);
        let league_names: Vec<String> = leagues.iter().map(|l| l.name_pretty.clone()).collect();
        assert!(league_names.contains(&"Soccer SmallSize".to_string()));
        assert!(league_names.contains(&"Soccer MidSize".to_string()));

        // Cleanup
        fs::remove_file(&db_filename).expect("Failed to delete database file");
    }
}
