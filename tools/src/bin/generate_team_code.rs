use tools::{get_arg, validate_team_name};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().collect();

    let team_input = get_arg(&args, "--team")
        .ok_or_else(|| anyhow::anyhow!("Usage: generate_team_code --team \"Team Name\""))?;

    let config_path = "config.toml";
    let config = configuration::AppConfig::load_from_file(config_path)
        .map_err(|e| anyhow::anyhow!("Failed to load config from {}: {}", config_path, e))?;

    let metadata_client = configuration::helpers::load_any_metadata_client(&config);
    let known_teams = metadata_client
        .load_teams()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to load teams: {}", e))?;

    let team = validate_team_name(&team_input, &known_teams);

    let registry = configuration::helpers::build_registry_client(&config)
        .ok_or_else(|| anyhow::anyhow!(
            "Registry not configured. Add [data_access.registry.sqlite] to config.toml"
        ))?;

    let code = registry.generate_team_code(&team.name).await
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    println!("Code for {}: {}", team.name, code);

    Ok(())
}
