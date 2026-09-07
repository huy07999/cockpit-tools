use anyhow::{bail, Context};
use axum::extract::{Query, State};
use axum::http::{header::AUTHORIZATION, HeaderMap, StatusCode};
use axum::routing::{get, post};
use axum::{Json, Router};
use cockpit_core::models::cursor::CursorAccount;
use cockpit_core::models::github_copilot::GitHubCopilotAccount;
use cockpit_core::modules::{cursor_account, github_copilot_account};
use serde::{Deserialize, Serialize};
use std::env;
use std::net::{IpAddr, SocketAddr};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tower_http::services::{ServeDir, ServeFile};

#[derive(Clone)]
struct AppState {
    token: Option<Arc<str>>,
}

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
    version: &'static str,
    auth_required: bool,
    platforms: [&'static str; 2],
}

#[derive(Deserialize)]
struct AccountsQuery {
    platform: String,
}

#[derive(Serialize)]
struct AccountView {
    id: String,
    email: String,
    name: Option<String>,
    plan: Option<String>,
    tags: Vec<String>,
    status: Option<String>,
    last_used: i64,
    can_switch: bool,
}

#[derive(Serialize)]
struct AccountsResponse {
    platform: String,
    accounts: Vec<AccountView>,
}

#[derive(Deserialize)]
struct SwitchRequest {
    platform: String,
    account: String,
}

#[derive(Serialize)]
struct ActionResponse {
    success: bool,
    message: String,
}

#[derive(Debug, Serialize)]
struct ErrorResponse {
    error: String,
}

type ApiError = (StatusCode, Json<ErrorResponse>);

pub async fn serve(
    host: IpAddr,
    port: u16,
    web_root: Option<PathBuf>,
    token: Option<String>,
) -> anyhow::Result<()> {
    let token = token.and_then(|value| {
        let trimmed = value.trim();
        (!trimmed.is_empty()).then(|| Arc::<str>::from(trimmed))
    });
    if !host.is_loopback() && token.is_none() {
        bail!("--token or COCKPIT_WEB_TOKEN is required when binding outside localhost");
    }

    let web_root = resolve_web_root(web_root)?;
    let index_file = web_root.join("index.html");
    let state = AppState { token };
    let app = Router::new()
        .route("/api/health", get(health))
        .route("/api/accounts", get(list_accounts))
        .route("/api/switch", post(switch_account))
        .fallback_service(ServeDir::new(&web_root).fallback(ServeFile::new(index_file)))
        .with_state(state.clone());

    let address = SocketAddr::new(host, port);
    let listener = tokio::net::TcpListener::bind(address)
        .await
        .with_context(|| format!("failed to bind Cockpit WebUI to {address}"))?;

    println!("Cockpit WebUI: http://{address}");
    println!("Web root: {}", web_root.display());
    println!(
        "API authentication: {}",
        if state.token.is_some() {
            "bearer token required"
        } else {
            "localhost only"
        }
    );

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .context("Cockpit WebUI server stopped unexpectedly")
}

async fn health(State(state): State<AppState>) -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        version: env!("CARGO_PKG_VERSION"),
        auth_required: state.token.is_some(),
        platforms: ["cursor", "copilot"],
    })
}

async fn list_accounts(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<AccountsQuery>,
) -> Result<Json<AccountsResponse>, ApiError> {
    authorize(&state, &headers)?;
    let platform = normalize_platform(&query.platform)?;
    let accounts = match platform {
        "cursor" => cursor_account::list_accounts()
            .into_iter()
            .map(AccountView::from)
            .collect(),
        "copilot" => github_copilot_account::list_accounts()
            .into_iter()
            .map(AccountView::from)
            .collect(),
        _ => unreachable!(),
    };

    Ok(Json(AccountsResponse {
        platform: platform.to_string(),
        accounts,
    }))
}

async fn switch_account(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<SwitchRequest>,
) -> Result<Json<ActionResponse>, ApiError> {
    authorize(&state, &headers)?;
    let platform = normalize_platform(&request.platform)?;
    let account_id = request.account.trim();
    if account_id.is_empty() {
        return Err(api_error(StatusCode::BAD_REQUEST, "account is required"));
    }

    if platform == "copilot" {
        return Err(api_error(
            StatusCode::NOT_IMPLEMENTED,
            "GitHub Copilot switching is not available in the CLI backend yet",
        ));
    }

    let account_id = account_id.to_string();
    tokio::task::spawn_blocking(move || cursor_account::inject_to_cursor(&account_id))
        .await
        .map_err(|error| api_error(StatusCode::INTERNAL_SERVER_ERROR, error.to_string()))?
        .map_err(|error| api_error(StatusCode::BAD_REQUEST, error))?;

    Ok(Json(ActionResponse {
        success: true,
        message: "Cursor account switched successfully".to_string(),
    }))
}

fn normalize_platform(platform: &str) -> Result<&'static str, ApiError> {
    match platform.trim().to_ascii_lowercase().as_str() {
        "cursor" => Ok("cursor"),
        "copilot" | "github_copilot" | "github-copilot" => Ok("copilot"),
        _ => Err(api_error(StatusCode::BAD_REQUEST, "unsupported platform")),
    }
}

fn authorize(state: &AppState, headers: &HeaderMap) -> Result<(), ApiError> {
    let Some(expected) = state.token.as_deref() else {
        return Ok(());
    };
    let provided = headers
        .get(AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "));
    if provided == Some(expected) {
        Ok(())
    } else {
        Err(api_error(StatusCode::UNAUTHORIZED, "invalid bearer token"))
    }
}

fn api_error(status: StatusCode, message: impl Into<String>) -> ApiError {
    (
        status,
        Json(ErrorResponse {
            error: message.into(),
        }),
    )
}

fn resolve_web_root(explicit: Option<PathBuf>) -> anyhow::Result<PathBuf> {
    let mut candidates = Vec::new();
    if let Some(path) = explicit {
        candidates.push(path);
    }
    if let Some(path) = env::var_os("COCKPIT_WEB_ROOT") {
        candidates.push(PathBuf::from(path));
    }
    candidates.push(PathBuf::from("webui"));
    if let Ok(executable) = env::current_exe() {
        if let Some(parent) = executable.parent() {
            candidates.push(parent.join("webui"));
            candidates.push(parent.join("../share/cockpit-tools/webui"));
        }
    }

    candidates
        .into_iter()
        .find(|path| is_web_root(path))
        .map(|path| path.canonicalize().unwrap_or(path))
        .context("Cockpit WebUI files not found; extract the webui directory next to the binary or pass --web-root")
}

fn is_web_root(path: &Path) -> bool {
    path.join("index.html").is_file()
}

async fn shutdown_signal() {
    let _ = tokio::signal::ctrl_c().await;
}

impl From<CursorAccount> for AccountView {
    fn from(account: CursorAccount) -> Self {
        Self {
            id: account.id,
            email: account.email,
            name: account.name,
            plan: account.membership_type,
            tags: account.tags.unwrap_or_default(),
            status: account.status,
            last_used: account.last_used,
            can_switch: true,
        }
    }
}

impl From<GitHubCopilotAccount> for AccountView {
    fn from(account: GitHubCopilotAccount) -> Self {
        Self {
            id: account.id,
            email: account.github_email.unwrap_or(account.github_login),
            name: account.github_name,
            plan: account.copilot_plan,
            tags: account.tags.unwrap_or_default(),
            status: None,
            last_used: account.last_used,
            can_switch: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_unknown_platforms() {
        assert!(normalize_platform("unknown").is_err());
    }

    #[test]
    fn accepts_platform_aliases() {
        assert_eq!(normalize_platform("github-copilot").unwrap(), "copilot");
        assert_eq!(normalize_platform("Cursor").unwrap(), "cursor");
    }

    #[test]
    fn web_root_requires_index() {
        assert!(!is_web_root(Path::new("missing-web-root")));
    }
}
