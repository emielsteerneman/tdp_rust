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

    let registry = configuration::helpers::build_team_registry_client(&config)
        .ok_or_else(|| anyhow::anyhow!(
            "Team registry not configured. Add [data_access.teams.sqlite] to config.toml"
        ))?;

    let team = data_structures::file::TeamName::new(&team_name);
    let code = registry.generate_team_code(&team.name).await
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    println!("Code for {}: {}", team.name, code);

    Ok(())
}
