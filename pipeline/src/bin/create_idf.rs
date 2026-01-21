use data_processing::tdp_to_chunks;
use std::collections::HashMap;
use std::error::Error;

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn Error>> {
    /* Assumption: A "document" in my inverse document frequency is a chunk */
    let config = configuration::AppConfig::load_from_file("config.toml").unwrap();

    let metadata_client = configuration::helpers::load_any_metadata_client(&config);

    let mut vocabulary = HashMap::<String, u32>::new();
    let mut doc_freq = HashMap::<String, u32>::new();

    let tdps = load_all_tdp_jsons().await?;
    let tdps = tdps.iter().take(50).collect::<Vec<_>>();

    let mut all_chunks = vec![];
    for tdp in &tdps {
        let mut chunks = tdp_to_chunks(tdp, None).await;
        all_chunks.append(&mut chunks);
    }

    println!(
        "Parsed {} papers and retrieved {} chunks",
        tdps.len(),
        all_chunks.len()
    );

    let mut id_factory = 0;
    for chunk in &all_chunks {
        // println!("\n\n{}", chunk.text);
        let words = data_processing::utils::process_text_to_words(chunk.text.clone());
        // println!("{words:?}");

        for word in words {
            vocabulary.entry(word.clone()).or_insert_with(|| {
                let id = id_factory;
                id_factory += 1;
                id
            });

            doc_freq.entry(word).and_modify(|c| *c += 1).or_insert(1);
        }
    }

    let mut idf_map = HashMap::<String, f32>::new();
    for (word, count) in doc_freq.clone() {
        let n = all_chunks.len() as f32;
        let idf = f32::log10((n + 1.0) / (count as f32 + 1.0)) + 1.0;
        idf_map.insert(word, idf);
    }

    metadata_client.store_idf(idf_map.clone()).await?;

    let mut items: Vec<_> = doc_freq.iter().collect();
    items.sort_by_key(|&(_, count)| std::cmp::Reverse(count));

    for (word, _) in items.into_iter().take(20) {
        let count = doc_freq.get(word).unwrap();
        let idf = idf_map.get(word).unwrap();
        println!("{word:<20}: {count:<10} {idf:<10}");
    }

    Ok(())
}

use data_structures::paper::TDP;

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

    Ok(tdps)
}
