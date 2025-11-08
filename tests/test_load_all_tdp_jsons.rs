use std::error::Error;

use data_structures::paper::TDP;

#[tokio::test]
async fn test_load_all_tdp_jsons() -> Result<(), Box<dyn Error>> {
    let folder_path = "/home/emiel/projects/tdps_json";
    let files = std::fs::read_dir(folder_path)?;

    // parse each file into a TDPStructure
    for file in files {
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

    Ok(())
}
