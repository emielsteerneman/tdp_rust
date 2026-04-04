use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::path::Path;

use data_structures::file::TDPName;
use walkdir::WalkDir;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config_path = "config.toml";
    let config = configuration::AppConfig::load_from_file(config_path)
        .map_err(|e| anyhow::anyhow!("Failed to load config from {}: {}", config_path, e))?;

    let args: Vec<String> = std::env::args().collect();
    let subcommand = args.get(1).map(|s| s.as_str()).unwrap_or("all");

    let pdf_root = &config.data_processing.tdps_pdf_root;
    let md_root = &config.data_processing.tdps_markdown_root;

    match subcommand {
        "all" => {
            parsing(pdf_root, md_root)?;
            println!();
            indexing(md_root, &config).await?;
            println!();
            heatmap(pdf_root, md_root)?;
            println!();
            teams(&config).await?;
        }
        "parsing" => parsing(pdf_root, md_root)?,
        "indexing" => indexing(md_root, &config).await?,
        "heatmap" => heatmap(pdf_root, md_root)?,
        "teams" => teams(&config).await?,
        _ => {
            eprintln!("Usage: coverage <command>");
            eprintln!();
            eprintln!("Commands:");
            eprintln!("  all       Run all coverage checks (default)");
            eprintln!("  parsing   PDFs vs markdowns — what still needs parsing");
            eprintln!("  indexing  Markdowns on disk vs indexed in metadata DB");
            eprintln!("  heatmap   Papers per league per year grid");
            eprintln!("  teams     Teams without metadata in the team registry");
        }
    }

    Ok(())
}

/// Scan a directory for files with a given extension, return parsed TDPNames.
fn scan_dir(root: &str, ext: &str) -> HashSet<String> {
    let root_path = Path::new(root);
    if !root_path.is_dir() {
        eprintln!("Warning: directory does not exist: {}", root);
        return HashSet::new();
    }

    WalkDir::new(root_path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_file())
        .filter(|e| {
            e.path()
                .extension()
                .and_then(|x| x.to_str())
                .map(|x| x == ext)
                .unwrap_or(false)
        })
        .filter_map(|e| {
            e.path()
                .file_stem()
                .and_then(|s| s.to_str())
                .map(|s| s.to_string())
        })
        .filter(|stem| TDPName::try_from(stem.as_str()).is_ok())
        .collect()
}

/// Compare PDFs vs markdowns — what's been parsed, what hasn't.
fn parsing(pdf_root: &str, md_root: &str) -> anyhow::Result<()> {
    let pdfs = scan_dir(pdf_root, "pdf");
    let mds = scan_dir(md_root, "md");

    let pdfs_without_md: BTreeSet<_> = pdfs.difference(&mds).collect();
    let mds_without_pdf: BTreeSet<_> = mds.difference(&pdfs).collect();

    println!("=== Parsing Coverage ===");
    println!("PDFs on disk:      {}", pdfs.len());
    println!("Markdowns on disk: {}", mds.len());
    println!("Parsed:            {} / {} ({:.0}%)",
        mds.len(),
        pdfs.len(),
        if pdfs.is_empty() { 100.0 } else { mds.len() as f64 / pdfs.len() as f64 * 100.0 }
    );
    println!();

    if !pdfs_without_md.is_empty() {
        println!("PDFs without markdown ({}):", pdfs_without_md.len());
        for paper_lyt in pdfs_without_md.iter().take(20) {
            println!("  {}", paper_lyt);
        }
        if pdfs_without_md.len() > 20 {
            println!("  ... and {} more", pdfs_without_md.len() - 20);
        }
    }

    if !mds_without_pdf.is_empty() {
        println!();
        println!("Markdowns without PDF ({}) — unexpected:", mds_without_pdf.len());
        for paper_lyt in &mds_without_pdf {
            println!("  {}", paper_lyt);
        }
    }

    Ok(())
}

/// Compare markdowns on disk vs what's indexed in the metadata DB.
async fn indexing(md_root: &str, config: &configuration::AppConfig) -> anyhow::Result<()> {
    let metadata_client = configuration::helpers::load_any_metadata_client(config);

    let indexed_tdps = metadata_client.load_tdps().await
        .map_err(|e| anyhow::anyhow!("Failed to load TDPs from metadata DB: {}", e))?;
    let indexed: HashSet<String> = indexed_tdps
        .iter()
        .map(|t| t.get_paper_lyt())
        .collect();

    let on_disk = scan_dir(md_root, "md");

    let on_disk_not_indexed: BTreeSet<_> = on_disk.difference(&indexed).collect();
    let indexed_not_on_disk: BTreeSet<_> = indexed.difference(&on_disk).collect();

    println!("=== Indexing Coverage ===");
    println!("Markdowns on disk: {}", on_disk.len());
    println!("Indexed in DB:     {}", indexed.len());
    println!();

    if !on_disk_not_indexed.is_empty() {
        println!("On disk but not indexed ({}):", on_disk_not_indexed.len());
        for paper_lyt in on_disk_not_indexed.iter().take(20) {
            println!("  {}", paper_lyt);
        }
        if on_disk_not_indexed.len() > 20 {
            println!("  ... and {} more", on_disk_not_indexed.len() - 20);
        }
    }

    if !indexed_not_on_disk.is_empty() {
        println!("Indexed but not on disk ({}) — stale entries:", indexed_not_on_disk.len());
        for paper_lyt in &indexed_not_on_disk {
            println!("  {}", paper_lyt);
        }
    }

    if on_disk_not_indexed.is_empty() && indexed_not_on_disk.is_empty() {
        println!("All markdowns are indexed. No gaps.");
    }

    Ok(())
}

/// Show a heatmap of papers per league per year.
fn heatmap(pdf_root: &str, md_root: &str) -> anyhow::Result<()> {
    let pdfs = scan_dir(pdf_root, "pdf");
    let mds = scan_dir(md_root, "md");

    // Parse into (league_pretty, year) -> (pdf_count, md_count)
    let mut years: BTreeSet<u32> = BTreeSet::new();
    let mut leagues: BTreeSet<String> = BTreeSet::new();
    let mut pdf_counts: HashMap<(String, u32), usize> = HashMap::new();
    let mut md_counts: HashMap<(String, u32), usize> = HashMap::new();

    for paper_lyt in &pdfs {
        if let Ok(tdp) = TDPName::try_from(paper_lyt.as_str()) {
            let league = tdp.league.name_pretty().to_string();
            years.insert(tdp.year);
            leagues.insert(league.clone());
            *pdf_counts.entry((league, tdp.year)).or_default() += 1;
        }
    }
    for paper_lyt in &mds {
        if let Ok(tdp) = TDPName::try_from(paper_lyt.as_str()) {
            let league = tdp.league.name_pretty().to_string();
            years.insert(tdp.year);
            leagues.insert(league.clone());
            *md_counts.entry((league, tdp.year)).or_default() += 1;
        }
    }

    println!("=== Coverage Heatmap (PDFs / Markdowns) ===");
    println!();

    // Header
    let label_width = leagues.iter().map(|l| l.len()).max().unwrap_or(20);
    print!("{:width$}", "", width = label_width + 2);
    for y in &years {
        print!("{:>9}", y);
    }
    println!("    TOTAL");

    // Rows
    for league in &leagues {
        print!("{:width$}  ", league, width = label_width);
        let mut league_pdf_total = 0;
        let mut league_md_total = 0;
        for y in &years {
            let key = (league.clone(), *y);
            let pc = pdf_counts.get(&key).copied().unwrap_or(0);
            let mc = md_counts.get(&key).copied().unwrap_or(0);
            league_pdf_total += pc;
            league_md_total += mc;
            if pc == 0 && mc == 0 {
                print!("{:>9}", "·");
            } else if mc == pc {
                print!("{:>9}", format!("{}", pc));
            } else {
                print!("{:>9}", format!("{}/{}", mc, pc));
            }
        }
        println!("{:>9}", format!("{}/{}", league_md_total, league_pdf_total));
    }

    // Footer totals
    print!("{:width$}  ", "TOTAL", width = label_width);
    let mut grand_pdf = 0;
    let mut grand_md = 0;
    for y in &years {
        let pc: usize = leagues.iter().map(|l| pdf_counts.get(&(l.clone(), *y)).copied().unwrap_or(0)).sum();
        let mc: usize = leagues.iter().map(|l| md_counts.get(&(l.clone(), *y)).copied().unwrap_or(0)).sum();
        grand_pdf += pc;
        grand_md += mc;
        if mc == pc {
            print!("{:>9}", format!("{}", pc));
        } else {
            print!("{:>9}", format!("{}/{}", mc, pc));
        }
    }
    println!("{:>9}", format!("{}/{}", grand_md, grand_pdf));

    println!();
    println!("Legend: count = PDFs (all parsed), md/pdf = partially parsed, · = no papers");

    Ok(())
}

/// Find teams that have papers but no metadata in the team registry.
async fn teams(config: &configuration::AppConfig) -> anyhow::Result<()> {
    let metadata_client = configuration::helpers::load_any_metadata_client(config);
    let registry = match configuration::helpers::build_registry_client(config) {
        Some(r) => r,
        None => {
            println!("=== Team Registry Coverage ===");
            println!("No team registry configured. Skipping.");
            return Ok(());
        }
    };

    let indexed_tdps = metadata_client.load_tdps().await
        .map_err(|e| anyhow::anyhow!("Failed to load TDPs: {}", e))?;

    // Collect unique team names from indexed papers
    let teams: BTreeMap<String, usize> = {
        let mut map = BTreeMap::new();
        for tdp in &indexed_tdps {
            *map.entry(tdp.team_name.name_pretty.clone()).or_default() += 1;
        }
        map
    };

    let mut teams_without_metadata = Vec::new();
    let mut teams_with_metadata = 0;

    for (team_name, paper_count) in &teams {
        let entries = registry.get_team_metadata(team_name).await
            .unwrap_or_default();
        if entries.is_empty() {
            teams_without_metadata.push((team_name.clone(), *paper_count));
        } else {
            teams_with_metadata += 1;
        }
    }

    println!("=== Team Registry Coverage ===");
    println!("Teams with papers:    {}", teams.len());
    println!("Teams with metadata:  {} ({:.0}%)",
        teams_with_metadata,
        if teams.is_empty() { 100.0 } else { teams_with_metadata as f64 / teams.len() as f64 * 100.0 }
    );
    println!("Teams without metadata: {}", teams_without_metadata.len());

    if !teams_without_metadata.is_empty() {
        println!();
        // Sort by paper count descending — teams with more papers are higher priority
        teams_without_metadata.sort_by(|a, b| b.1.cmp(&a.1));
        println!("Teams without metadata (sorted by paper count):");
        for (team, count) in &teams_without_metadata {
            println!("  {:>3} papers  {}", count, team);
        }
    }

    Ok(())
}
