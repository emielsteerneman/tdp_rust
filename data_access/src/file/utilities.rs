// fn find_all_in_dir(file_root: &str, ext: &str) -> Result<Vec<std::path::PathBuf>> {
//     let folder_path = "/home/emiel/projects/tdps_json";
//     let files = std::fs::read_dir(file_root)?;

//     // parse each file into a TDPStructure
//     for file in files {
//         // print filename
//         println!("Processing file: {:?}", file);
//         let file = file?;
//         let path = file.path();
//         if path.extension().and_then(|s| s.to_str()) == Some("json") {
//             let content = std::fs::read_to_string(&path)?;
//             let tdp: TDP = serde_json::from_str(&content)?;
//             println!("Loaded TDP: {}", tdp.name.get_filename());
//         }
//     }

//     Ok(())
// }
