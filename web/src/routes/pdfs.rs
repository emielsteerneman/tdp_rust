use axum::body::Body;
use axum::extract::{Path, State};
use axum::http::{HeaderValue, StatusCode, header};
use axum::response::Response;
use data_structures::file::TDPName;

use crate::state::AppState;

pub async fn serve_pdf_file(
    State(state): State<AppState>,
    Path(path): Path<String>,
) -> Result<Response<Body>, StatusCode> {
    // Strip .pdf extension
    let lyti_str = path
        .strip_suffix(".pdf")
        .ok_or(StatusCode::BAD_REQUEST)?;

    let tdp_name = TDPName::try_from(lyti_str).map_err(|_| StatusCode::BAD_REQUEST)?;

    // Build filesystem path
    let root = std::path::Path::new(&state.tdps_pdf_root);
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

    let file_path = league_path.join(format!("{}.pdf", lyti_str));

    // Security: canonicalize and verify under root
    let canonical_root = std::fs::canonicalize(root).map_err(|_| StatusCode::NOT_FOUND)?;
    let canonical_file = std::fs::canonicalize(&file_path).map_err(|_| StatusCode::NOT_FOUND)?;

    if !canonical_file.starts_with(&canonical_root) {
        return Err(StatusCode::FORBIDDEN);
    }

    // Read and return PDF
    let contents = tokio::fs::read(&canonical_file)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;

    let response = Response::builder()
        .status(StatusCode::OK)
        .header(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/pdf"),
        )
        .body(Body::from(contents))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(response)
}
