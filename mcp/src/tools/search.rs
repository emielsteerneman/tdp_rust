use crate::state::AppState;
use data_processing::utils::process_text_to_words;
use data_structures::IDF;
use rmcp::schemars::JsonSchema;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SearchArgs {
    pub query: String,
    pub limit: Option<u64>,
}

pub async fn search(state: &AppState, args: SearchArgs) -> anyhow::Result<String> {
    let limit = args.limit.unwrap_or(5);
    let query = args.query.trim();
    if query.is_empty() {
        return Ok("Empty query provided.".to_string());
    }

    // 1. Generate sparse embedding
    // We prioritize sparse search as per the reference implementation
    let sparse = embed_sparse(query, &state.idf_map);

    // 2. Search
    // Note: Dense search is currently disabled in reference implementation (pipeline/src/bin/initialize.rs)
    // so we pass None for dense embedding.
    let results = state
        .vector_client
        .search_chunks(None, Some(sparse), limit)
        .await?;

    if results.is_empty() {
        return Ok("No results found.".to_string());
    }

    let mut output = String::new();
    for (i, (chunk, score)) in results.iter().enumerate() {
        use std::fmt::Write;
        writeln!(
            &mut output,
            "[{}] Score: {:.4} - {} - {} - {}\n{}\n",
            i + 1,
            score,
            chunk.league.name_pretty,
            chunk.team.name_pretty,
            chunk.year,
            chunk.text
        )?;
    }

    Ok(output)
}

fn embed_sparse(text: &str, idf_map: &IDF) -> HashMap<u32, f32> {
    let mut map = HashMap::new();

    let (ngram1, ngram2, ngram3) = process_text_to_words(text);
    let iter = ngram1.iter().chain(ngram2.iter()).chain(ngram3.iter());

    for word in iter {
        if let Some((id, idf)) = idf_map.get(word) {
            *map.entry(*id).or_insert(0.0) += idf;
        }
    }

    map
}
