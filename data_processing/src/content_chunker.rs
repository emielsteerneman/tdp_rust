use data_structures::content::{ContentType, MarkdownTDP};
use data_structures::intermediate::Chunk;

const MAX_CHUNK_CHARS: usize = 1500;
const OVERLAP_CHARS: usize = 200;

pub fn tdp_to_chunks(tdp: &MarkdownTDP) -> Vec<Chunk> {
    let mut chunks = Vec::new();

    for item in &tdp.content_items {
        match item.content_type {
            ContentType::Text => {
                let splits = split_text(&item.body, MAX_CHUNK_CHARS, OVERLAP_CHARS);
                for (chunk_seq, text) in splits.into_iter().enumerate() {
                    chunks.push(Chunk {
                        league_year_team_idx: tdp.name.get_filename(),
                        league: tdp.name.league.clone(),
                        year: tdp.name.year,
                        team: tdp.name.team_name.clone(),
                        content_seq: item.content_seq,
                        chunk_seq: chunk_seq as u32,
                        content_type: item.content_type.clone(),
                        title: item.title.clone(),
                        image_path: None,
                        text,
                        ..Default::default()
                    });
                }
            }
            ContentType::Table => {
                let text = if !item.title.is_empty() {
                    format!("{}\n{}", item.title, item.body)
                } else {
                    item.body.clone()
                };
                chunks.push(Chunk {
                    league_year_team_idx: tdp.name.get_filename(),
                    league: tdp.name.league.clone(),
                    year: tdp.name.year,
                    team: tdp.name.team_name.clone(),
                    content_seq: item.content_seq,
                    chunk_seq: 0,
                    content_type: item.content_type.clone(),
                    title: item.title.clone(),
                    image_path: None,
                    text,
                    ..Default::default()
                });
            }
            ContentType::Image => {
                let text = item.title.clone();
                chunks.push(Chunk {
                    league_year_team_idx: tdp.name.get_filename(),
                    league: tdp.name.league.clone(),
                    year: tdp.name.year,
                    team: tdp.name.team_name.clone(),
                    content_seq: item.content_seq,
                    chunk_seq: 0,
                    content_type: item.content_type.clone(),
                    title: item.title.clone(),
                    image_path: item.image_path.clone(),
                    text,
                    ..Default::default()
                });
            }
        }
    }

    chunks
}

fn split_text(text: &str, max_chars: usize, overlap_chars: usize) -> Vec<String> {
    if text.len() <= max_chars {
        return vec![text.to_string()];
    }

    let paragraphs: Vec<&str> = text.split("\n\n").collect();
    let mut result: Vec<String> = Vec::new();
    let mut current = String::new();

    for para in &paragraphs {
        if current.is_empty() {
            current.push_str(para);
        } else {
            // Check if adding this paragraph would exceed max_chars
            let candidate_len = current.len() + 2 + para.len(); // 2 for "\n\n"
            if candidate_len > max_chars {
                // Finalize current chunk
                result.push(current.clone());

                // Start new chunk with overlap from end of previous chunk
                let overlap_start = if current.len() > overlap_chars {
                    current.floor_char_boundary(current.len() - overlap_chars)
                } else {
                    0
                };
                current = format!("{}\n\n{}", &current[overlap_start..], para);
            } else {
                current.push_str("\n\n");
                current.push_str(para);
            }
        }
    }

    if !current.is_empty() {
        result.push(current);
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use data_structures::content::{ContentItem, ContentType, FrontMatter, MarkdownTDP};
    use data_structures::file::TDPName;

    fn make_tdp(content_items: Vec<ContentItem>) -> MarkdownTDP {
        let name: TDPName = "soccer_smallsize__2024__TestTeam__0".try_into().unwrap();
        MarkdownTDP {
            name,
            front_matter: FrontMatter::default(),
            content_items,
            references: vec![],
            raw_markdown: String::new(),
        }
    }

    #[test]
    fn test_short_text_no_split() {
        let short_text = "a".repeat(1500);
        let result = split_text(&short_text, MAX_CHUNK_CHARS, OVERLAP_CHARS);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], short_text);
    }

    #[test]
    fn test_long_text_splits_on_paragraph_boundary() {
        let para1 = "a".repeat(800);
        let para2 = "b".repeat(800);
        let text = format!("{}\n\n{}", para1, para2);

        let result = split_text(&text, MAX_CHUNK_CHARS, OVERLAP_CHARS);
        assert_eq!(result.len(), 2);
        assert!(result[0].contains(&para1));
        assert!(result[1].contains(&para2));
    }

    #[test]
    fn test_table_chunk() {
        let item = ContentItem {
            content_seq: 3,
            content_type: ContentType::Table,
            depth: 0,
            title: "Table 1: Results".to_string(),
            body: "| col1 | col2 |\n| a | b |".to_string(),
            image_path: None,
        };
        let tdp = make_tdp(vec![item]);
        let chunks = tdp_to_chunks(&tdp);

        assert_eq!(chunks.len(), 1);
        let c = &chunks[0];
        assert_eq!(c.content_type, ContentType::Table);
        assert_eq!(c.chunk_seq, 0);
        assert_eq!(c.content_seq, 3);
        assert_eq!(c.text, "Table 1: Results\n| col1 | col2 |\n| a | b |");
    }

    #[test]
    fn test_split_text_with_multibyte_chars() {
        // Greek rho (ρ) is 2 bytes in UTF-8. Fill text so the overlap boundary
        // lands inside a multi-byte character.
        let para1 = "a".repeat(700) + &"ρ".repeat(100); // 700 + 200 bytes = 900 bytes
        let para2 = "b".repeat(800);
        let text = format!("{}\n\n{}", para1, para2);

        // Should not panic
        let result = split_text(&text, MAX_CHUNK_CHARS, OVERLAP_CHARS);
        assert!(result.len() >= 2);
    }

    #[test]
    fn test_image_chunk() {
        let item = ContentItem {
            content_seq: 5,
            content_type: ContentType::Image,
            depth: 0,
            title: "Figure 1: Robot design".to_string(),
            body: String::new(),
            image_path: Some("images/fig1.png".to_string()),
        };
        let tdp = make_tdp(vec![item]);
        let chunks = tdp_to_chunks(&tdp);

        assert_eq!(chunks.len(), 1);
        let c = &chunks[0];
        assert_eq!(c.content_type, ContentType::Image);
        assert_eq!(c.chunk_seq, 0);
        assert_eq!(c.content_seq, 5);
        assert_eq!(c.text, "Figure 1: Robot design");
        assert_eq!(c.image_path, Some("images/fig1.png".to_string()));
    }
}
