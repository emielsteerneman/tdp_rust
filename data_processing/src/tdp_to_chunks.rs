use std::collections::HashMap;

use data_structures::{intermediate::Chunk, paper::TDP};
use tracing::{info, instrument};

use super::create_paragraph_chunks;

#[instrument("data_processing", skip_all)]
pub async fn tdp_to_chunks(tdp: &TDP) -> Vec<Chunk> {
    info!("\nLoaded TDP: {}", tdp.name.get_filename());

    let mut chunks = Vec::<Chunk>::new();

    let mut paragraph_chunks_map = HashMap::<String, (Vec<Chunk>, Vec<Vec<f32>>)>::new();

    for paragraph in &tdp.structure.paragraphs {
        let chunks_paragraph =
            create_paragraph_chunks(&tdp.name, paragraph.sentences.clone(), 500, 100);
    }

    /*
    for paragraph in &tdp.structure.paragraphs {
        let chunks_paragraph =
            create_paragraph_chunks(&tdp.name, paragraph.sentences.clone(), 500, 100);

        let texts_raw: Vec<String> = chunks_paragraph.raw();
        let embeddings: Vec<Vec<f32>> = embed_client
            .embed_strings(texts_raw.iter().map(|t| t.as_str()).collect())
            .await?;

        paragraph_chunks_map.insert(
            paragraph.title.raw.clone(),
            (chunks_paragraph.clone(), embeddings),
        );

        println!(
            "Section: {} - {} chunks",
            paragraph.title.raw,
            chunks_paragraph.len()
        );
        chunks.extend(chunks_paragraph);
        // println!("\n----------------------------------------\n");
    }

    let mut embeddings = Vec::with_capacity(chunks.len());
    for entry in &chunks {
        let embedding = embed_client.embed_string(&entry.text).await?;
        embeddings.push(embedding);
    }*/
    vec![]
}
