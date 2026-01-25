use data_processing::tdp_to_chunks;
use data_structures::{intermediate::Chunk, paper::TDP};
use std::{
    collections::{HashMap, HashSet},
    error::Error,
};
use tracing::info;

async fn load_all_tdp_jsons() -> Result<Vec<TDP>, Box<dyn Error>> {
    let mut tdps = Vec::new();
    let folder_path = "/home/emiel/projects/tdps_json";
    let files = std::fs::read_dir(folder_path)?;

    for entry in files {
        let path = entry?.path();

        if path.extension().and_then(|s| s.to_str()) != Some("json") {
            continue;
        }

        let content = tokio::fs::read_to_string(&path).await?;
        let tdp: TDP = serde_json::from_str(&content)?;
        tdps.push(tdp);
    }

    info!("Parsed {} tdps", tdps.len());

    Ok(tdps)
}

// TODO very annoying that this is async just because tdp_to_chunks is async
async fn load_all_chunks() -> Result<Vec<Chunk>, Box<dyn Error>> {
    let tdps = load_all_tdp_jsons().await?;

    // tracing::warn!("Don't forget to remove this tdps subset");
    // let tdps = tdps.into_iter().take(50).collect::<Vec<_>>();

    let mut chunks = vec![];
    for tdp in tdps {
        chunks.append(&mut tdp_to_chunks(&tdp, None).await);
    }

    info!("Created {} chunks", chunks.len());

    Ok(chunks)
}

fn calculate_idf(n_docs: u32, n_word: u32) -> f32 {
    // Formally, the formula for the inverse document frequency is:
    //  IDF(t) = Log( (N+1) / (DF(t)+1) ) + 1
    // Where:
    //  N = the number of documents. In this case, it equals the total number of chunks
    //  DF(t) = the number of documents (chunks) containing the term t
    // Note that if a term t shows up multiple times in a document, it still only counts as 1 occurence!
    // DF(t) counts the number of documents containing term t, not the total amount of occurences of term t!

    let n_docs = n_docs as f32;
    f32::log10((n_docs + 1.0) / (n_word as f32 + 1.0)) + 1.0
}

fn create_idf(texts: &[&str]) -> HashMap<String, (u32, f32)> {
    let n_docs = texts.len() as u32;
    let min_counts = vec![1, 5, 10];
    let ngram_weight = vec![1.0, 2.0, 3.0];

    // Step 1: Collect document frequency
    info!("Step 1: Collecting document frequency");
    let mut doc_counts: Vec<HashMap<String, u32>> =
        vec![HashMap::new(), HashMap::new(), HashMap::new()];

    for (i_text, text) in texts.iter().enumerate() {
        print!("\rProcessing text {i_text}/{}", texts.len());
        let (ngram1, ngram2, ngram3) = data_processing::utils::process_text_to_words(text);

        // We count the number of documents that contain the word, not the total word occurrences
        let ngrams_vec = [ngram1, ngram2, ngram3];

        for (i, ngram) in ngrams_vec.into_iter().enumerate() {
            let unique_words: HashSet<String> = ngram.into_iter().collect();
            for word in unique_words {
                *doc_counts[i].entry(word).or_insert(0) += 1;
            }
        }
    }

    // Step 2: Remove words below min_counts
    info!("Step 2: Removing words below min_counts");
    for i in 0..3 {
        doc_counts[i].retain(|_word, &mut count| count >= min_counts[i]);
    }

    // Step 3: Assign unique integer IDs and compute weighted IDF
    info!("Step 3: Assigning unique integer IDs and computing weighted IDF");
    let mut id_factory: u32 = 0;
    let mut idf_map: HashMap<String, (u32, f32)> = HashMap::new();

    for i in 0..3 {
        for (word, doc_count) in doc_counts[i].drain() {
            let id = id_factory;
            id_factory += 1;

            let idf = calculate_idf(n_docs, doc_count);
            let weighted_idf = idf * ngram_weight[i];

            idf_map.insert(word, (id, weighted_idf));
        }
    }

    idf_map
}

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn Error>> {
    let _stdout_subscriber = tracing_subscriber::fmt::init();

    /* Assumption: A "document" in my inverse document frequency is a chunk */
    let config = configuration::AppConfig::load_from_file("config.toml").unwrap();

    let chunks = load_all_chunks().await?;
    let texts: Vec<&str> = chunks.iter().map(|c| c.text.as_str()).collect();
    let idf_map = create_idf(&texts);

    // Printing some cute statistics
    let mut items = idf_map.clone().into_iter().collect::<Vec<_>>();
    let n_items = items.len();
    // Stupid lame weird sort needed because f32 does not implement Ord (f32 can be NaN)
    items.sort_by(|(_, (_, idf_a)), (_, (_, idf_b))| {
        idf_b
            .partial_cmp(&idf_a)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    for (word, (_, idf)) in items.into_iter().take(100) {
        println!("{word:<20}: {idf:.4}");
    }
    println!("Total amount of words: {n_items}");

    let metadata_client = configuration::helpers::load_any_metadata_client(&config);
    metadata_client.store_idf(idf_map).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::create_idf;

    #[test]
    fn test_create_idf() -> Result<(), Box<dyn std::error::Error>> {
        // This test assumes that the min_counts are 1, 2, 3

        let texts = vec![
            "I want to know more about computer vision algorithms",
            "I love computer vision algorithms",
            "Tell me more about computer vision algorithms",
        ];
        let idf_map = create_idf(&texts);

        assert!(!idf_map.is_empty());
        // 1-gram
        assert!(idf_map.contains_key("computer"));
        assert!(idf_map.contains_key("love"));
        // 2-gram
        assert!(idf_map.contains_key("computer vision"));
        assert!(idf_map.contains_key("about computer"));
        assert!(!idf_map.contains_key("i want"));
        // 3-gram
        assert!(idf_map.contains_key("computer vision algorithms"));
        assert!(!idf_map.contains_key("about computer vision"));

        for (word, (id, idf)) in idf_map.iter() {
            println!("{word:<20}: {id:<10}: {idf:.4}");
        }

        Ok(())
    }
}
