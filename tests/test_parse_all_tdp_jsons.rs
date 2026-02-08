use std::error::Error;

use data_structures::paper::TDP;

#[tokio::test]
async fn test_load_all_tdp_jsons() -> Result<(), Box<dyn Error>> {
    let config = configuration::AppConfig::load_from_file("config.toml")
        .expect("Could not find config.toml");
    let files = std::fs::read_dir(&config.data_processing.tdps_json_root)?;

    let mut count = 0;
    // parse each file into a TDPStructure
    for file in files {
        count += 1;
        // print filename
        println!("Processing file: {:?}", file);
        let file = file?;
        let path = file.path();
        if path.extension().and_then(|s| s.to_str()) == Some("json") {
            let content = std::fs::read_to_string(&path)?;
            let tdp: TDP = serde_json::from_str(&content)?;
            println!("Loaded TDP: {}", tdp.name.get_filename());
        }
    }

    println!("Processed {count} files");

    Ok(())
}
