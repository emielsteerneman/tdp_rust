use axum::extract::State;
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};

use crate::state::AppState;

/// Entity-escape a `<loc>` value per the sitemap protocol. Team-derived paper slugs are
/// pipeline-curated (so metacharacters are unlikely), but a team name containing `&`, `<`,
/// etc. must not be able to emit malformed XML.
fn xml_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            _ => out.push(c),
        }
    }
    out
}

fn build_sitemap_xml(base_url: &str, paper_lyts: &[String]) -> String {
    let base = base_url.trim_end_matches('/');
    let mut out = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    out.push_str("<urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">\n");

    // Static, prerendered pages.
    for path in ["/", "/connect/mcp", "/connect/api"] {
        let loc = xml_escape(&format!("{base}{path}"));
        out.push_str(&format!("  <url><loc>{loc}</loc></url>\n"));
    }
    // One entry per paper.
    for lyt in paper_lyts {
        let loc = xml_escape(&format!("{base}/paper/{lyt}"));
        out.push_str(&format!("  <url><loc>{loc}</loc></url>\n"));
    }

    out.push_str("</urlset>\n");
    out
}

pub async fn sitemap_handler(State(state): State<AppState>) -> Response {
    // Fetch papers WITHOUT dispatching a ListPapers analytics event: bots poll the
    // sitemap and every hit would otherwise pollute analytics. This mirrors what
    // `list_papers` does internally, minus the event and the default no-op filter.
    let papers = state.metadata_client.load_tdps().await;

    let paper_lyts: Vec<String> = match papers {
        Ok(list) => list.iter().map(|p| p.get_paper_lyt()).collect(),
        Err(e) => {
            eprintln!("sitemap: failed to list papers: {e}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    let xml = build_sitemap_xml(&state.website_url, &paper_lyts);
    (
        [(header::CONTENT_TYPE, "application/xml; charset=utf-8")],
        xml,
    )
        .into_response()
}

#[cfg(test)]
mod tests {
    use super::build_sitemap_xml;

    #[test]
    fn builds_urlset_with_static_and_paper_urls() {
        let xml = build_sitemap_xml(
            "https://tdpsearch.com",
            &["soccer_smallsize__2024__RoboTeam_Twente".to_string()],
        );
        assert!(xml.starts_with("<?xml version=\"1.0\" encoding=\"UTF-8\"?>"));
        assert!(xml.contains("<urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">"));
        // homepage present
        assert!(xml.contains("<loc>https://tdpsearch.com/</loc>"));
        // paper URL present, correctly joined
        assert!(xml.contains(
            "<loc>https://tdpsearch.com/paper/soccer_smallsize__2024__RoboTeam_Twente</loc>"
        ));
        assert!(xml.trim_end().ends_with("</urlset>"));
    }

    #[test]
    fn trims_trailing_slash_on_base_url() {
        let xml = build_sitemap_xml("https://tdpsearch.com/", &[]);
        assert!(xml.contains("<loc>https://tdpsearch.com/</loc>"));
        assert!(!xml.contains("tdpsearch.com//"));
    }

    #[test]
    fn escapes_xml_metacharacters_in_paper_lyt() {
        let xml = build_sitemap_xml(
            "https://tdpsearch.com",
            &["soccer__2024__Smith_&_Sons".to_string()],
        );
        // The raw ampersand must be entity-escaped, never emitted literally.
        assert!(xml.contains("/paper/soccer__2024__Smith_&amp;_Sons</loc>"));
        assert!(!xml.contains("Smith_&_Sons"));
    }
}
