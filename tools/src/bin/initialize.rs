use data_processing::{
    content_chunker::tdp_to_chunks,
    embed::embed_chunks,
    markdown_parser::load_all_markdown_tdps,
    text::create_idf,
};
use data_structures::{IDF, embed_type::EmbedType, filter::Filter};
use tracing::info;

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _stdout_subscriber = tracing_subscriber::fmt::init();
    let config = configuration::AppConfig::load_from_file("config.toml").unwrap();

    let embed_client = configuration::helpers::load_any_embed_client(&config);
    let vector_client = configuration::helpers::load_any_vector_client(&config).await?;
    let metadata_client = configuration::helpers::load_any_metadata_client(&config);

    let mut filter = Filter::default();
    for year in 2015..2026 {
        filter.add_year(year);
    }
    filter.add_league(data_structures::file::League::new(
        "soccer".to_string(),
        "smallsize".to_string(),
        None,
    ));

    /* Step 1 : Load markdown TDPs */
    info!("Loading markdown TDPs");
    let tdps =
        load_all_markdown_tdps(&config.data_processing.tdps_markdown_root, Some(filter))?;
    info!("Loaded {} TDPs", tdps.len());

    /* Step 2 : Create chunks */
    info!("Creating chunks");
    let mut chunks: Vec<_> = tdps.iter().flat_map(tdp_to_chunks).collect();
    info!("Created {} chunks", chunks.len());

    /* Step 3 : Store paper metadata */
    info!("Storing paper metadata");
    for tdp in tdps {
        metadata_client.store_paper(tdp).await?;
    }

    /* Step 4 : Create and store IDF */
    info!("Creating IDF");
    let texts: Vec<&str> = chunks.iter().map(|c| c.text.as_str()).collect();
    let idf_map = create_idf(&texts, &[1, 5, 10]);
    metadata_client.store_idf(idf_map.clone()).await?;

    /* Step 5 : Create embeddings */
    info!("Creating embeddings");
    embed_chunks(
        &mut chunks,
        &*embed_client,
        EmbedType::HYBRID,
        Some(&idf_map),
    )
    .await?;

    /* Step 6 : Store chunks */
    info!("Storing chunks");
    for chunk in chunks {
        vector_client.store_chunk(chunk).await?;
    }

    Ok(())
}

pub fn print_idf_statistics(idf_map: &IDF) {
    let mut items = idf_map.0.clone().into_iter().collect::<Vec<_>>();
    let n_items = items.len();
    // Stupid lame weird sort needed because f32 does not implement Ord (f32 can be NaN)
    items.sort_by(|(_, (_, idf_a)), (_, (_, idf_b))| {
        idf_b
            .partial_cmp(&idf_a)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    for (word, (_, idf)) in items.into_iter().take(25) {
        println!("{word:<20}: {idf:.4}");
    }
    println!("Total amount of words: {n_items}");
}

#[cfg(test)]
mod tests {
    use data_access::embed::embed_sparse;
    use data_structures::{IDF, intermediate::Chunk};
    use std::collections::HashMap;

    #[test]
    fn test_sparse_embedding() {
        let idf_map = IDF::from([
            ("hello".to_string(), (0, 1.0)),
            ("world".to_string(), (1, 2.0)),
            ("hello world".to_string(), (2, 3.0)),
        ]);
        let chunk = Chunk {
            text: "hello world. I am world".to_string(),
            ..Default::default()
        };
        let sparse = embed_sparse(&chunk.text, &idf_map);
        assert_eq!(sparse, HashMap::from([(0, 1.0), (1, 4.0), (2, 3.0)]));
    }
}
