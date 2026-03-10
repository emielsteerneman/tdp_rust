use axum::body::Body;
use axum::extract::{Path, State};
use axum::http::{HeaderValue, StatusCode, header};
use axum::response::Response;
use data_structures::file::TDPName;

use crate::state::AppState;

pub async fn serve_tdps_file(
    State(state): State<AppState>,
    Path(path): Path<String>,
) -> Result<Response<Body>, StatusCode> {
    // The path is either:
    //   {lyti}.md          — the markdown file itself
    //   {lyti}/{subpath}   — a file inside the paper's image folder
    let (lyti_str, subpath) = if let Some((lyti, sub)) = path.split_once('/') {
        (lyti, Some(sub))
    } else {
        (path.as_str(), None)
    };

    // Strip .md extension from lyti if present (for the markdown file case)
    let lyti_for_parse = if lyti_str.ends_with(".md") {
        &lyti_str[..lyti_str.len() - 3]
    } else {
        lyti_str
    };

    let tdp_name: TDPName = TDPName::try_from(lyti_for_parse).map_err(|_| StatusCode::BAD_REQUEST)?;

    // Build the filesystem path for the file
    let root = std::path::Path::new(&state.tdps_markdown_root);

    let league = &tdp_name.league;
    let year = tdp_name.year.to_string();

    let league_path = if let Some(ref sub) = league.league_sub {
        root.join(&league.league_major)
            .join(&league.league_minor)
            .join(sub)
            .join(&year)
    } else {
        root.join(&league.league_major)
            .join(&league.league_minor)
            .join(&year)
    };

    let lyti_base = lyti_for_parse;

    let file_path = match subpath {
        None => {
            // Markdown file: {league_path}/{lyti}.md
            league_path.join(format!("{}.md", lyti_base))
        }
        Some(sub) => {
            // Image/asset inside paper folder: {league_path}/{lyti}/{subpath}
            league_path.join(lyti_base).join(sub)
        }
    };

    // Security: canonicalize root and target, ensure target is under root
    let canonical_root = std::fs::canonicalize(root).map_err(|_| StatusCode::NOT_FOUND)?;
    let canonical_file = std::fs::canonicalize(&file_path).map_err(|_| StatusCode::NOT_FOUND)?;

    if !canonical_file.starts_with(&canonical_root) {
        return Err(StatusCode::FORBIDDEN);
    }

    // Read file contents
    let contents = tokio::fs::read(&canonical_file)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;

    // Determine Content-Type from extension
    let content_type = match canonical_file
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .as_deref()
    {
        Some("md") => "text/markdown; charset=utf-8",
        Some("jpeg") | Some("jpg") => "image/jpeg",
        Some("png") => "image/png",
        Some("gif") => "image/gif",
        Some("webp") => "image/webp",
        Some("svg") => "image/svg+xml",
        _ => "application/octet-stream",
    };

    let content_type_value =
        HeaderValue::from_str(content_type).unwrap_or_else(|_| HeaderValue::from_static("application/octet-stream"));

    let response = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, content_type_value)
        .body(Body::from(contents))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(response)
}
