use axum::Json;
use serde::Serialize;

#[derive(Serialize)]
pub struct ApiRoute {
    pub method: &'static str,
    pub path: &'static str,
    pub description: &'static str,
}

pub async fn api_index_handler() -> Json<Vec<ApiRoute>> {
    Json(vec![
        ApiRoute {
            method: "GET",
            path: "/api",
            description: "List all available API routes",
        },
        ApiRoute {
            method: "GET",
            path: "/api/search?query=<query>&league=&year=&team=&content_type=&search_type=",
            description: "Search across all papers using hybrid semantic+keyword search",
        },
        ApiRoute {
            method: "GET",
            path: "/api/papers?league=&year=&team=",
            description: "List papers, optionally filtered by league, year, or team",
        },
        ApiRoute {
            method: "GET",
            path: "/api/papers/{paper_lyt}/toc",
            description: "Get the table of contents for a paper",
        },
        ApiRoute {
            method: "GET",
            path: "/api/papers/{paper_lyt}/abstract",
            description: "Get the abstract of a paper",
        },
        ApiRoute {
            method: "GET",
            path: "/api/papers/{paper_lyt}/references",
            description: "Get the references/bibliography of a paper",
        },
        ApiRoute {
            method: "GET",
            path: "/api/papers/{paper_lyt}/info",
            description: "Get paper metadata: title, authors, institutions, URLs",
        },
        ApiRoute {
            method: "GET",
            path: "/api/papers/{paper_lyt}/paragraph/{seq}",
            description: "Get a specific paragraph by content sequence number",
        },
        ApiRoute {
            method: "GET",
            path: "/api/papers/{paper_lyt}/table/{seq}",
            description: "Get a specific table by content sequence number",
        },
        ApiRoute {
            method: "GET",
            path: "/api/papers/{paper_lyt}/image/{seq}",
            description: "Get a specific image by content sequence number",
        },
        ApiRoute {
            method: "POST",
            path: "/api/papers/{paper_lyt}/open",
            description: "Track a paper open event (analytics)",
        },
        ApiRoute {
            method: "POST",
            path: "/api/papers/{paper_lyt}/pdf-open",
            description: "Track a PDF open event (analytics)",
        },
        ApiRoute {
            method: "GET",
            path: "/api/teams",
            description: "List all teams in the corpus",
        },
        ApiRoute {
            method: "GET",
            path: "/api/leagues",
            description: "List all leagues",
        },
        ApiRoute {
            method: "GET",
            path: "/api/years?league=&team=",
            description: "List all years, optionally filtered by league or team",
        },
        ApiRoute {
            method: "POST",
            path: "/api/suggestion",
            description: "Submit a user suggestion or feedback message",
        },
        ApiRoute {
            method: "GET",
            path: "/api/registry/team/{name}",
            description: "Get team metadata: GitHub, website, social links",
        },
        ApiRoute {
            method: "POST",
            path: "/api/registry/team",
            description: "Update team metadata (requires team code or master password)",
        },
        ApiRoute {
            method: "GET",
            path: "/api/registry/league/{name}",
            description: "Get league metadata: official site, GitHub org, rules, community links",
        },
    ])
}
