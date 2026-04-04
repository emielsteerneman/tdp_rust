mod fastembed_client;
mod openai_client;
pub use fastembed_client::{FastEmbedConfig, FastembedClient};
pub use openai_client::{OpenAIClient, OpenAiConfig};

use async_openai::error::OpenAIError;
use data_structures::IDF;
use data_structures::text_utils::process_text_to_words;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;

#[derive(thiserror::Error, Debug)]
pub enum EmbedClientError {
    #[error("Internal client error: {0}")]
    Internal(String),
    #[error("Initialization error: {0}")]
    Initialization(String),
    #[error("Internal client error: {0}")]
    Any(#[from] anyhow::Error),
    #[error("OpenAI client error: {0}")]
    OpenAI(#[from] OpenAIError),
}

pub trait EmbedClient {
    fn embed_string<'a>(
        &'a self,
        string: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<f32>, EmbedClientError>> + Send + 'a>>;

    fn embed_strings<'a>(
        &'a self,
        strings: Vec<String>,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<Vec<f32>>, EmbedClientError>> + Send + 'a>>;

    fn embed_sparse(&self, text: &str, idf_map: &IDF) -> HashMap<u32, f32> {
        embed_sparse(text, idf_map)
    }
}

pub fn embed_sparse(text: &str, idf_map: &IDF) -> HashMap<u32, f32> {
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

pub fn extract_highlight_terms(query: &str, idf_map: &IDF, min_base_idf: f32) -> Vec<String> {
    let (ngram1, ngram2, ngram3) = process_text_to_words(query);

    let mut terms: Vec<(String, f32)> = ngram1
        .iter()
        .chain(ngram2.iter())
        .chain(ngram3.iter())
        .filter_map(|word| {
            // Skip short unigrams — 1-2 char words like "am", "do", "be" are
            // rarely meaningful and can match inside longer words (e.g. "am" in "team")
            let is_unigram = !word.contains(' ');
            if is_unigram && word.len() < 3 {
                return None;
            }
            let (_, weighted_idf) = idf_map.get(word)?;
            let ngram_weight = (word.matches(' ').count() as f32 + 1.0).min(3.0);
            let base_idf = weighted_idf / ngram_weight;
            if base_idf >= min_base_idf {
                Some((word.clone(), *weighted_idf))
            } else {
                None
            }
        })
        .collect();

    terms.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    terms.dedup_by(|a, b| a.0 == b.0);
    terms.into_iter().map(|(term, _)| term).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use data_structures::IDF;

    #[test]
    fn test_extract_highlight_terms_filters_by_base_idf() {
        // Simulate IDF map with:
        // - "robot" as a common unigram (base_idf 1.2, weighted 1.2) — below threshold
        // - "solenoid" as a rare unigram (base_idf 3.5, weighted 3.5) — above threshold
        // - "solenoid winder" as a rare bigram (base_idf 3.8, weighted 7.6) — above threshold
        let idf_map = IDF::from([
            ("robot".to_string(), (0, 1.2)),
            ("solenoid".to_string(), (1, 3.5)),
            ("solenoid winder".to_string(), (2, 7.6)),
        ]);

        let terms = extract_highlight_terms("robot solenoid winder", &idf_map, 1.5);

        assert!(terms.contains(&"solenoid".to_string()));
        assert!(terms.contains(&"solenoid winder".to_string()));
        assert!(!terms.contains(&"robot".to_string()));
    }

    #[test]
    fn test_extract_highlight_terms_sorted_by_weighted_idf_descending() {
        let idf_map = IDF::from([
            ("winder".to_string(), (0, 2.0)),
            ("solenoid".to_string(), (1, 3.5)),
            ("solenoid winder".to_string(), (2, 7.6)),
        ]);

        let terms = extract_highlight_terms("solenoid winder", &idf_map, 1.5);

        // "solenoid winder" (7.6) should come before "solenoid" (3.5), then "winder" (2.0)
        assert_eq!(terms[0], "solenoid winder");
        assert_eq!(terms[1], "solenoid");
        assert_eq!(terms[2], "winder");
    }

    #[test]
    fn test_extract_highlight_terms_empty_query() {
        let idf_map = IDF::new();
        let terms = extract_highlight_terms("", &idf_map, 1.5);
        assert!(terms.is_empty());
    }

    #[test]
    fn test_extract_highlight_terms_no_matches() {
        let idf_map = IDF::new();
        let terms = extract_highlight_terms("unknown words here", &idf_map, 1.5);
        assert!(terms.is_empty());
    }

    #[test]
    fn test_extract_highlight_terms_filters_short_unigrams() {
        // "am" has high IDF but only 2 chars — should be filtered
        // "do" same — 2 chars, filtered
        // "run" is 3 chars — should pass
        let idf_map = IDF::from([
            ("am".to_string(), (0, 4.0)),
            ("do".to_string(), (1, 3.8)),
            ("run".to_string(), (2, 3.5)),
        ]);

        let terms = extract_highlight_terms("am do run", &idf_map, 1.5);

        assert!(!terms.contains(&"am".to_string()));
        assert!(!terms.contains(&"do".to_string()));
        assert!(terms.contains(&"run".to_string()));
    }
}
