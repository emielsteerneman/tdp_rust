use std::collections::HashMap;

use data_access::embed::{self, EmbedClient, FastembedClient};
use data_processing::{Recreate, create_paragraph_chunks};
use data_structures::{intermediate::Chunk, paper::TDP};
use serde_json;

struct SentenceEntry {
    paragraph_title: String,
    text: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // let mut embed_client = FastembedClient::new()?;
    let mut embed_client = embed::OpenAIClient::new();

    let files =
        std::fs::read_dir("/home/emiel/projects/tdps_json").expect("Failed to read directory");

    for file in files {
        let file = file.expect("Failed to read file entry");
        let path = file.path();
        let content = std::fs::read_to_string(&path).expect("Failed to read file content");
        let tdp: TDP = serde_json::from_str(&content).expect("Failed to parse JSON");
        println!("\nLoaded TDP: {}", tdp.name.get_filename());

        let mut chunks = Vec::<Chunk>::new();

        let mut paragraph_chunks_map = HashMap::<String, (Vec<Chunk>, Vec<Vec<f32>>)>::new();

        for paragraph in &tdp.structure.paragraphs {
            let chunks_paragraph = create_paragraph_chunks(paragraph.sentences.clone(), 500, 100);

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
        }

        let matrix = build_similarity_matrix(&embeddings);

        // Create map to export to json with chunks and matrix
        let _output = serde_json::json!({
            "map": &paragraph_chunks_map,
            "similarity_matrix": &matrix,
        })
        .to_string();

        let output_path = format!(
            "/home/emiel/projects/tdp_similarity_matrices/{}_similarity_matrix.json",
            tdp.name.get_filename()
        );
        std::fs::write(&output_path, _output).unwrap();
        println!("Wrote similarity matrix to: {}", output_path);

        print_legend(&chunks);
        print_similarity_matrix(&matrix);

        break;
    }

    Ok(())
}

fn build_similarity_matrix(embeddings: &[Vec<f32>]) -> Vec<Vec<f32>> {
    let len = embeddings.len();
    if len == 0 {
        return Vec::new();
    }

    let mut matrix = vec![vec![0.0; len]; len];
    for i in 0..len {
        matrix[i][i] = 1.0;
        for j in (i + 1)..len {
            let similarity = normalized_cosine_similarity(&embeddings[i], &embeddings[j]);
            matrix[i][j] = similarity;
            matrix[j][i] = similarity;
        }
    }
    matrix
}

fn normalized_cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }

    let dot = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum::<f32>();
    let norm_a = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b = b.iter().map(|x| x * x).sum::<f32>().sqrt();

    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }

    let cosine = (dot / (norm_a * norm_b)).clamp(-1.0, 1.0);
    ((cosine + 1.0) / 2.0).clamp(0.0, 1.0)
}

fn euclidean_distance(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() || a.is_empty() {
        return f32::MAX;
    }

    let sum_sq: f32 = a.iter().zip(b.iter()).map(|(x, y)| (x - y) * (x - y)).sum();

    sum_sq.sqrt()
}

fn print_legend(sentences: &[Chunk]) {
    println!("\nLegend:");
    for (idx, entry) in sentences.iter().enumerate() {
        println!("{idx:>3}: {}", entry.text);
    }
}

fn print_similarity_matrix(matrix: &[Vec<f32>]) {
    if matrix.is_empty() {
        println!("No similarity matrix to display.");
        return;
    }

    println!("\nSentence Similarity Matrix (1.00 = very similar, 0.00 = very different):");
    print!("     ");
    for idx in 0..matrix.len() {
        print!(" {:>5}", idx);
    }
    println!();

    for (row_idx, row) in matrix.iter().enumerate() {
        print!("{row_idx:>3} ");
        for value in row {
            print!(" {:>5.2}", value);
        }
        println!();
    }
    println!();
}
