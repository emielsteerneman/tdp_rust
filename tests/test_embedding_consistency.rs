use std::error::Error;

use data_access::{
    embed::{EmbedClient, FastembedClient},
    vector::{QdrantClient, VectorClient},
};
use ndarray::Array1;
use ndarray_stats::DeviationExt;

#[tokio::test]
async fn test_embedding_consistency() -> Result<(), Box<dyn Error>> {
    let mut _vector_client = QdrantClient::new()?;
    let mut _embed_client = FastembedClient::new_with_custom_model()?;

    let mock_data = _vector_client.get_all_mock().await?;

    let mut vectors_in = Vec::<Vec<f32>>::new();
    let mut vectors_out = Vec::<Vec<f32>>::new();

    for mock_vector in mock_data.into_iter() {
        println!("\n\n");

        let vector2 = _embed_client.embed_string(&mock_vector.text)?;

        vectors_in.push(mock_vector.vector.clone());
        vectors_out.push(vector2.clone());

        let v1 = Array1::from(mock_vector.vector.clone());
        let v2 = Array1::from(vector2.clone());

        let dist = v1.l1_dist(&v2)?;
        let dist2 = v1.l2_dist(&v2)?;

        println!("{}", mock_vector.text);
        println!("Distance: {dist:.3} : {dist2:.3}");
        println!(
            "Python {:?}",
            mock_vector.vector.iter().take(5).collect::<Vec<&f32>>()
        );
        println!("Rust   {:?}", vector2.iter().take(5).collect::<Vec<&f32>>());
    }

    println!("\n");

    for v1 in vectors_in.iter() {
        for v2 in vectors_out.iter() {
            let v11 = Array1::from(v1.clone());
            let v22 = Array1::from(v2.clone());

            let dist = v11.l1_dist(&v22)?;
            let dist2 = v11.l2_dist(&v22)?;
            println!("Distance: {dist:.3} {dist2:.3}");
        }
        println!();
    }

    Ok(())
}
