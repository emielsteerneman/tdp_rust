use serde::Deserialize;

use crate::structure::Text;

#[derive(Debug, Deserialize)]
pub struct Paragraph {
    pub sequence_id: u32,
    pub title: Text,
    pub sentences: Vec<Text>,
}

#[cfg(test)]
mod tests {
    use crate::structure::Paragraph;

    #[test]
    pub fn test_deserialize() {
        let json = r#"{"sequence_id": 1, "title": {"sequence_id": 0, "paragraph_id": 0, "raw": "1 Introduction", "processed": "introduction"}, "sentences": [{"sequence_id": 0, "paragraph_id": 0, "raw": "This paper is part of the qualification process to attend the RoboCup 2019 in Sydney Australia.", "processed": "paper part qualification process attend robocup 2019 sydney australia"}, {"sequence_id": 1, "paragraph_id": 0, "raw": "This paper is organized as follows.", "processed": "paper organized follows"}, {"sequence_id": 2, "paragraph_id": 0, "raw": "Section 2 provides the description of the Technical Institute of Applied Science HFTM.", "processed": "section provides description technical institute applied science hftm"}]}"#;

        let paragraph: Paragraph = serde_json::from_str(json).unwrap();

        println!("{}", paragraph.sequence_id);
        println!("{:?}", paragraph.title);
        println!("{}", paragraph.sentences.len());

        assert_eq!(paragraph.sequence_id, 1);
        assert_eq!(paragraph.title.raw, "1 Introduction");
        assert_eq!(paragraph.sentences.len(), 3);
    }
}
