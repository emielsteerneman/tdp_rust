use std::vec;

use data_access::{
    embed::{EmbedClient, FastembedClient},
    file::utilities::load_from_dir_all_tdp_json,
    vector::QdrantClient,
};
use data_processing::create_paragraph_chunks;

#[tokio::main]
async fn main() {
    println!("Running TDPs JSON to Qdrant importer");

    let mut embed_client = FastembedClient::new().unwrap();
    let mut vector_client = QdrantClient::new().unwrap();

    let tdps = load_from_dir_all_tdp_json("/home/emiel/projects/tdps_json").unwrap();
    println!("Loaded {} TDPs", tdps.len());

    let tdp = &tdps[0];

    for paragraph in &tdp.structure.paragraphs {
        println!("Processing paragraph : {}", paragraph.title.raw);
        let chunks = create_paragraph_chunks(paragraph.sentences.clone(), 500, 100);
        let texts = chunks
            .iter()
            .map(|c| c.text.as_ref())
            .collect::<Vec<&str>>();
        let embeddings = embed_client.embed_strings(texts).await.unwrap();

        // TODO insert into vector store

        println!("Generated {} embeddings", embeddings.len());
    }
}
