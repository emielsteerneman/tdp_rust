use std::{error::Error, fs::read};

use data_access::embed::{EmbedClient, FastembedClient};
use fastembed::{InitOptionsUserDefined, TextEmbedding, TokenizerFiles, UserDefinedEmbeddingModel};

#[test]
pub fn test_custom_fastembed() -> Result<(), Box<dyn Error>> {
    let folder = "/home/emiel/Desktop/projects/fastembed_cache/models--qdrant--bge-base-en-v1.5-onnx-q/snapshots/738cad1c108e2f23649db9e44b2eab988626493b";

    let onnx_file = read(format!("{folder}/model_optimized.onnx"))?;

    let tokenizer_files = TokenizerFiles {
        config_file: read(format!("{folder}/config.json"))?,
        special_tokens_map_file: read(format!("{folder}/special_tokens_map.json"))?,
        tokenizer_config_file: read(format!("{folder}/tokenizer_config.json"))?,
        tokenizer_file: read(format!("{folder}/tokenizer.json"))?,
    };

    let udem = UserDefinedEmbeddingModel::new(onnx_file, tokenizer_files);
    let options = InitOptionsUserDefined::new();

    let mut model = TextEmbedding::try_new_from_user_defined(udem, options)?;
    let vec = model.embed(vec!["Hello World! What's up"], Some(1))?;
    println!("{:?}", vec[0].iter().take(5).collect::<Vec<&f32>>());

    let mut f = FastembedClient::new_with_custom_model()?;
    let vec = f.embed_string("Hello World! What's up")?;
    println!("{:?}", vec.iter().take(5).collect::<Vec<&f32>>());

    Ok(())
}
