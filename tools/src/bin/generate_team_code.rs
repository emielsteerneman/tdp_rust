#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().collect();

    let team_name = if let Some(pos) = args.iter().position(|a| a == "--team") {
        args.get(pos + 1)
            .ok_or_else(|| anyhow::anyhow!("--team requires a value"))?
            .clone()
    } else {
        anyhow::bail!("Usage: generate_team_code --team \"Team Name\"");
    };

    let config_path = "config.toml";
    let config = configuration::AppConfig::load_from_file(config_path)
        .map_err(|e| anyhow::anyhow!("Failed to load config from {}: {}", config_path, e))?;

    // Verify team exists in the metadata database
    let metadata_client = configuration::helpers::load_any_metadata_client(&config);
    let known_teams = metadata_client
        .load_teams()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to load teams: {}", e))?;

    let team = data_structures::file::TeamName::new(&team_name);

    if !known_teams.iter().any(|t| t.name == team.name) {
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

    let registry = configuration::helpers::build_team_registry_client(&config)
        .ok_or_else(|| anyhow::anyhow!(
            "Team registry not configured. Add [data_access.teams.sqlite] to config.toml"
        ))?;

    let code = registry.generate_team_code(&team.name).await
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    println!("Code for {}: {}", team.name, code);

    Ok(())
}
