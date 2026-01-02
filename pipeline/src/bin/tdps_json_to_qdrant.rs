use data_access::file::utilities::load_from_dir_all_tdp_json;
use data_processing::create_paragraph_chunks;

#[tokio::main]
async fn main() {
    println!("Running TDPs JSON to Qdrant importer");

    let config = configuration::AppConfig::load_from_file("config.toml").unwrap();

    let mut embed_client = configuration::helpers::load_any_embed_client(&config);
    let mut vector_client = configuration::helpers::load_any_vector_client(&config).await;

    let tdps = load_from_dir_all_tdp_json("/home/emiel/projects/tdps_json").unwrap();
    println!("Loaded {} TDPs", tdps.len());

    let tdp = &tdps[0];

    for paragraph in &tdp.structure.paragraphs {
        println!("Processing paragraph : {}", paragraph.title.raw);
        let chunks = create_paragraph_chunks(&tdp.name, paragraph.sentences.clone(), 500, 100);
        let texts = chunks
            .iter()
            .map(|c| c.text.as_ref())
            .collect::<Vec<&str>>();
        let embeddings = embed_client.embed_strings(texts).await.unwrap();

        for (chunk, embedding) in chunks.iter().zip(embeddings.iter()) {
            chunk.embedding = Some(embedding.clone());
            vector_client.store_chunk(chunk.clone()).await.unwrap();
        }

        println!("Generated {} embeddings", embeddings.len());
    }
}
