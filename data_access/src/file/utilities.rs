use data_structures::paper::TDP;

use crate::file::FileError;

pub fn load_from_file_tdp_json(file_path: &str) -> Result<TDP, FileError> {
    let content = std::fs::read_to_string(file_path)?;
    let tdp: TDP = serde_json::from_str(&content)?;
    Ok(tdp)
}

pub fn load_from_file_all_tdp_json(file_paths: Vec<&str>) -> Result<Vec<TDP>, FileError> {
    let mut tdps = Vec::new();
    for file_path in file_paths {
        let tdp = load_from_file_tdp_json(file_path)?;
        tdps.push(tdp);
    }
    Ok(tdps)
}

pub fn load_from_dir_all_tdp_json(dir_path: &str) -> Result<Vec<TDP>, FileError> {
    let mut tdps = Vec::new();
    let entries = std::fs::read_dir(dir_path)?;
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("json") {
            let tdp = load_from_file_tdp_json(path.to_str().unwrap())?;
            tdps.push(tdp);
        }
    }
    Ok(tdps)
}

#[allow(dead_code)]
fn find_all_in_dir(file_root: &str, ext: &str) -> Result<Vec<std::path::PathBuf>, FileError> {
    // let folder_path = "/home/emiel/projects/tdps_json";
    let files = std::fs::read_dir(file_root)?;

    // parse each file into a TDPStructure
    for file in files {
        // print filename
        println!("### Processing file: {:?}", file);

        if file.is_err() {
            println!("@@@ Error reading file: {:?}", file.err());
            continue;
        }

        let file = file?;
        let path = file.path();
        if path.extension().and_then(|s| s.to_str()) == Some(ext) {
            let content = std::fs::read_to_string(&path)?;
            let tdp: TDP = serde_json::from_str(&content)?;
            println!("Loaded TDP: {}", tdp.name.get_filename());
        }
    }

    Ok(vec![])
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_load_all_tdps_from_dir() {
        let result = load_from_dir_all_tdp_json("/home/emiel/projects/tdps_json");
        println!("Result: {:?}", result);
        assert!(result.is_ok());
    }
}
