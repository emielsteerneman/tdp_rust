use data_structures::{file::TDPName, filter::Filter, intermediate::Chunk, paper::TDP};
use tracing::{debug, info};

use crate::create_sentence_chunks;

pub async fn load_all_tdp_jsons(
    folder_path: &str,
    filter: Option<Filter>,
) -> Result<Vec<TDP>, Box<dyn std::error::Error>> {
    let mut tdps = Vec::new();
    let files = std::fs::read_dir(folder_path)?;

    for entry in files {
        let path = entry?.path();

        if path.extension().and_then(|s| s.to_str()) != Some("json") {
            continue;
        }

        if let Some(f) = &filter {
            let tdp_name: TDPName = match path
                .file_stem()
                .and_then(|s| s.to_str())
                .and_then(|s| s.try_into().ok())
            {
                Some(v) => v,
                None => continue,
            };

            if !f.matches_tdp_name(&tdp_name) {
                continue;
            }
        }

        // Check if "smallsize" in the path
        // warn!("Don't forget to remove the 'smallsize' check");
        if !path.to_str().unwrap().contains("smallsize") {
            continue;
        }

        let content = tokio::fs::read_to_string(&path).await?;
        let tdp: TDP = serde_json::from_str(&content)?;
        tdps.push(tdp);
    }

    info!("Parsed {} tdps", tdps.len());

    Ok(tdps)
}

pub fn tdp_to_chunks(tdp: &TDP) -> Vec<Chunk> {
    let mut chunks = Vec::<Chunk>::new();

    for (i_paragraph, paragraph) in tdp.structure.paragraphs.iter().enumerate() {
        debug!("Processing paragraph: {}", paragraph.title.raw);
        let raw_chunks = create_sentence_chunks(paragraph.sentences.clone(), 500, 100);

        let paragraph_chunks = raw_chunks
            .into_iter()
            .map(|raw_chunk| raw_chunk.into_chunk(None, None, tdp.name.clone(), i_paragraph))
            .collect::<Vec<Chunk>>();

        chunks.extend(paragraph_chunks);
    }

    chunks
}

pub fn load_all_chunks_from_tdps(tdps: &[TDP]) -> Result<Vec<Chunk>, Box<dyn std::error::Error>> {
    let mut chunks = vec![];
    for tdp in tdps {
        chunks.append(&mut tdp_to_chunks(&tdp));
    }

    info!("Created {} chunks", chunks.len());

    Ok(chunks)
}

#[cfg(test)]
mod tests {
    use data_structures::text_utils::process_text_to_words;

    #[test]
    pub fn test_clean_text() {
        let text = "To the June competition, following goals are being sought: rework the remaining parts on the mechanical project such as making improvements on the coiling of the solenoid coil; stabilize the electronic project, including robot feedback and conclude the implementation of planning algorithms to be used in support decision making. Acknowledgements This research was partially supported by Fundacao Carlos Chagas Filho de Amparo a Pesquisa do Estado do Rio de Janeiro -FAPERJ(grant E-26/111.362/2012); Fundacao Ricardo Franco (FRF) and Fabrica de Material de Comunicacao e Eletronica (FMCE/IMBEL). The team also acknowledges the assistance of Mr. Carlos Beckhauser from FMCE. References 1. Alexandre Tadeu Rossini da Silva: Comportamento social cooperativo na realizacao de tarefas em ambientes dinamicos e competitivos. Instituto Militar de Engenharia, Rio de Janeiro (2006) 2.".to_string();

        let (words, ngram2, ngram3) = process_text_to_words(&text);
        print!("\nngram1: ");
        for word in words {
            print!("{word} | ");
        }
        print!("\nngram2: ");
        for word in ngram2 {
            print!("{word} | ");
        }
        print!("\nngram3: ");
        for word in ngram3 {
            print!("{word} | ");
        }
    }
}
