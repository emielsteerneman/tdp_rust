use data_structures::intermediate::Chunk;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}

#[allow(dead_code)]
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

#[allow(dead_code)]
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

#[allow(dead_code)]
fn euclidean_distance(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() || a.is_empty() {
        return f32::MAX;
    }

    let sum_sq: f32 = a.iter().zip(b.iter()).map(|(x, y)| (x - y) * (x - y)).sum();

    sum_sq.sqrt()
}

#[allow(dead_code)]
fn print_legend(sentences: &[Chunk]) {
    println!("\nLegend:");
    for (idx, entry) in sentences.iter().enumerate() {
        println!("{idx:>3}: {}", entry.text);
    }
}

#[allow(dead_code)]
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
