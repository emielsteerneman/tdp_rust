use std::{fmt::Debug, ops::AddAssign};

use data_structures::paper::Text;
use num_traits::Zero;

pub struct Chunk {
    pub sentences: Vec<Text>,
    pub start: usize,
    pub end: usize,
    pub text: String,
}

impl Debug for Chunk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "\nChunk {{ start: {}, end: {}, text_length: {}, text: '{}' }}",
            self.start,
            self.end,
            self.text.len(),
            self.text
        )
    }
}

pub trait Recreate {
    fn recreate(&self) -> String;
}

impl Recreate for Vec<Chunk> {
    fn recreate(&self) -> String {
        if self.is_empty() {
            return String::new();
        }

        let mut reconstructed = String::new();
        reconstructed.push_str(&self[0].text);

        for i in 1..self.len() {
            let prev = &self[i - 1];
            let curr = &self[i];

            // Find overlap between previous chunk's end and current chunk's start
            // We compare suffix of prev.text with prefix of curr.text
            let max_overlap = prev.text.len().min(curr.text.len());
            let mut overlap_size = 0;

            for k in (1..=max_overlap).rev() {
                if prev.text.ends_with(&curr.text[..k]) {
                    overlap_size = k;
                    break;
                }
            }

            // Append only non-overlapping part
            reconstructed.push_str(&curr.text[overlap_size..]);
        }

        reconstructed.trim().to_string()
    }
}

pub fn create_paragraph_chunks(
    sentences: Vec<Text>,
    n_characters_per_chunk: usize,
    n_characters_overlap: usize,
) -> Vec<Chunk> {
    let lengths = sentences
        .iter()
        .map(|t| t.raw.len() + 2) // +2 for the ". " that will be added when joining
        .collect::<Vec<usize>>();

    let cumsum_end = cumsum(&lengths, |t| *t);
    let mut cumsum_start = Vec::with_capacity(cumsum_end.len() + 1);
    cumsum_start.push(0);
    cumsum_start.extend(&cumsum_end);

    let chunks = lengths_to_chunks(
        lengths.clone(),
        n_characters_per_chunk,
        n_characters_overlap,
    );

    let mut result = Vec::<Chunk>::new();

    for indices in chunks.clone() {
        let chunk_sentences = indices
            .iter()
            .map(|&i| sentences[i].clone())
            .collect::<Vec<Text>>();

        let start = cumsum_start[*indices.first().unwrap()];
        let end = cumsum_end[*indices.last().unwrap()];

        let text = chunk_sentences
            .iter()
            .map(|t| t.raw.as_str())
            .collect::<Vec<&str>>()
            .join(". ")
            + ". ";

        assert!(
            text.len() == end - start,
            "Text length does not match chunk length"
        );

        result.push(Chunk {
            sentences: chunk_sentences,
            start,
            end,
            text,
        });
    }

    result
}

pub fn lengths_to_chunks(
    lengths: Vec<usize>,
    n_characters_per_chunk: usize,
    n_characters_overlap: usize,
) -> Vec<Vec<usize>> {
    let cumsum_end = cumsum(&lengths, |t| *t);
    let cumsum_start = cumsum_end
        .iter()
        .map(|v| *v - lengths[0])
        .collect::<Vec<usize>>();
    let total_length: usize = lengths.iter().sum();
    let step = n_characters_per_chunk - n_characters_overlap;

    let mut i_end_prev: usize = 99999;

    let mut chunks = Vec::<Vec<usize>>::new();

    for offset in (0..total_length).step_by(step as usize) {
        // Find the first sentence that starts after the offset. Else, take all
        let mut i_start = argmax(&cumsum_start, |v| *v <= offset).unwrap_or(0);

        // Find the first sentence that ends after the offset + n_chars_per_group. Else, take all
        let i_end = argmin(&cumsum_end, |v| offset + n_characters_per_chunk <= *v)
            .unwrap_or(lengths.len() - 1);

        // Ensure that no sentence is skipped
        i_start = i_start.min(i_end_prev);

        // Ensure that the start and end are not the same
        if i_start == i_end {
            continue;
        }

        i_end_prev = i_end;

        chunks.push((i_start..i_end + 1).collect::<Vec<_>>());

        // if 1 < len(chunks) and len(chunks[-1].text) < n_chars_per_group * 0.33:
        //     # print(f"Merging last chunk with previous chunk")
        //     chunks[-2].text = chunks[-2].text[:chunks[-1].start-chunks[-2].start] + chunks[-1].text
        //     chunks[-2].end = chunks[-1].end
        //     chunks = chunks[:-1]
        // if 1 < chunks.len()
    }

    trim_duplicates_and_subsets(chunks)
}

fn trim_duplicates_and_subsets(chunks: Vec<Vec<usize>>) -> Vec<Vec<usize>> {
    let mut trimmed_chunks = Vec::<Vec<usize>>::new();

    for chunk in chunks.clone() {
        // Check if chunk is a subset of any existing chunk
        let is_subset = trimmed_chunks
            .iter()
            .any(|existing_chunk| chunk.iter().all(|item| existing_chunk.contains(item)));

        if !is_subset {
            trimmed_chunks.push(chunk);
        }
    }

    trimmed_chunks
}

fn cumsum<T, F, O>(vec: &Vec<T>, f: F) -> Vec<O>
where
    F: Fn(&T) -> O,
    O: AddAssign + Zero + Copy,
{
    let mut summations = Vec::<O>::with_capacity(vec.len());
    let mut sum: O = O::zero();

    for v in vec {
        sum += f(v);
        summations.push(sum);
    }

    summations
}

fn argmin<T, F>(vec: &Vec<T>, f: F) -> Option<usize>
where
    F: Fn(&T) -> bool,
{
    for (i, v) in vec.iter().enumerate() {
        if f(v) {
            return Some(i);
        }
    }

    None
}

fn argmax<T, F>(vec: &Vec<T>, f: F) -> Option<usize>
where
    F: Fn(&T) -> bool,
{
    for (i, v) in vec.iter().enumerate().rev() {
        if f(v) {
            return Some(i);
        }
    }

    None
}

#[cfg(test)]
mod tests {

    use crate::create_paragraph_chunks::Recreate;
    use data_structures::paper::Text;

    fn clean_text(text: &str) -> String {
        text.chars()
            .map(|c| match c {
                '\n' | '\r' | '\t' => ' ',
                _ if c.is_ascii_graphic() || c.is_ascii_whitespace() => c,
                _ => ' ', // replace non-ASCII or control chars
            })
            .collect::<String>()
            .split_whitespace()
            .collect::<Vec<&str>>()
            .join(" ")
    }

    fn split_into_sentences(text: &str) -> Vec<String> {
        text.split('.')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect()
    }

    #[test]
    fn test_create_paragraph_chunks_1() {
        // aaaa bbbb cccc dddd eeee ffff gggg hhhh iiii jjjj kkkk llll mmmm nnnn oooo pppp qqqq rrrr ssss tttt uuuu vvvv wwww xxxx yyyy zzzz
        let mut sentences = vec![];
        for c in b'a'..=b'z' {
            let s = std::str::from_utf8(&vec![c; 4]).unwrap().to_string();
            sentences.push(Text {
                raw: s.clone(),
                processed: s,
            });
        }

        let _chunks = super::create_paragraph_chunks(sentences, 10, 3);
        println!("{_chunks:?}");
    }

    #[test]
    fn test_create_paragraph_chunks_2() {
        let txt1 = Text {
            raw: "1".to_string(),
            processed: "1".to_string(),
        };
        let txt2 = Text {
            raw: "22".to_string(),
            processed: "22".to_string(),
        };
        let txt3 = Text {
            raw: "333".to_string(),
            processed: "333".to_string(),
        };
        let txt4 = Text {
            raw: "4444".to_string(),
            processed: "4444".to_string(),
        };
        let txt5 = Text {
            raw: "55555".to_string(),
            processed: "55555".to_string(),
        };
        let txt6 = Text {
            raw: "666666".to_string(),
            processed: "666666".to_string(),
        };
        let txt7 = Text {
            raw: "7777777".to_string(),
            processed: "7777777".to_string(),
        };
        let txt8 = Text {
            raw: "88888888".to_string(),
            processed: "88888888".to_string(),
        };
        let txt9 = Text {
            raw: "999999999".to_string(),
            processed: "999999999".to_string(),
        };
        let sentences = vec![txt1, txt2, txt3, txt4, txt5, txt6, txt7, txt8, txt9];

        let _chunks = super::create_paragraph_chunks(sentences, 5, 1);

        println!("{_chunks:?}");

        let reconstructed = _chunks.recreate();
        println!("\nReconstructed: '{}'", reconstructed);
    }

    #[test]
    fn test_chunk_and_reconstruct() {
        let story = r#"In the misty mountains of Thrynn lived a dragon named Sargath, feared across kingdoms not for his fire but for his fashion. His wings shimmered like black silk, and his cavern was no ordinary lair—it was a cathedral of socks. Piles upon piles of them, neatly sorted by color, texture, and weave. Where other dragons slept on treasure, Sargath slumbered on a mound of mismatched masterpieces.

His collection was legendary. There were socks woven from phoenix feathers that never burned, socks stitched from mermaid silk that shimmered like moonlight, and socks knitted by frost giants, forever cool to the touch. One drawer was dedicated to stormweave socks that crackled faintly with static lightning. Another contained socks from the Moon Bazaar, dyed in colors no mortal eye could fully name.
w
When knights and thieves came seeking gold, they found instead a dragon who roared about craftsmanship and heel reinforcement. Those who insulted his taste were promptly chased off in a torrent of laundry-scented smoke. Those who admired his precision were often gifted a single sock—never a pair, of course.

Yet for all his splendor, Sargath was lonely. No one truly understood the art of sock curation. Until one day, a young tailor arrived from the valley below. She carried no sword, only a bundle wrapped in linen. Inside was a single sock, woven from starlight thread and dew collected from midnight flowers. The dragon’s great eyes widened. He offered half his hoard for the pair.

“There is no pair,” the tailor said calmly. “The second one has yet to be made.”

Sargath paused, unsure whether to roar or weep. The tailor smiled and added, “But I could make it here, if you’d let me.”

And so she stayed. The sound of her loom joined the mountain winds, and for the first time in centuries, laughter echoed through the dragon’s halls. Together they crafted socks that sang when worn, socks that told stories, socks that could only exist through shared hands.

When travelers now speak of the dragon of Thrynn, they tell not of his greed, but of his gallery—a living museum where scales and silk met, and where a lonely dragon learned that beauty was best when made, not hoarded."#;

        let clean = clean_text(story);

        let sentences = split_into_sentences(&clean);

        let texts = sentences
            .iter()
            .map(|s| Text {
                raw: s.to_string(),
                processed: s.to_string(),
            })
            .collect::<Vec<Text>>();

        let chunks = super::create_paragraph_chunks(texts, 500, 100);

        println!("\n\nChunks:\n{:?}", chunks);

        let reconstructed = chunks.recreate();

        println!("\n\nCleaned Text:\n{}", clean);
        println!("\nReconstructed Text:\n{}", reconstructed);

        assert_eq!(clean, reconstructed);
    }
}
