use tools::{get_arg, validate_team_name};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().collect();

    let team_input = get_arg(&args, "--team")
        .ok_or_else(|| anyhow::anyhow!("Usage: set_team_metadata --team \"Team Name\" --key \"key\" --value \"value\""))?;
    let key = get_arg(&args, "--key")
        .ok_or_else(|| anyhow::anyhow!("--key is required"))?;
    let value = get_arg(&args, "--value")
        .ok_or_else(|| anyhow::anyhow!("--value is required"))?;

    let config_path = "config.toml";
    let config = configuration::AppConfig::load_from_file(config_path)
        .map_err(|e| anyhow::anyhow!("Failed to load config from {}: {}", config_path, e))?;

    let metadata_client = configuration::helpers::load_any_metadata_client(&config);
    let known_teams = metadata_client
        .load_teams()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to load teams: {}", e))?;

    let team = validate_team_name(&team_input, &known_teams);

    let registry = configuration::helpers::build_team_registry_client(&config)
        .ok_or_else(|| anyhow::anyhow!(
            "Team registry not configured. Add [data_access.teams.sqlite] to config.toml"
        ))?;

    // Load existing entries, upsert the key, write back
    let existing = registry.get_team_metadata(&team.name).await
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    let mut entries: Vec<(String, String)> = existing
        .into_iter()
        .filter(|e| e.key != key)
        .map(|e| (e.key, e.value))
        .collect();
    entries.push((key.clone(), value.clone()));

    registry.set_team_metadata(&team.name, entries).await
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    println!("Set {}={} for {}", key, value, team.name);

    Ok(())
}
