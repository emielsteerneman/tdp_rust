use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};

use data_structures::IDF;
use data_structures::content::{Author, ContentItem, ContentType, MarkdownTDP, PaperInfo, TocEntry};
use rusqlite::{Connection, params};
use serde::Deserialize;
use tracing::info;

use crate::metadata::{MetadataClient, MetadataClientError};

pub struct SqliteClient {
    conn: Arc<Mutex<Connection>>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SqliteConfig {
    pub filename: String,
}

impl SqliteClient {
    pub fn new(config: SqliteConfig) -> Self {
        let conn = Connection::open(&config.filename).expect("Failed to open SQLite database");

        // Enable WAL mode for better concurrency
        conn.query_row("PRAGMA journal_mode=WAL;", [], |_| Ok(()))
            .expect("Failed to set WAL mode");

        let client = Self {
            conn: Arc::new(Mutex::new(conn)),
        };

        client.ensure_database_idf();
        client.ensure_database_paper_v2();

        client
    }

    fn ensure_database_idf(&self) {
        let conn = self.conn.lock().unwrap();

        conn.execute(
            "CREATE TABLE IF NOT EXISTS idf_index (
                word TEXT NOT NULL UNIQUE,
                idx INTEGER NOT NULL,
                idf REAL NOT NULL
            )",
            [],
        )
        .expect("Failed to create table idf_index");
    }

    fn ensure_database_paper_v2(&self) {
        let conn = self.conn.lock().unwrap();

        conn.execute(
            "CREATE TABLE IF NOT EXISTS paper (
                paper_lyt TEXT PRIMARY KEY,
                league TEXT NOT NULL,
                year INTEGER NOT NULL,
                team TEXT NOT NULL,
                title TEXT,
                abstract_text TEXT,
                urls_json TEXT,
                raw_markdown TEXT
            )",
            [],
        )
        .expect("Failed to create table paper");

        conn.execute(
            "CREATE TABLE IF NOT EXISTS author (
                paper_lyt TEXT NOT NULL,
                name TEXT NOT NULL,
                affiliation TEXT,
                FOREIGN KEY (paper_lyt) REFERENCES paper(paper_lyt)
            )",
            [],
        )
        .expect("Failed to create table author");

        conn.execute(
            "CREATE TABLE IF NOT EXISTS toc_entry (
                paper_lyt TEXT NOT NULL,
                content_seq INTEGER NOT NULL,
                content_type TEXT NOT NULL,
                depth INTEGER NOT NULL,
                title TEXT NOT NULL,
                body TEXT,
                image_path TEXT,
                FOREIGN KEY (paper_lyt) REFERENCES paper(paper_lyt),
                UNIQUE(paper_lyt, content_seq)
            )",
            [],
        )
        .expect("Failed to create table toc_entry");

        conn.execute("CREATE INDEX IF NOT EXISTS paper_league ON paper (league)", [])
            .expect("Failed to create index on paper (league)");

        conn.execute("CREATE INDEX IF NOT EXISTS paper_year ON paper (year)", [])
            .expect("Failed to create index on paper (year)");

        conn.execute("CREATE INDEX IF NOT EXISTS paper_team ON paper (team)", [])
            .expect("Failed to create index on paper (team)");
    }
}

impl MetadataClient for SqliteClient {
    fn store_idf<'a>(
        &'a self,
        map: IDF,
    ) -> Pin<Box<dyn Future<Output = Result<(), MetadataClientError>> + Send + 'a>> {
        let conn = self.conn.clone();

        Box::pin(async move {
            tokio::task::spawn_blocking(move || {
                let mut conn = conn.lock().unwrap();

                let tx = conn
                    .transaction()
                    .map_err(|e| MetadataClientError::Internal(e.to_string()))?;

                {
                    // Clear existing entries to ensure overwrite
                    tx.execute("DELETE FROM idf_index", [])
                        .map_err(|e| MetadataClientError::Internal(e.to_string()))?;

                    let mut stmt = tx
                        .prepare(
                            "INSERT INTO idf_index (word, idx, idf) VALUES (?1, ?2, ?3)",
                        )
                        .map_err(|e| MetadataClientError::Internal(e.to_string()))?;

                    for (word, (idx, idf)) in map.iter() {
                        stmt.execute(params![word, idx, idf])
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

        Box::pin(async move {
            tokio::task::spawn_blocking(move || {
                let conn = conn.lock().unwrap();

                let mut stmt = conn
                    .prepare("SELECT word, idx, idf FROM idf_index")
                    .map_err(|e| MetadataClientError::Internal(e.to_string()))?;

                info!("Retrieving IDF from sqlite database..");
                let rows = stmt
                    .query_map([], |row| {
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

        Box::pin(async move {
            tokio::task::spawn_blocking(move || {
                let conn = conn.lock().unwrap();

                let mut stmt = conn
                    .prepare("SELECT paper_lyt FROM paper")
                    .map_err(|e| MetadataClientError::Internal(e.to_string()))?;

                let rows = stmt
                    .query_map([], |row| {
                        let paper_lyt: String = row.get(0)?;
                        Ok(paper_lyt)
                    })
                    .map_err(|e| MetadataClientError::Internal(e.to_string()))?;

                let mut results = Vec::new();
                for row in rows {
                    let paper_lyt = row.map_err(|e| MetadataClientError::Internal(e.to_string()))?;
                    let tdp_name = data_structures::file::TDPName::try_from(paper_lyt.as_str())
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

        Box::pin(async move {
            tokio::task::spawn_blocking(move || {
                let conn = conn.lock().unwrap();

                let mut stmt = conn
                    .prepare("SELECT DISTINCT team FROM paper")
                    .map_err(|e| MetadataClientError::Internal(e.to_string()))?;

                let rows = stmt
                    .query_map([], |row| {
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

        Box::pin(async move {
            tokio::task::spawn_blocking(move || {
                let conn = conn.lock().unwrap();

                let mut stmt = conn
                    .prepare("SELECT DISTINCT league FROM paper")
                    .map_err(|e| MetadataClientError::Internal(e.to_string()))?;

                let rows = stmt
                    .query_map([], |row| {
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

    fn get_tdp_markdown<'a>(
        &'a self,
        tdp_name: data_structures::file::TDPName,
    ) -> Pin<Box<dyn Future<Output = Result<String, MetadataClientError>> + Send + 'a>> {
        let conn = self.conn.clone();

        Box::pin(async move {
            tokio::task::spawn_blocking(move || {
                let conn = conn.lock().unwrap();
                let paper_lyt = tdp_name.get_paper_lyt();

                let mut stmt = conn
                    .prepare("SELECT raw_markdown FROM paper WHERE paper_lyt = ?1")
                    .map_err(|e| MetadataClientError::Internal(e.to_string()))?;

                let markdown: String = stmt
                    .query_row(params![paper_lyt], |row| row.get(0))
                    .map_err(|e| match e {
                        rusqlite::Error::QueryReturnedNoRows => {
                            MetadataClientError::NotFound(paper_lyt.clone())
                        }
                        _ => MetadataClientError::Internal(e.to_string()),
                    })?;

                Ok(markdown)
            })
            .await
            .map_err(|e| MetadataClientError::Internal(e.to_string()))?
        })
    }

    fn print_analytics<'a>(
        &'a self,
    ) -> Pin<Box<dyn Future<Output = Result<(), MetadataClientError>> + Send + 'a>> {
        let conn = self.conn.clone();

        Box::pin(async move {
            tokio::task::spawn_blocking(move || {
                let conn = conn.lock().unwrap();

                let tdp_count: i64 = conn
                    .query_row(
                        "SELECT COUNT(*) FROM paper",
                        [],
                        |row| row.get(0),
                    )
                    .map_err(|e| MetadataClientError::Internal(e.to_string()))?;

                let idf_count: i64 = conn
                    .query_row(
                        "SELECT COUNT(*) FROM idf_index",
                        [],
                        |row| row.get(0),
                    )
                    .map_err(|e| MetadataClientError::Internal(e.to_string()))?;

                info!(
                    "Analytics: {} TDPs, {} IDF entries",
                    tdp_count, idf_count
                );

                Ok(())
            })
            .await
            .map_err(|e| MetadataClientError::Internal(e.to_string()))?
        })
    }

    fn store_paper<'a>(
        &'a self,
        tdp: MarkdownTDP,
    ) -> Pin<Box<dyn Future<Output = Result<(), MetadataClientError>> + Send + 'a>> {
        let conn = self.conn.clone();

        Box::pin(async move {
            tokio::task::spawn_blocking(move || {
                let mut conn = conn.lock().unwrap();
                let paper_lyt = tdp.name.get_paper_lyt();
                let league = tdp.name.league.name();
                let year = tdp.name.year;
                let team = &tdp.name.team_name.name_pretty;
                let title = &tdp.front_matter.title;
                let abstract_text = tdp.front_matter.abstract_text.as_deref();
                let urls_json = serde_json::to_string(&tdp.front_matter.urls)
                    .map_err(|e| MetadataClientError::Internal(e.to_string()))?;
                let raw_markdown = &tdp.raw_markdown;

                let tx = conn
                    .transaction()
                    .map_err(|e| MetadataClientError::Internal(e.to_string()))?;

                {
                    // Delete existing data for this paper_lyt (upsert)
                    tx.execute("DELETE FROM toc_entry WHERE paper_lyt = ?1", params![paper_lyt])
                        .map_err(|e| MetadataClientError::Internal(e.to_string()))?;
                    tx.execute("DELETE FROM author WHERE paper_lyt = ?1", params![paper_lyt])
                        .map_err(|e| MetadataClientError::Internal(e.to_string()))?;
                    tx.execute("DELETE FROM paper WHERE paper_lyt = ?1", params![paper_lyt])
                        .map_err(|e| MetadataClientError::Internal(e.to_string()))?;

                    // Insert paper
                    tx.execute(
                        "INSERT INTO paper (paper_lyt, league, year, team, title, abstract_text, urls_json, raw_markdown) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                        params![paper_lyt, league, year, team, title, abstract_text, urls_json, raw_markdown],
                    )
                    .map_err(|e| MetadataClientError::Internal(e.to_string()))?;

                    // Insert authors
                    let mut author_stmt = tx
                        .prepare("INSERT INTO author (paper_lyt, name, affiliation) VALUES (?1, ?2, ?3)")
                        .map_err(|e| MetadataClientError::Internal(e.to_string()))?;

                    for author in &tdp.front_matter.authors {
                        author_stmt
                            .execute(params![paper_lyt, author.name, author.affiliation])
                            .map_err(|e| MetadataClientError::Internal(e.to_string()))?;
                    }
                    drop(author_stmt);

                    // Insert content items into toc_entry
                    let mut toc_stmt = tx
                        .prepare("INSERT INTO toc_entry (paper_lyt, content_seq, content_type, depth, title, body, image_path) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)")
                        .map_err(|e| MetadataClientError::Internal(e.to_string()))?;

                    for item in &tdp.content_items {
                        toc_stmt
                            .execute(params![
                                paper_lyt,
                                item.content_seq,
                                item.content_type.as_str(),
                                item.depth,
                                item.title,
                                item.body,
                                item.image_path
                            ])
                            .map_err(|e| MetadataClientError::Internal(e.to_string()))?;
                    }
                    drop(toc_stmt);
                }

                tx.commit()
                    .map_err(|e| MetadataClientError::Internal(e.to_string()))?;

                Ok(())
            })
            .await
            .map_err(|e| MetadataClientError::Internal(e.to_string()))?
        })
    }

    fn load_toc<'a>(
        &'a self,
        paper_lyt: String,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<TocEntry>, MetadataClientError>> + Send + 'a>> {
        let conn = self.conn.clone();

        Box::pin(async move {
            tokio::task::spawn_blocking(move || {
                let conn = conn.lock().unwrap();

                let mut stmt = conn
                    .prepare("SELECT content_seq, content_type, depth, title FROM toc_entry WHERE paper_lyt = ?1 ORDER BY content_seq")
                    .map_err(|e| MetadataClientError::Internal(e.to_string()))?;

                let rows = stmt
                    .query_map(params![paper_lyt], |row| {
                        let content_seq: u32 = row.get(0)?;
                        let content_type_str: String = row.get(1)?;
                        let depth: u8 = row.get(2)?;
                        let title: String = row.get(3)?;
                        Ok((content_seq, content_type_str, depth, title))
                    })
                    .map_err(|e| MetadataClientError::Internal(e.to_string()))?;

                let mut results = Vec::new();
                for row in rows {
                    let (content_seq, content_type_str, depth, title) =
                        row.map_err(|e| MetadataClientError::Internal(e.to_string()))?;
                    let content_type = ContentType::try_from(content_type_str.as_str())
                        .map_err(|e| MetadataClientError::Internal(e))?;
                    results.push(TocEntry {
                        content_seq,
                        content_type,
                        depth,
                        title,
                    });
                }

                if results.is_empty() {
                    return Err(MetadataClientError::NotFound(format!(
                        "No toc entries found for paper_lyt: {}",
                        paper_lyt
                    )));
                }

                Ok(results)
            })
            .await
            .map_err(|e| MetadataClientError::Internal(e.to_string()))?
        })
    }

    fn load_content_item<'a>(
        &'a self,
        paper_lyt: String,
        content_seq: u32,
    ) -> Pin<Box<dyn Future<Output = Result<ContentItem, MetadataClientError>> + Send + 'a>> {
        let conn = self.conn.clone();

        Box::pin(async move {
            tokio::task::spawn_blocking(move || {
                let conn = conn.lock().unwrap();

                let mut stmt = conn
                    .prepare("SELECT content_seq, content_type, depth, title, body, image_path FROM toc_entry WHERE paper_lyt = ?1 AND content_seq = ?2")
                    .map_err(|e| MetadataClientError::Internal(e.to_string()))?;

                stmt.query_row(params![paper_lyt, content_seq], |row| {
                    let content_seq: u32 = row.get(0)?;
                    let content_type_str: String = row.get(1)?;
                    let depth: u8 = row.get(2)?;
                    let title: String = row.get(3)?;
                    let body: String = row.get::<_, Option<String>>(4)?.unwrap_or_default();
                    let image_path: Option<String> = row.get(5)?;
                    Ok((content_seq, content_type_str, depth, title, body, image_path))
                })
                .map_err(|e| match e {
                    rusqlite::Error::QueryReturnedNoRows => {
                        MetadataClientError::NotFound(format!(
                            "Content item not found for paper_lyt: {}, content_seq: {}",
                            paper_lyt, content_seq
                        ))
                    }
                    _ => MetadataClientError::Internal(e.to_string()),
                })
                .and_then(|(content_seq, content_type_str, depth, title, body, image_path)| {
                    let content_type = ContentType::try_from(content_type_str.as_str())
                        .map_err(|e| MetadataClientError::Internal(e))?;
                    Ok(ContentItem {
                        content_seq,
                        content_type,
                        depth,
                        title,
                        body,
                        image_path,
                    })
                })
            })
            .await
            .map_err(|e| MetadataClientError::Internal(e.to_string()))?
        })
    }

    fn load_content_items_range<'a>(
        &'a self,
        paper_lyt: String,
        start_seq: u32,
        end_seq_exclusive: u32,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<ContentItem>, MetadataClientError>> + Send + 'a>>
    {
        let conn = self.conn.clone();

        Box::pin(async move {
            tokio::task::spawn_blocking(move || {
                let conn = conn.lock().unwrap();

                let mut stmt = conn
                    .prepare(
                        "SELECT content_seq, content_type, depth, title, body, image_path
                         FROM toc_entry
                         WHERE paper_lyt = ?1 AND content_seq >= ?2 AND content_seq < ?3
                         ORDER BY content_seq",
                    )
                    .map_err(|e| MetadataClientError::Internal(e.to_string()))?;

                let rows = stmt
                    .query_map(params![paper_lyt, start_seq, end_seq_exclusive], |row| {
                        let content_seq: u32 = row.get(0)?;
                        let content_type_str: String = row.get(1)?;
                        let depth: u8 = row.get(2)?;
                        let title: String = row.get(3)?;
                        let body: String = row.get::<_, Option<String>>(4)?.unwrap_or_default();
                        let image_path: Option<String> = row.get(5)?;
                        Ok((content_seq, content_type_str, depth, title, body, image_path))
                    })
                    .map_err(|e| MetadataClientError::Internal(e.to_string()))?;

                let mut results = Vec::new();
                for row in rows {
                    let (content_seq, content_type_str, depth, title, body, image_path) =
                        row.map_err(|e| MetadataClientError::Internal(e.to_string()))?;
                    let content_type = ContentType::try_from(content_type_str.as_str())
                        .map_err(|e| MetadataClientError::Internal(e))?;
                    results.push(ContentItem {
                        content_seq,
                        content_type,
                        depth,
                        title,
                        body,
                        image_path,
                    });
                }

                if results.is_empty() {
                    return Err(MetadataClientError::NotFound(format!(
                        "No content items found for paper_lyt: {}, range: {}..{}",
                        paper_lyt, start_seq, end_seq_exclusive
                    )));
                }

                Ok(results)
            })
            .await
            .map_err(|e| MetadataClientError::Internal(e.to_string()))?
        })
    }

    fn load_paper_abstract<'a>(
        &'a self,
        paper_lyt: String,
    ) -> Pin<Box<dyn Future<Output = Result<String, MetadataClientError>> + Send + 'a>> {
        let conn = self.conn.clone();

        Box::pin(async move {
            tokio::task::spawn_blocking(move || {
                let conn = conn.lock().unwrap();

                let mut stmt = conn
                    .prepare("SELECT abstract_text FROM paper WHERE paper_lyt = ?1")
                    .map_err(|e| MetadataClientError::Internal(e.to_string()))?;

                let abstract_text: Option<String> = stmt
                    .query_row(params![paper_lyt], |row| row.get(0))
                    .map_err(|e| match e {
                        rusqlite::Error::QueryReturnedNoRows => {
                            MetadataClientError::NotFound(format!(
                                "Paper not found for paper_lyt: {}",
                                paper_lyt
                            ))
                        }
                        _ => MetadataClientError::Internal(e.to_string()),
                    })?;

                abstract_text.ok_or_else(|| {
                    MetadataClientError::NotFound(format!(
                        "Abstract not found for paper_lyt: {}",
                        paper_lyt
                    ))
                })
            })
            .await
            .map_err(|e| MetadataClientError::Internal(e.to_string()))?
        })
    }

    fn load_paper_info<'a>(
        &'a self,
        paper_lyt: String,
    ) -> Pin<Box<dyn Future<Output = Result<PaperInfo, MetadataClientError>> + Send + 'a>> {
        let conn = self.conn.clone();

        Box::pin(async move {
            tokio::task::spawn_blocking(move || {
                let conn = conn.lock().unwrap();

                // Load title, urls from paper table
                let (title, urls_json): (Option<String>, Option<String>) = conn
                    .query_row(
                        "SELECT title, urls_json FROM paper WHERE paper_lyt = ?1",
                        params![paper_lyt],
                        |row| Ok((row.get(0)?, row.get(1)?)),
                    )
                    .map_err(|e| match e {
                        rusqlite::Error::QueryReturnedNoRows => {
                            MetadataClientError::NotFound(format!("Paper not found: {}", paper_lyt))
                        }
                        _ => MetadataClientError::Internal(e.to_string()),
                    })?;

                let urls: Vec<String> = urls_json
                    .and_then(|j| serde_json::from_str(&j).ok())
                    .unwrap_or_default();

                // Load authors from author table
                let mut stmt = conn
                    .prepare("SELECT name, affiliation FROM author WHERE paper_lyt = ?1")
                    .map_err(|e| MetadataClientError::Internal(e.to_string()))?;

                let authors: Vec<Author> = stmt
                    .query_map(params![paper_lyt], |row| {
                        Ok(Author {
                            name: row.get(0)?,
                            affiliation: row.get(1)?,
                        })
                    })
                    .map_err(|e| MetadataClientError::Internal(e.to_string()))?
                    .filter_map(|r| r.ok())
                    .collect();

                // Derive institutions from author affiliations
                let mut institutions: Vec<String> = authors
                    .iter()
                    .filter_map(|a| a.affiliation.clone())
                    .filter(|a| !a.is_empty())
                    .collect();
                institutions.sort();
                institutions.dedup();

                Ok(PaperInfo {
                    title: title.unwrap_or_default(),
                    authors,
                    institutions,
                    urls,
                })
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

        let config = SqliteConfig {
            filename: db_filename.clone(),
        };
        let client = SqliteClient::new(config);

        // 1. Store and load IDF
        let mut map = IDF::new();
        map.insert("apple".to_string(), (1, 1.0));
        map.insert("banana".to_string(), (2, 2.0));

        client
            .store_idf(map.clone())
            .await
            .expect("Failed to store map");

        let loaded_map = client.load_idf().await.expect("Failed to load map");
        assert_eq!(map, loaded_map, "Loaded map should match stored map");

        // 2. Overwrite with new data
        let mut map_new = IDF::new();
        map_new.insert("apple".to_string(), (1, 1.5)); // Updated value
        map_new.insert("date".to_string(), (4, 4.0)); // New value
        // "banana" is removed

        client
            .store_idf(map_new.clone())
            .await
            .expect("Failed to overwrite map");

        // 3. Verify overwrite
        let loaded_map_new = client
            .load_idf()
            .await
            .expect("Failed to load overwritten map");
        assert_eq!(
            map_new, loaded_map_new,
            "Map should match the new map after overwrite"
        );

        // Ensure "banana" is gone
        assert!(
            !loaded_map_new.contains_key("banana"),
            "Old keys should be removed on overwrite"
        );

        // 4. Cleanup
        drop(client);
        fs::remove_file(&db_filename).expect("Failed to delete database file");
        let _ = fs::remove_file(format!("{}-wal", db_filename));
        let _ = fs::remove_file(format!("{}-shm", db_filename));
    }

    #[tokio::test]
    async fn test_read_existing_db() -> Result<(), Box<dyn std::error::Error>> {
        let db_filename = "../data/metadata.db";

        // Check if file exists
        match std::fs::exists(db_filename) {
            Ok(true) => {}
            _ => return Err("Database file does not exist".into()),
        };

        let config = SqliteConfig {
            filename: db_filename.to_string(),
        };
        let client = SqliteClient::new(config);

        let idfs = client.load_idf().await?;
        println!("Number of entries in {db_filename} (IDF): {}", idfs.len());

        let tdps = client.load_tdps().await?;
        println!("Number of TDPs: {}", tdps.len());
        for tdp in tdps.iter().take(5) {
            println!("  {}", tdp.get_paper_lyt());
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

        let config = SqliteConfig {
            filename: db_filename.clone(),
        };
        let client = SqliteClient::new(config);

        // Insert rows directly into the paper table
        {
            let conn = client.conn.lock().unwrap();
            conn.execute(
                "INSERT INTO paper (paper_lyt, league, year, team, raw_markdown) VALUES (?1, ?2, ?3, ?4, ?5)",
                params!["soccer_smallsize__2019__RoboTeam_Twente", "Soccer SmallSize", 2019, "RoboTeam Twente", "# Test markdown 1"],
            ).unwrap();
            conn.execute(
                "INSERT INTO paper (paper_lyt, league, year, team, raw_markdown) VALUES (?1, ?2, ?3, ?4, ?5)",
                params!["soccer_smallsize__2019__Tigers_Mannheim", "Soccer SmallSize", 2019, "Tigers Mannheim", "# Test markdown 2"],
            ).unwrap();
            conn.execute(
                "INSERT INTO paper (paper_lyt, league, year, team, raw_markdown) VALUES (?1, ?2, ?3, ?4, ?5)",
                params!["soccer_midsize__2020__RoboTeam_Twente", "Soccer MidSize", 2020, "RoboTeam Twente", "# Test markdown 3"],
            ).unwrap();
        }

        // Test load_teams
        let teams = client.load_teams().await.expect("Failed to load teams");
        assert_eq!(teams.len(), 2);
        let team_names: Vec<String> = teams.iter().map(|t| t.name_pretty.clone()).collect();
        assert!(team_names.contains(&"RoboTeam Twente".to_string()));
        assert!(team_names.contains(&"Tigers Mannheim".to_string()));

        // Test load_leagues
        let leagues = client.load_leagues().await.expect("Failed to load leagues");
        assert_eq!(leagues.len(), 2);
        let league_names: Vec<String> = leagues.iter().map(|l| l.name_pretty().to_string()).collect();
        assert!(league_names.contains(&"Soccer SmallSize".to_string()));
        assert!(league_names.contains(&"Soccer MidSize".to_string()));

        // Test get_tdp_markdown
        let tdp_name = data_structures::file::TDPName::try_from("soccer_smallsize__2019__RoboTeam_Twente").unwrap();
        let markdown = client
            .get_tdp_markdown(tdp_name)
            .await
            .expect("Failed to get markdown");
        assert_eq!(markdown, "# Test markdown 1");

        // Cleanup
        drop(client);
        fs::remove_file(&db_filename).expect("Failed to delete database file");
        let _ = fs::remove_file(format!("{}-wal", db_filename));
        let _ = fs::remove_file(format!("{}-shm", db_filename));
    }

    #[tokio::test]
    async fn test_store_and_load_paper() {
        use data_structures::content::{
            Author, ContentItem, ContentType, FrontMatter, MarkdownTDP,
        };

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let db_filename = format!("test_paper_{}.db", timestamp);

        let config = SqliteConfig {
            filename: db_filename.clone(),
        };
        let client = SqliteClient::new(config);

        let league = data_structures::file::League::try_from("soccer_smallsize").unwrap();
        let team = data_structures::file::TeamName::new("RoboTeam Twente");
        let name =
            data_structures::file::TDPName::new(league.clone(), 2024, team.clone());
        let paper_lyt = name.get_paper_lyt();

        let tdp = MarkdownTDP {
            name,
            front_matter: FrontMatter {
                title: "Our Cool Robot".to_string(),
                authors: vec![
                    Author {
                        name: "Alice".to_string(),
                        affiliation: Some("University of Twente".to_string()),
                    },
                    Author {
                        name: "Bob".to_string(),
                        affiliation: None,
                    },
                ],
                institutions: vec!["University of Twente".to_string()],
                urls: vec!["https://example.com".to_string()],
                abstract_text: Some("This paper describes our cool robot.".to_string()),
            },
            content_items: vec![
                ContentItem {
                    content_seq: 0,
                    content_type: ContentType::Text,
                    depth: 1,
                    title: "Introduction".to_string(),
                    body: "We built a robot.".to_string(),
                    image_path: None,
                },
                ContentItem {
                    content_seq: 1,
                    content_type: ContentType::Image,
                    depth: 2,
                    title: "Robot Photo".to_string(),
                    body: "".to_string(),
                    image_path: Some("images/robot.png".to_string()),
                },
                ContentItem {
                    content_seq: 2,
                    content_type: ContentType::Table,
                    depth: 2,
                    title: "Performance Results".to_string(),
                    body: "| Metric | Value |\n| --- | --- |\n| Speed | 1.5 m/s |".to_string(),
                    image_path: None,
                },
            ],
            references: vec!["[1] Some Reference".to_string()],
            raw_markdown: "# Our Cool Robot\n\nFull markdown content here.".to_string(),
        };

        // Store
        client
            .store_paper(tdp.clone())
            .await
            .expect("Failed to store paper");

        // Test load_toc
        let toc = client
            .load_toc(paper_lyt.clone())
            .await
            .expect("Failed to load toc");
        assert_eq!(toc.len(), 3);
        assert_eq!(toc[0].content_seq, 0);
        assert_eq!(toc[0].title, "Introduction");
        assert_eq!(toc[0].content_type, ContentType::Text);
        assert_eq!(toc[0].depth, 1);
        assert_eq!(toc[1].content_seq, 1);
        assert_eq!(toc[1].title, "Robot Photo");
        assert_eq!(toc[1].content_type, ContentType::Image);
        assert_eq!(toc[2].content_seq, 2);
        assert_eq!(toc[2].title, "Performance Results");
        assert_eq!(toc[2].content_type, ContentType::Table);

        // Test load_content_item
        let item = client
            .load_content_item(paper_lyt.clone(), 0)
            .await
            .expect("Failed to load content item");
        assert_eq!(item.content_seq, 0);
        assert_eq!(item.title, "Introduction");
        assert_eq!(item.body, "We built a robot.");
        assert!(item.image_path.is_none());

        let item_img = client
            .load_content_item(paper_lyt.clone(), 1)
            .await
            .expect("Failed to load image content item");
        assert_eq!(item_img.image_path, Some("images/robot.png".to_string()));

        // Test load_content_item not found
        let not_found = client.load_content_item(paper_lyt.clone(), 99).await;
        assert!(not_found.is_err());

        // Test load_paper_abstract
        let abstract_text = client
            .load_paper_abstract(paper_lyt.clone())
            .await
            .expect("Failed to load abstract");
        assert_eq!(abstract_text, "This paper describes our cool robot.");

        // Test load_toc not found
        let toc_not_found = client.load_toc("nonexistent__lyti".to_string()).await;
        assert!(toc_not_found.is_err());

        // Cleanup
        drop(client);
        fs::remove_file(&db_filename).expect("Failed to delete database file");
        let _ = fs::remove_file(format!("{}-wal", db_filename));
        let _ = fs::remove_file(format!("{}-shm", db_filename));
    }
}
