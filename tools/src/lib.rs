use data_structures::file::TeamName;

/// Validate that a team exists in the known teams list.
/// On failure, prints fuzzy suggestions to stderr and exits.
pub fn validate_team_name(input: &str, known_teams: &[TeamName]) -> TeamName {
    let team = TeamName::new(input);

    if known_teams.iter().any(|t| t.name == team.name) {
        return team;
    }

    eprintln!("Error: Team '{}' not found in the database.", team.name);

    let query = team.name_pretty.to_lowercase();
    let mut scored: Vec<_> = known_teams
        .iter()
        .map(|t| {
            let score = strsim::jaro_winkler(&query, &t.name_pretty.to_lowercase());
            (t, score)
        })
        .filter(|(_, score)| *score > 0.7)
        .collect();
    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    if !scored.is_empty() {
        eprintln!("\nDid you mean one of these?");
        for (t, _) in scored.iter().take(5) {
            eprintln!("  - {}", t.name_pretty);
        }
    }

    std::process::exit(1);
}

/// Extract a --flag value from CLI args.
pub fn get_arg(args: &[String], flag: &str) -> Option<String> {
    args.iter()
        .position(|a| a == flag)
        .and_then(|pos| args.get(pos + 1))
        .cloned()
}
