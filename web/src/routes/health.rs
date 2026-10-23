use axum::http::StatusCode;

/// Liveness probe for the container healthcheck. Deliberately does no work and is
/// registered outside the activity-logging layer, so polling it every few seconds
/// neither costs a query nor floods the activity DB.
pub async fn healthz_handler() -> StatusCode {
    StatusCode::OK
}
