use std::collections::HashMap;

use data_structures::text_utils::process_text_to_words;
use strsim::jaro_winkler;

const MAX_MATCHES_PER_FRAGMENT: usize = 5;

pub fn match_terms(teams: Vec<String>, input: String, threshold: Option<f32>) -> Vec<String> {
    let threshold = threshold.unwrap_or(0.8) as f64;

    let (n1, n2, n3) = process_text_to_words(&input.to_lowercase());
    let query_fragments: Vec<String> = n1.into_iter().chain(n2).chain(n3).collect();

    // Pre-compute team fragments once
    let team_fragments: Vec<(String, Vec<String>)> = teams
        .iter()
        .map(|team| {
            let team_lower = team.to_lowercase();
            let (t_n1, t_n2, t_n3) = process_text_to_words(&team_lower);
            let mut frags: Vec<String> = t_n1.into_iter().chain(t_n2).chain(t_n3).collect();
            let stripped: String = team_lower.chars().filter(|c| c.is_alphanumeric()).collect();
            if !stripped.is_empty() {
                frags.push(stripped);
            }
            (team.clone(), frags)
        })
        .collect();

    // For each query fragment, count how many teams it matches above threshold.
    // If a fragment matches too many teams, it's too generic (e.g. "robot") — skip it.
    let specific_fragments: Vec<&String> = query_fragments
        .iter()
        .filter(|q_frag| {
            let match_count = team_fragments
                .iter()
                .filter(|(_, t_frags)| {
                    t_frags
                        .iter()
                        .any(|t_frag| jaro_winkler(t_frag, q_frag) > threshold)
                })
                .count();
            match_count > 0 && match_count <= MAX_MATCHES_PER_FRAGMENT
        })
        .collect();

    if specific_fragments.is_empty() {
        return vec![];
    }

    // Score teams using only specific fragments
    let mut scores = HashMap::<String, f64>::new();

    for (team, t_frags) in &team_fragments {
        let mut max_score = 0.0;

        for q_frag in &specific_fragments {
            for t_frag in t_frags {
                let score = jaro_winkler(t_frag, q_frag);
                if score > max_score {
                    max_score = score;
                }
            }
        }

        scores.insert(team.clone(), max_score);
    }

    let mut result: Vec<_> = scores.into_iter().collect();
    result.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    result
        .into_iter()
        .filter(|(_, score)| *score > threshold)
        .map(|(t, _)| t)
        .collect()
}
