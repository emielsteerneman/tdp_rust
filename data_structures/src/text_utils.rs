use scirs2_text::{BasicNormalizer, BasicTextCleaner, preprocess::TextPreprocessor};

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
