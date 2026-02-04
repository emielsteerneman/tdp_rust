use std::collections::HashMap;

use data_structures::text_utils::process_text_to_words;
use strsim::jaro_winkler;

pub fn match_terms(teams: Vec<String>, input: String) -> Vec<String> {
    let (n1, n2, n3) = process_text_to_words(&input.to_lowercase());
    let query_fragments = n1.into_iter().chain(n2).chain(n3).collect::<Vec<_>>();

    let mut scores = HashMap::<String, f64>::new();

    for team in &teams {
        // Process team name into fragments to handle multi-word teams and punctuation
        let team_lower = team.to_lowercase();
        let (t_n1, t_n2, t_n3) = process_text_to_words(&team_lower);

        let mut team_fragments = t_n1.into_iter().chain(t_n2).chain(t_n3).collect::<Vec<_>>();

        // Add "stripped" version of the team name (only alphanumeric)
        // This helps match "erforce" to "Er-Force"
        let stripped_team: String = team_lower.chars().filter(|c| c.is_alphanumeric()).collect();
        if !stripped_team.is_empty() {
            team_fragments.push(stripped_team);
        }

        // Find best match between any query fragment and any team fragment
        let mut max_score = 0.0;

        for q_frag in &query_fragments {
            for t_frag in &team_fragments {
                let score = jaro_winkler(t_frag, q_frag);
                if score > max_score {
                    max_score = score;
                }
            }
        }

        scores.insert(team.clone(), max_score);
    }

    let mut result: Vec<_> = scores.into_iter().collect();
    // Sort descending
    result.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    result
        .into_iter()
        .filter(|(_, score)| *score > 0.9)
        .map(|(t, _)| t)
        .collect()
}
