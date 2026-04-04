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
    //   {paper_lyt}.md          — the markdown file itself
    //   {paper_lyt}/{subpath}   — a file inside the paper's image folder
    let (paper_lyt_str, subpath) = if let Some((paper_lyt, sub)) = path.split_once('/') {
        (paper_lyt, Some(sub))
    } else {
        (path.as_str(), None)
    };

    // Strip .md extension from paper_lyt if present (for the markdown file case)
    let paper_lyt_for_parse = if paper_lyt_str.ends_with(".md") {
        &paper_lyt_str[..paper_lyt_str.len() - 3]
    } else {
        paper_lyt_str
    };

    let tdp_name: TDPName = TDPName::try_from(paper_lyt_for_parse).map_err(|_| StatusCode::BAD_REQUEST)?;

    // Build the filesystem path for the file
    let root = std::path::Path::new(&state.tdps_markdown_root);

    let league = &tdp_name.league;
    let year = tdp_name.year.to_string();

    let league_path = if let Some(sub) = league.sub() {
        root.join(league.major())
            .join(league.minor())
            .join(sub)
            .join(&year)
    } else {
        root.join(league.major())
            .join(league.minor())
            .join(&year)
    };

    let paper_lyt_base = paper_lyt_for_parse;

    let file_path = match subpath {
        None => {
            // Markdown file: {league_path}/{paper_lyt}.md
            league_path.join(format!("{}.md", paper_lyt_base))
        }
        Some(sub) => {
            // Image/asset inside paper folder: {league_path}/{paper_lyt}/{subpath}
            league_path.join(paper_lyt_base).join(sub)
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
