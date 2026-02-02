use data_structures::IDF;
use data_structures::text_utils::process_text_to_words;
use std::collections::{HashMap, HashSet};
use tracing::info;

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

pub fn create_idf(texts: &[&str], min_counts: &[u32; 3]) -> IDF {
    let n_docs = texts.len() as u32;
    let ngram_weight = vec![1.0, 2.0, 3.0];

    // Step 1: Collect document frequency
    info!("Step 1: Collecting document frequency");
    let mut doc_counts: Vec<HashMap<String, u32>> =
        vec![HashMap::new(), HashMap::new(), HashMap::new()];

    for (i_text, text) in texts.iter().enumerate() {
        print!("\rProcessing text {i_text}/{}", texts.len());
        let (ngram1, ngram2, ngram3) = process_text_to_words(text);

        // We count the number of documents that contain the word, not the total word occurrences
        let ngrams_vec: [Vec<String>; 3] = [ngram1, ngram2, ngram3];

        for (i, ngram) in ngrams_vec.into_iter().enumerate() {
            let unique_words: HashSet<String> = ngram.into_iter().collect();
            for word in unique_words {
                *doc_counts[i].entry(word).or_insert(0) += 1;
            }
        }
    }
    print!("\r");

    // Step 2:
    // Remove words below min_counts
    // Remove words that are just one character
    info!("Step 2.1: Removing terms below min_counts");
    for i in 0..3 {
        doc_counts[i].retain(|_word, &mut count| count >= min_counts[i]);
    }

    info!("Step 2.2: Removing terms that are just one character per word");
    for i in 0..3 {
        doc_counts[i].retain(|word, _| (1 + i * 2) < word.chars().count());
    }

    // Step 3: Assign unique integer IDs and compute weighted IDF
    info!("Step 3: Assigning unique integer IDs and computing weighted IDF");
    let mut id_factory: u32 = 0;
    let mut idf_map = IDF::new();

    /* ===================================================
    let mut total_doc_count = HashMap::<String, u32>::new();
    for i in 0..3 {
        for (word, doc_count) in doc_counts[i].iter() {
            *total_doc_count.entry(word.clone()).or_insert(0) += doc_count;
        }
    }
    // =================================================== */

    for i in 0..3 {
        for (word, doc_count) in doc_counts[i].drain() {
            let id = id_factory;
            id_factory += 1;

            let idf = calculate_idf(n_docs, doc_count);
            let weighted_idf = idf * ngram_weight[i];

            idf_map.insert(word, (id, weighted_idf));
        }
    }

    /* ==========================================
    let mut items = idf_map.clone().into_iter().collect::<Vec<_>>();
    let n_items = items.len();
    // Stupid lame weird sort needed because f32 does not implement Ord (f32 can be NaN)
    items.sort_by(|(_, (_, idf_a)), (_, (_, idf_b))| {
        idf_b
            .partial_cmp(&idf_a)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    for (word, (_, idf)) in items.into_iter().take(10000) {
        println!(
            "{word:<40}: {idf:.4} : {}",
            total_doc_count.get(&word).unwrap()
        );
    }
    println!("Total amount of words: {n_items}");
    println!(
        "path planning: {:?} -> {:?}",
        total_doc_count.get("path planning"),
        idf_map.get("path planning")
    );
    // ========================================== */

    idf_map
}

#[cfg(test)]
mod tests {
    use crate::text::create_idf;

    #[test]
    fn test_create_idf() -> Result<(), Box<dyn std::error::Error>> {
        let texts = vec![
            "I want to know more about computer vision algorithms",
            "I love computer vision algorithms",
            "Tell me more about computer vision algorithms",
        ];
        let idf_map = create_idf(&texts, &[1, 2, 3]);

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
