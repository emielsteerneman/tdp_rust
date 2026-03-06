use data_processing::{
    content_chunker::tdp_to_chunks,
    markdown_parser::load_all_markdown_tdps,
    text::create_idf,
};
use data_structures::filter::Filter;
use std::error::Error;

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn Error>> {
    /* Assumption: A "document" in the inverse document frequency is a chunk */

    let _stdout_subscriber = tracing_subscriber::fmt::init();
    let config = configuration::AppConfig::load_from_file("config.toml").unwrap();

    let mut filter = Filter::default();
    filter.add_league("soccer_smallsize".try_into()?);

    let tdps = load_all_markdown_tdps(&config.data_processing.tdps_markdown_root, Some(filter))?;
    let chunks: Vec<_> = tdps.iter().flat_map(tdp_to_chunks).collect();
    let texts: Vec<&str> = chunks.iter().map(|c| c.text.as_str()).collect();
    let idf_map = create_idf(&texts, &[1, 5, 10]);

    // Printing some cute statistics
    let mut items = idf_map.0.clone().into_iter().collect::<Vec<_>>();
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
mod tests {}
