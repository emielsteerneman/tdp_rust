use axum::body::Body;
use axum::http::{Request, header};
use axum::middleware::Next;
use axum::response::Response;
use event_processing::client;
use std::time::Instant;

use crate::state::AppState;

pub async fn activity_logging(
    axum::extract::State(state): axum::extract::State<AppState>,
    request: Request<Body>,
    next: Next,
) -> Response {
    let method = request.method().to_string();
    let path = request.uri().path().to_string();
    let user_agent = request
        .headers()
        .get(header::USER_AGENT)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();
    let ip = request
        .headers()
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.split(',').next())
        .map(|s| s.trim().to_string())
        .or_else(|| {
            request
                .headers()
                .get("x-real-ip")
                .and_then(|v| v.to_str().ok())
                .map(|s| s.to_string())
        });

    let start = Instant::now();
    // Lets listeners (Telegram) tell crawlers from people for events dispatched in the handler.
    let response = client::scope(client::is_crawler_user_agent(&user_agent), next.run(request)).await;
    let duration_ms = start.elapsed().as_millis() as u64;

    let status = response.status().as_u16();

    state.dispatcher.dispatch(
        event_processing::EventSource::Web,
        event_processing::Event::HttpRequest(event_processing::HttpRequestEvent {
            method,
            path,
            status,
            duration_ms,
            ip,
            user_agent,
        }),
    );

    response
}
