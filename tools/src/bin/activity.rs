use std::collections::HashMap;
use std::sync::Arc;

use data_access::activity::ActivityClient;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config_path = "config.toml";
    let config = configuration::AppConfig::load_from_file(config_path)
        .map_err(|e| anyhow::anyhow!("Failed to load config from {}: {}", config_path, e))?;

    let client: Arc<dyn ActivityClient + Send + Sync> =
        configuration::helpers::load_activity_client(&config)
            .ok_or_else(|| anyhow::anyhow!("No activity client configured in config.toml"))?;

    let args: Vec<String> = std::env::args().collect();
    let subcommand = args.get(1).map(|s| s.as_str()).unwrap_or("summary");

    match subcommand {
        "summary" => summary(&client, &args[2..]).await?,
        "recent" => recent(&client, &args[2..]).await?,
        "agents" => agents(&client, &args[2..]).await?,
        _ => {
            eprintln!("Usage: activity <command>");
            eprintln!();
            eprintln!("Commands:");
            eprintln!("  summary [--since DATE]   Event counts by type and source");
            eprintln!("  recent  [--limit N]      Most recent events");
            eprintln!("  agents  [--since DATE]   User-agent breakdown (scraper detection)");
        }
    }

    Ok(())
}

fn parse_flag(args: &[String], flag: &str) -> Option<String> {
    args.iter()
        .position(|a| a == flag)
        .and_then(|i| args.get(i + 1).cloned())
}

async fn summary(
    client: &Arc<dyn ActivityClient + Send + Sync>,
    args: &[String],
) -> anyhow::Result<()> {
    let since = parse_flag(args, "--since");

    let events = client
        .query_events(None, None, since.clone(), None)
        .await
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    if events.is_empty() {
        println!("No events found.");
        return Ok(());
    }

    println!("=== Activity Summary ===");
    if let Some(ref s) = since {
        println!("Since: {}", s);
    }
    println!("Total events: {}\n", events.len());

    // By event type
    let mut by_type: HashMap<String, usize> = HashMap::new();
    for e in &events {
        *by_type.entry(e.event_type.clone()).or_default() += 1;
    }
    let mut by_type_sorted: Vec<_> = by_type.into_iter().collect();
    by_type_sorted.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));

    println!("By event type:");
    for (t, count) in &by_type_sorted {
        println!("  {:<20} {}", t, count);
    }
    println!();

    // By source
    let mut by_source: HashMap<String, usize> = HashMap::new();
    for e in &events {
        *by_source.entry(e.source.clone()).or_default() += 1;
    }
    println!("By source:");
    for (s, count) in &by_source {
        println!("  {:<20} {}", s, count);
    }
    println!();

    // Top search queries
    let mut queries: HashMap<String, usize> = HashMap::new();
    for e in events.iter().filter(|e| e.event_type == "search") {
        if let Some(ref payload) = e.payload {
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(payload) {
                if let Some(q) = v.get("query").and_then(|q| q.as_str()) {
                    *queries.entry(q.to_string()).or_default() += 1;
                }
            }
        }
    }
    if !queries.is_empty() {
        let mut queries_sorted: Vec<_> = queries.into_iter().collect();
        queries_sorted.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));

        println!("Top search queries:");
        for (q, count) in queries_sorted.iter().take(10) {
            println!("  {:<40} {}", q, count);
        }
        println!();
    }

    Ok(())
}

async fn recent(
    client: &Arc<dyn ActivityClient + Send + Sync>,
    args: &[String],
) -> anyhow::Result<()> {
    let limit: u32 = parse_flag(args, "--limit")
        .and_then(|s| s.parse().ok())
        .unwrap_or(20);

    let events = client
        .query_events(None, None, None, Some(limit))
        .await
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    if events.is_empty() {
        println!("No events found.");
        return Ok(());
    }

    for e in &events {
        let payload_preview = e
            .payload
            .as_deref()
            .unwrap_or("")
            .chars()
            .take(80)
            .collect::<String>();
        println!(
            "{} [{:<3}] {:<20} {}",
            &e.timestamp[..19],
            e.source,
            e.event_type,
            payload_preview
        );
    }

    Ok(())
}

async fn agents(
    client: &Arc<dyn ActivityClient + Send + Sync>,
    args: &[String],
) -> anyhow::Result<()> {
    let since = parse_flag(args, "--since");

    let events = client
        .query_events(None, Some("http_request".to_string()), since, None)
        .await
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    if events.is_empty() {
        println!("No http_request events found.");
        return Ok(());
    }

    // Group by user_agent
    let mut by_agent: HashMap<String, usize> = HashMap::new();
    let mut by_ip: HashMap<String, usize> = HashMap::new();

    for e in &events {
        if let Some(ref payload) = e.payload {
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(payload) {
                let ua = v
                    .get("user_agent")
                    .and_then(|u| u.as_str())
                    .unwrap_or("(empty)")
                    .to_string();
                *by_agent.entry(ua).or_default() += 1;

                if let Some(ip) = v.get("ip").and_then(|i| i.as_str()) {
                    *by_ip.entry(ip.to_string()).or_default() += 1;
                }
            }
        }
    }

    println!("=== User-Agent Breakdown ({} requests) ===\n", events.len());

    let mut agents_sorted: Vec<_> = by_agent.into_iter().collect();
    agents_sorted.sort_by(|a, b| b.1.cmp(&a.1));
    println!("By user-agent:");
    for (ua, count) in agents_sorted.iter().take(20) {
        println!("  {:>6}  {}", count, ua);
    }

    println!();

    let mut ips_sorted: Vec<_> = by_ip.into_iter().collect();
    ips_sorted.sort_by(|a, b| b.1.cmp(&a.1));
    println!("By IP:");
    for (ip, count) in ips_sorted.iter().take(20) {
        println!("  {:>6}  {}", count, ip);
    }

    Ok(())
}
