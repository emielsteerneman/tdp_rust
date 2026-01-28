use data_structures::{intermediate::Chunk, paper::TDP};
use scirs2_text::{BasicNormalizer, BasicTextCleaner, preprocess::TextPreprocessor};
use tracing::info;

use crate::tdp_to_chunks;

pub fn process_text_to_words(text: &str) -> (Vec<String>, Vec<String>, Vec<String>) {
    let cleaner = TextPreprocessor::new(
        BasicNormalizer::new(true, true),
        BasicTextCleaner::new(true, true, true),
    );

    let cleaned = cleaner.process(text).unwrap();
    let words = cleaned
        .split_whitespace()
        .map(str::to_owned)
        .collect::<Vec<_>>();

    let ngram2 = words
        .windows(2)
        .map(|a_b: &[String]| format!("{} {}", a_b[0], a_b[1]))
        .collect::<Vec<_>>();

    let ngram3 = words
        .windows(3)
        .map(|a_b: &[String]| format!("{} {} {}", a_b[0], a_b[1], a_b[2]))
        .collect::<Vec<_>>();

    (words, ngram2, ngram3)
}

pub async fn load_all_tdp_jsons() -> Result<Vec<TDP>, Box<dyn std::error::Error>> {
    let mut tdps = Vec::new();
    let folder_path = "/home/emiel/projects/tdps_json";
    let files = std::fs::read_dir(folder_path)?;

    for entry in files {
        let path = entry?.path();

        if path.extension().and_then(|s| s.to_str()) != Some("json") {
            continue;
        }

        // Check if "smallsize" in the path
        // warn!("Don't forget to remove the 'smallsize' check");
        if !path.to_str().unwrap().contains("smallsize") {
            continue;
        }
        // if !path.to_str().unwrap().contains("2024") {
        //     continue;
        // }

        let content = tokio::fs::read_to_string(&path).await?;
        let tdp: TDP = serde_json::from_str(&content)?;
        tdps.push(tdp);
    }

    info!("Parsed {} tdps", tdps.len());

    Ok(tdps)
}

// TODO very annoying that this is async just because tdp_to_chunks is async
pub async fn load_all_chunks_from_tdps(
    tdps: &[TDP],
) -> Result<Vec<Chunk>, Box<dyn std::error::Error>> {
    let mut chunks = vec![];
    for tdp in tdps {
        chunks.append(&mut tdp_to_chunks(&tdp, None).await);
    }

    info!("Created {} chunks", chunks.len());

    Ok(chunks)
}

#[cfg(test)]
mod tests {
    use crate::utils::process_text_to_words;

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
