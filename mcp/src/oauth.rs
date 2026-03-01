use std::{collections::HashMap, sync::Arc};

use axum::{
    Form, Json, Router,
    body::Body,
    extract::{Query, State},
    http::{HeaderMap, Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Redirect, Response},
    routing::{get, post},
};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tokio::sync::RwLock;
use tower_http::cors::{Any, CorsLayer};
use uuid::Uuid;

// ─── Store ────────────────────────────────────────────────────────────────────

/// Thread-safe in-memory OAuth state. `Clone` is cheap — all fields are `Arc`.
#[derive(Clone)]
pub struct OAuthStore {
    clients: Arc<RwLock<HashMap<String, RegisteredClient>>>,
    auth_codes: Arc<RwLock<HashMap<String, AuthCode>>>,
    access_tokens: Arc<RwLock<HashMap<String, StoredToken>>>,
}

#[derive(Clone)]
#[allow(dead_code)] // fields are stored for potential future use (e.g. token revocation, client info)
struct RegisteredClient {
    client_id: String,
    client_secret: String,
    client_name: String,
    redirect_uris: Vec<String>,
}

/// A single-use authorization code pending exchange for a token.
#[derive(Clone)]
struct AuthCode {
    client_id: String,
    redirect_uri: String,
    /// BASE64URL(SHA256(code_verifier)) — stored so we can verify PKCE later.
    code_challenge: String,
}

#[derive(Clone)]
struct StoredToken {
    #[allow(dead_code)]
    client_id: String,
}

impl OAuthStore {
    pub fn new() -> Self {
        Self {
            clients: Arc::new(RwLock::new(HashMap::new())),
            auth_codes: Arc::new(RwLock::new(HashMap::new())),
            access_tokens: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Check whether `token` is a valid, previously issued access token.
    pub async fn validate_token(&self, token: &str) -> bool {
        self.access_tokens.read().await.contains_key(token)
    }
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

fn base_url(host: &str) -> String {
    let scheme = if host.starts_with("localhost") || host.starts_with("127.0.0.1") {
        "http"
    } else {
        "https"
    };
    format!("{scheme}://{host}")
}

fn pkce_challenge(verifier: &str) -> String {
    URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()))
}

// ─── Request / response types ─────────────────────────────────────────────────

#[derive(Deserialize)]
struct RegistrationRequest {
    #[serde(default)]
    client_name: Option<String>,
    redirect_uris: Vec<String>,
}

#[derive(Serialize)]
struct RegistrationResponse {
    client_id: String,
    client_secret: String,
    client_name: String,
    redirect_uris: Vec<String>,
}

#[derive(Deserialize)]
struct AuthorizeParams {
    response_type: String,
    client_id: String,
    redirect_uri: String,
    code_challenge: String,
    code_challenge_method: String,
    state: Option<String>,
}

#[derive(Deserialize)]
struct TokenRequest {
    grant_type: String,
    code: String,
    client_id: String,
    redirect_uri: String,
    code_verifier: String,
}

#[derive(Serialize)]
struct TokenResponse {
    access_token: String,
    token_type: String,
    expires_in: u64,
}

#[derive(Serialize)]
struct OAuthErrorResponse {
    error: String,
    error_description: String,
}

fn oauth_error(error: &str, description: &str) -> (StatusCode, Json<OAuthErrorResponse>) {
    (
        StatusCode::BAD_REQUEST,
        Json(OAuthErrorResponse {
            error: error.to_string(),
            error_description: description.to_string(),
        }),
    )
}

// ─── Handlers ─────────────────────────────────────────────────────────────────

/// RFC 8414 — Authorization Server Metadata discovery.
/// The base URL is derived from the `Host` header so it works correctly both
/// behind the Cloudflare tunnel (public URL) and in local development.
async fn metadata(headers: HeaderMap) -> impl IntoResponse {
    let host = headers
        .get("host")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("localhost");
    let base = base_url(host);
    Json(serde_json::json!({
        "issuer": format!("{base}/"),
        "authorization_endpoint": format!("{base}/oauth/authorize"),
        "token_endpoint": format!("{base}/oauth/token"),
        "registration_endpoint": format!("{base}/oauth/register"),
        "response_types_supported": ["code"],
        "grant_types_supported": ["authorization_code"],
        "code_challenge_methods_supported": ["S256"],
        "token_endpoint_auth_methods_supported": ["client_secret_post", "none"],
    }))
}

/// RFC 7591 — Dynamic Client Registration.
async fn register(
    State(store): State<OAuthStore>,
    Json(req): Json<RegistrationRequest>,
) -> impl IntoResponse {
    if req.redirect_uris.is_empty() {
        return oauth_error("invalid_client_metadata", "redirect_uris must not be empty")
            .into_response();
    }

    let client_id = format!("client-{}", Uuid::new_v4());
    let client_secret = Uuid::new_v4().to_string();
    let client_name = req.client_name.unwrap_or_else(|| "Unknown Client".to_string());

    store.clients.write().await.insert(
        client_id.clone(),
        RegisteredClient {
            client_id: client_id.clone(),
            client_secret: client_secret.clone(),
            client_name: client_name.clone(),
            redirect_uris: req.redirect_uris.clone(),
        },
    );

    (
        StatusCode::CREATED,
        Json(RegistrationResponse {
            client_id,
            client_secret,
            client_name,
            redirect_uris: req.redirect_uris,
        }),
    )
        .into_response()
}

/// OAuth 2.1 Authorization Endpoint — auto-approves all requests.
///
/// Validates the client and PKCE parameters, then immediately redirects back
/// with an authorization code. No consent form, no user login.
async fn authorize(
    State(store): State<OAuthStore>,
    Query(params): Query<AuthorizeParams>,
) -> impl IntoResponse {
    if params.response_type != "code" {
        return oauth_error("unsupported_response_type", "only 'code' is supported").into_response();
    }
    if params.code_challenge_method != "S256" {
        return oauth_error("invalid_request", "only S256 code_challenge_method is supported")
            .into_response();
    }

    let clients = store.clients.read().await;
    let Some(client) = clients.get(&params.client_id) else {
        return oauth_error("invalid_client", "unknown client_id").into_response();
    };
    if !client.redirect_uris.contains(&params.redirect_uri) {
        return oauth_error("invalid_request", "redirect_uri not registered for this client")
            .into_response();
    }
    drop(clients);

    let code = format!("code-{}", Uuid::new_v4());
    store.auth_codes.write().await.insert(
        code.clone(),
        AuthCode {
            client_id: params.client_id,
            redirect_uri: params.redirect_uri.clone(),
            code_challenge: params.code_challenge,
        },
    );

    let mut redirect_url = format!("{}?code={}", params.redirect_uri, code);
    if let Some(state) = params.state {
        redirect_url.push_str(&format!("&state={state}"));
    }

    Redirect::to(&redirect_url).into_response()
}

/// OAuth 2.1 Token Endpoint — exchanges an authorization code for an access token.
/// Verifies PKCE S256 and that codes are single-use.
async fn token(
    State(store): State<OAuthStore>,
    Form(req): Form<TokenRequest>,
) -> impl IntoResponse {
    if req.grant_type != "authorization_code" {
        return oauth_error("unsupported_grant_type", "only authorization_code is supported")
            .into_response();
    }

    // Remove the code from the store — codes are single-use.
    let Some(auth_code) = store.auth_codes.write().await.remove(&req.code) else {
        return oauth_error("invalid_grant", "unknown or already-used authorization code")
            .into_response();
    };

    if auth_code.client_id != req.client_id {
        return oauth_error("invalid_grant", "client_id mismatch").into_response();
    }
    if auth_code.redirect_uri != req.redirect_uri {
        return oauth_error("invalid_grant", "redirect_uri mismatch").into_response();
    }

    // PKCE verification: SHA256(code_verifier) must equal the stored challenge.
    if pkce_challenge(&req.code_verifier) != auth_code.code_challenge {
        return oauth_error("invalid_grant", "code_verifier does not match code_challenge")
            .into_response();
    }

    let access_token = format!("token-{}", Uuid::new_v4());
    store.access_tokens.write().await.insert(
        access_token.clone(),
        StoredToken {
            client_id: auth_code.client_id,
        },
    );

    Json(TokenResponse {
        access_token,
        token_type: "Bearer".to_string(),
        expires_in: 3600,
    })
    .into_response()
}

// ─── Middleware ────────────────────────────────────────────────────────────────

/// Axum middleware that validates Bearer tokens issued by this OAuth server.
/// Returns 401 for requests without a valid token.
pub async fn validate_token(
    State(store): State<OAuthStore>,
    request: Request<Body>,
    next: Next,
) -> Response {
    let token = request
        .headers()
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .map(str::to_owned);

    match token {
        Some(t) if store.validate_token(&t).await => next.run(request).await,
        _ => StatusCode::UNAUTHORIZED.into_response(),
    }
}

// ─── Router builder ───────────────────────────────────────────────────────────

/// Builds the OAuth endpoints router. Merge this with your protected routes in
/// `main.rs`:
///
/// ```ignore
/// let auth_router = oauth::oauth_router(store.clone())
///     .merge(protected_mcp_router);
/// ```
pub fn oauth_router(store: OAuthStore) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .route("/.well-known/oauth-authorization-server", get(metadata))
        .route("/oauth/register", post(register))
        .route("/oauth/authorize", get(authorize))
        .route("/oauth/token", post(token))
        .layer(cors)
        .with_state(store)
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    /// Build a router that has the OAuth endpoints plus a simple protected route,
    /// useful for testing the middleware without spinning up a real MCP service.
    fn test_app(store: OAuthStore) -> Router {
        let protected = Router::new()
            .route("/protected", get(|| async { "ok" }))
            .layer(axum::middleware::from_fn_with_state(
                store.clone(),
                validate_token,
            ));

        oauth_router(store).merge(protected)
    }

    async fn body_json(body: Body) -> serde_json::Value {
        let bytes = body.collect().await.unwrap().to_bytes();
        serde_json::from_slice(&bytes).unwrap()
    }

    /// Complete OAuth flow: register → authorize → token exchange.
    /// Returns (client_id, redirect_uri, access_token).
    async fn full_oauth_flow(app: Router) -> (String, String, String) {
        let redirect_uri = "http://localhost/callback";
        let code_verifier = "dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk";
        let code_challenge = pkce_challenge(code_verifier);

        // 1. Register
        let resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/oauth/register")
                    .header("content-type", "application/json")
                    .header("host", "localhost:50002")
                    .body(Body::from(
                        serde_json::json!({
                            "client_name": "Test Client",
                            "redirect_uris": [redirect_uri]
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::CREATED);
        let reg = body_json(resp.into_body()).await;
        let client_id = reg["client_id"].as_str().unwrap().to_owned();

        // 2. Authorize (auto-approve → redirect with code; axum Redirect::to uses 303)
        let authorize_url = format!(
            "/oauth/authorize?response_type=code&client_id={client_id}&redirect_uri={redirect_uri}&code_challenge={code_challenge}&code_challenge_method=S256&state=xyz"
        );
        let resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(&authorize_url)
                    .header("host", "localhost:50002")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert!(resp.status().is_redirection());
        let location = resp
            .headers()
            .get("location")
            .unwrap()
            .to_str()
            .unwrap()
            .to_owned();
        let code = location
            .split("code=")
            .nth(1)
            .unwrap()
            .split('&')
            .next()
            .unwrap()
            .to_owned();

        // 3. Exchange code for token
        let body = format!(
            "grant_type=authorization_code&code={code}&client_id={client_id}&redirect_uri={redirect_uri}&code_verifier={code_verifier}"
        );
        let resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/oauth/token")
                    .header("content-type", "application/x-www-form-urlencoded")
                    .header("host", "localhost:50002")
                    .body(Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let tok = body_json(resp.into_body()).await;
        let access_token = tok["access_token"].as_str().unwrap().to_owned();

        (client_id, redirect_uri.to_string(), access_token)
    }

    #[tokio::test]
    async fn test_metadata_endpoint() {
        let app = test_app(OAuthStore::new());
        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/.well-known/oauth-authorization-server")
                    .header("host", "mcp-auth.emielsteerneman.nl")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let json = body_json(resp.into_body()).await;
        assert!(json["authorization_endpoint"]
            .as_str()
            .unwrap()
            .starts_with("https://mcp-auth.emielsteerneman.nl"));
        assert!(json["token_endpoint"].as_str().is_some());
        assert!(json["registration_endpoint"].as_str().is_some());
    }

    #[tokio::test]
    async fn test_metadata_localhost_uses_http() {
        let app = test_app(OAuthStore::new());
        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/.well-known/oauth-authorization-server")
                    .header("host", "localhost:50002")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let json = body_json(resp.into_body()).await;
        assert!(json["authorization_endpoint"]
            .as_str()
            .unwrap()
            .starts_with("http://localhost"));
    }

    #[tokio::test]
    async fn test_register_client() {
        let app = test_app(OAuthStore::new());
        let resp = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/oauth/register")
                    .header("content-type", "application/json")
                    .header("host", "localhost:50002")
                    .body(Body::from(
                        serde_json::json!({
                            "client_name": "My Client",
                            "redirect_uris": ["https://example.com/callback"]
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::CREATED);
        let json = body_json(resp.into_body()).await;
        assert!(json["client_id"].as_str().unwrap().starts_with("client-"));
        assert!(json["client_secret"].as_str().is_some());
    }

    #[tokio::test]
    async fn test_register_no_redirect_uris() {
        let app = test_app(OAuthStore::new());
        let resp = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/oauth/register")
                    .header("content-type", "application/json")
                    .header("host", "localhost:50002")
                    .body(Body::from(
                        serde_json::json!({
                            "client_name": "Bad Client",
                            "redirect_uris": []
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_full_oauth_flow() {
        let store = OAuthStore::new();
        let app = test_app(store.clone());
        let (_, _, access_token) = full_oauth_flow(app).await;
        // The token should now be in the store
        assert!(store.validate_token(&access_token).await);
    }

    #[tokio::test]
    async fn test_pkce_wrong_verifier() {
        let store = OAuthStore::new();
        let app = test_app(store.clone());

        let redirect_uri = "http://localhost/callback";
        let code_verifier = "correct-verifier-long-enough-for-pkce-spec-requirements";
        let code_challenge = pkce_challenge(code_verifier);

        // Register
        let resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/oauth/register")
                    .header("content-type", "application/json")
                    .header("host", "localhost:50002")
                    .body(Body::from(
                        serde_json::json!({
                            "redirect_uris": [redirect_uri]
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        let client_id = body_json(resp.into_body()).await["client_id"]
            .as_str()
            .unwrap()
            .to_owned();

        // Authorize
        let resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(format!(
                        "/oauth/authorize?response_type=code&client_id={client_id}&redirect_uri={redirect_uri}&code_challenge={code_challenge}&code_challenge_method=S256"
                    ))
                    .header("host", "localhost:50002")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let location = resp.headers().get("location").unwrap().to_str().unwrap().to_owned();
        let code = location.split("code=").nth(1).unwrap().split('&').next().unwrap();

        // Token exchange with wrong verifier
        let resp = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/oauth/token")
                    .header("content-type", "application/x-www-form-urlencoded")
                    .header("host", "localhost:50002")
                    .body(Body::from(format!(
                        "grant_type=authorization_code&code={code}&client_id={client_id}&redirect_uri={redirect_uri}&code_verifier=wrong-verifier"
                    )))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
        let json = body_json(resp.into_body()).await;
        assert_eq!(json["error"], "invalid_grant");
    }

    #[tokio::test]
    async fn test_auth_code_single_use() {
        let store = OAuthStore::new();
        let redirect_uri = "http://localhost/callback";
        let code_verifier = "dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk";
        let code_challenge = pkce_challenge(code_verifier);

        // Register
        let app = test_app(store.clone());
        let resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/oauth/register")
                    .header("content-type", "application/json")
                    .header("host", "localhost:50002")
                    .body(Body::from(
                        serde_json::json!({ "redirect_uris": [redirect_uri] }).to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        let client_id = body_json(resp.into_body()).await["client_id"]
            .as_str()
            .unwrap()
            .to_owned();

        // Authorize
        let resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(format!(
                        "/oauth/authorize?response_type=code&client_id={client_id}&redirect_uri={redirect_uri}&code_challenge={code_challenge}&code_challenge_method=S256"
                    ))
                    .header("host", "localhost:50002")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let location = resp.headers().get("location").unwrap().to_str().unwrap().to_owned();
        let code = location.split("code=").nth(1).unwrap().split('&').next().unwrap().to_owned();

        let token_body = format!(
            "grant_type=authorization_code&code={code}&client_id={client_id}&redirect_uri={redirect_uri}&code_verifier={code_verifier}"
        );

        // First exchange — should succeed
        let resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/oauth/token")
                    .header("content-type", "application/x-www-form-urlencoded")
                    .header("host", "localhost:50002")
                    .body(Body::from(token_body.clone()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        // Second exchange with the same code — should fail
        let resp = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/oauth/token")
                    .header("content-type", "application/x-www-form-urlencoded")
                    .header("host", "localhost:50002")
                    .body(Body::from(token_body))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_middleware_valid_token() {
        let store = OAuthStore::new();
        let app = test_app(store.clone());
        let (_, _, token) = full_oauth_flow(app.clone()).await;

        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/protected")
                    .header("Authorization", format!("Bearer {token}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_middleware_no_token() {
        let app = test_app(OAuthStore::new());
        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/protected")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_middleware_invalid_token() {
        let app = test_app(OAuthStore::new());
        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/protected")
                    .header("Authorization", "Bearer not-a-real-token")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }
}
