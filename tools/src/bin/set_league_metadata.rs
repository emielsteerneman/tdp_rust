use tools::get_arg;
use data_structures::file::League;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().collect();

    let league_input = get_arg(&args, "--league")
        .ok_or_else(|| anyhow::anyhow!("Usage: set_league_metadata --league \"Soccer SmallSize\" --key \"key\" --value \"value\""))?;
    let key = get_arg(&args, "--key")
        .ok_or_else(|| anyhow::anyhow!("--key is required"))?;
    let value = get_arg(&args, "--value")
        .ok_or_else(|| anyhow::anyhow!("--value is required"))?;

    let league = League::try_from(league_input.as_str())
        .map_err(|_| anyhow::anyhow!("Unknown league: '{}'. Use machine name (e.g. soccer_smallsize) or pretty name (e.g. Soccer SmallSize).", league_input))?;

    let config_path = "config.toml";
    let config = configuration::AppConfig::load_from_file(config_path)
        .map_err(|e| anyhow::anyhow!("Failed to load config from {}: {}", config_path, e))?;

    let registry = configuration::helpers::build_registry_client(&config)
        .ok_or_else(|| anyhow::anyhow!(
            "Registry not configured. Add [data_access.registry.sqlite] to config.toml"
        ))?;

    let league_name = league.name();

    // Load existing entries, upsert the key, write back
    let existing = registry.get_league_metadata(league_name).await
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    let mut entries: Vec<(String, String)> = existing
        .into_iter()
        .filter(|e| e.key != key)
        .map(|e| (e.key, e.value))
        .collect();
    entries.push((key.clone(), value.clone()));

    registry.set_league_metadata(league_name, entries).await
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    println!("Set {}={} for league {}", key, value, league.name_pretty());

    Ok(())
}
