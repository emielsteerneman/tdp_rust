use std::collections::HashMap;

use data_processing::utils::process_text_to_words;

fn similarity(a: &str, b: &str) -> i32 {
    let abc = "abcdefghijklmnopqrstuvwxyz";
    let count_a: Vec<u32> = abc
        .chars()
        .map(|c| a.chars().filter(|x| *x == c).count() as u32)
        .collect();
    let count_b: Vec<u32> = abc
        .chars()
        .map(|c| b.chars().filter(|x| *x == c).count() as u32)
        .collect();

    let mut score = 0i32;
    for (ca, cb) in count_a.into_iter().zip(count_b.into_iter()) {
        if ca != 0 && cb != 0 {
            score += if ca == cb { 1 } else { -1 };
        }
    }
    score
}

pub fn match_names(teams: Vec<String>, input: String) -> Vec<String> {
    let (n1, n2, n3) = process_text_to_words(&input);
    let inputs = n1.into_iter().chain(n2).chain(n3).collect::<Vec<_>>();

    let mut scores = HashMap::<String, i32>::new();

    for input in &inputs {
        for team in &teams {
            let score = similarity(team, input);
            println!("   {team:20} | {input:20} | {score}");
            scores
                .entry(team.clone())
                .and_modify(|v| *v = score.max(*v))
                .or_insert(score);
        }
    }

    for team in &teams {
        match scores.get(team) {
            Some(score) => println!("{team:40} {score}"),
            None => println!("{team:40} /"),
        }
    }

    vec![]

    // scores.sort();
    // scores.reverse();

    // for (score, team) in scores.iter() {
    //     println!("{score} : {team}");
    // }

    // scores
    //     .into_iter()
    //     .filter(|(s, _)| 0 < *s)
    //     .map(|(_, t)| t)
    //     .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_match_names() {
        let teams = vec![
            "RoboTeam Twente".to_string(),
            "Er-Force".to_string(),
            "TIGERs Mannheim".to_string(),
            "RoboCIN".to_string(),
            "RoboIME".to_string(),
        ];

        let scores = match_names(teams, "Twente".to_string());
        println!("Matches for \"Twente\": {scores:?}");
    }

    #[test]
    fn test_match_names_specific() {
        let teams = vec![
            "Er-Force".to_string(),
            "RFC Cambridge".to_string(),
            "Delft Mercurians m".to_string(),
            "Warthog Robotics".to_string(),
        ];
        let query = "battery capacity er-force";
        match_names(teams, query.to_string());
    }

    #[test]
    fn test_similarity() {
        let score = similarity("abc", "def");
        assert_eq!(score, 0);

        let score = similarity("aab", "abb");
        assert_eq!(score, -2);

        let score = similarity("ab", "ab");
        assert_eq!(score, 2);
    }
}
