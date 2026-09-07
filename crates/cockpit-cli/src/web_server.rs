use anyhow::{bail, Context};
use axum::extract::{Path as AxumPath, Query, State};
use axum::http::{header::AUTHORIZATION, HeaderMap, StatusCode};
use axum::routing::{delete, get, patch, post};
use axum::{Json, Router};
use cockpit_core::modules::{config, cursor_account};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::env;
use std::fs;
use std::net::{IpAddr, SocketAddr};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tower_http::services::{ServeDir, ServeFile};

#[derive(Clone)]
struct AppState {
    token: Option<Arc<str>>,
}

#[derive(Clone, Copy, Serialize)]
struct ProviderDefinition {
    id: &'static str,
    label: &'static str,
    #[serde(skip_serializing)]
    directory: &'static str,
    #[serde(skip_serializing)]
    index_file: &'static str,
    can_switch: bool,
}

const PROVIDERS: [ProviderDefinition; 18] = [
    provider(
        "antigravity",
        "Antigravity",
        "accounts",
        "accounts.json",
        false,
    ),
    provider(
        "codex",
        "Codex",
        "codex_accounts",
        "codex_accounts.json",
        false,
    ),
    provider(
        "claude",
        "Claude",
        "claude_accounts",
        "claude_accounts.json",
        false,
    ),
    provider("zed", "Zed", "zed_accounts", "zed_accounts.json", false),
    provider(
        "github-copilot",
        "GitHub Copilot",
        "github_copilot_accounts",
        "github_copilot_accounts.json",
        false,
    ),
    provider(
        "windsurf",
        "Windsurf",
        "windsurf_accounts",
        "windsurf_accounts.json",
        false,
    ),
    provider("kiro", "Kiro", "kiro_accounts", "kiro_accounts.json", false),
    provider(
        "cursor",
        "Cursor",
        "cursor_accounts",
        "cursor_accounts.json",
        true,
    ),
    provider("grok", "Grok", "grok_accounts", "grok_accounts.json", false),
    provider(
        "codebuddy",
        "CodeBuddy",
        "codebuddy_accounts",
        "codebuddy_accounts.json",
        false,
    ),
    provider(
        "codebuddy-cn",
        "CodeBuddy CN",
        "codebuddy_cn_accounts",
        "codebuddy_cn_accounts.json",
        false,
    ),
    provider(
        "qoder",
        "Qoder",
        "qoder_accounts",
        "qoder_accounts.json",
        false,
    ),
    provider(
        "zcode",
        "ZCode",
        "zcode_accounts",
        "zcode_accounts.json",
        false,
    ),
    provider("trae", "TRAE", "trae_accounts", "trae_accounts.json", false),
    provider(
        "trae-solo",
        "TRAE Solo",
        "trae_accounts",
        "trae_accounts.json",
        false,
    ),
    provider(
        "trae-cn",
        "TRAE CN",
        "trae_accounts",
        "trae_accounts.json",
        false,
    ),
    provider(
        "trae-solo-cn",
        "TRAE Solo CN",
        "trae_accounts",
        "trae_accounts.json",
        false,
    ),
    provider(
        "workbuddy",
        "WorkBuddy",
        "workbuddy_accounts",
        "workbuddy_accounts.json",
        false,
    ),
];

const fn provider(
    id: &'static str,
    label: &'static str,
    directory: &'static str,
    index_file: &'static str,
    can_switch: bool,
) -> ProviderDefinition {
    ProviderDefinition {
        id,
        label,
        directory,
        index_file,
        can_switch,
    }
}

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
    version: &'static str,
    auth_required: bool,
    providers: usize,
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

#[derive(Deserialize)]
struct TagsRequest {
    tags: Vec<String>,
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
    let state = AppState { token };
    let app = Router::new()
        .route("/api/health", get(health))
        .route("/api/providers", get(list_providers))
        .route("/api/accounts", get(list_accounts))
        .route("/api/accounts/{platform}/{account}", delete(delete_account))
        .route(
            "/api/accounts/{platform}/{account}/tags",
            patch(update_tags),
        )
        .route("/api/switch", post(switch_account))
        .fallback_service(
            ServeDir::new(&web_root).fallback(ServeFile::new(web_root.join("index.html"))),
        )
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
        providers: PROVIDERS.len(),
    })
}

async fn list_providers(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<&'static [ProviderDefinition]>, ApiError> {
    authorize(&state, &headers)?;
    Ok(Json(&PROVIDERS))
}

async fn list_accounts(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<AccountsQuery>,
) -> Result<Json<AccountsResponse>, ApiError> {
    authorize(&state, &headers)?;
    let definition = find_provider(&query.platform)?;
    let accounts = load_account_values(definition)?
        .into_iter()
        .filter(|value| provider_matches(definition.id, value))
        .filter_map(|value| account_view(definition, &value))
        .collect();
    Ok(Json(AccountsResponse {
        platform: definition.id.to_string(),
        accounts,
    }))
}

async fn switch_account(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<SwitchRequest>,
) -> Result<Json<ActionResponse>, ApiError> {
    authorize(&state, &headers)?;
    let definition = find_provider(&request.platform)?;
    if !definition.can_switch {
        return Err(api_error(
            StatusCode::NOT_IMPLEMENTED,
            format!(
                "{} switching requires its desktop client and is unavailable on Android",
                definition.label
            ),
        ));
    }
    let account = validate_account_id(&request.account)?.to_string();
    tokio::task::spawn_blocking(move || cursor_account::inject_to_cursor(&account))
        .await
        .map_err(|error| api_error(StatusCode::INTERNAL_SERVER_ERROR, error.to_string()))?
        .map_err(|error| api_error(StatusCode::BAD_REQUEST, error))?;
    Ok(Json(ActionResponse {
        success: true,
        message: "Cursor account switched successfully".to_string(),
    }))
}

async fn delete_account(
    State(state): State<AppState>,
    headers: HeaderMap,
    AxumPath((platform, account)): AxumPath<(String, String)>,
) -> Result<Json<ActionResponse>, ApiError> {
    authorize(&state, &headers)?;
    let definition = find_provider(&platform)?;
    let account = validate_account_id(&account)?;
    let data_dir = data_dir()?;
    let detail = data_dir
        .join(definition.directory)
        .join(format!("{account}.json"));
    if !detail.is_file() {
        return Err(api_error(StatusCode::NOT_FOUND, "account not found"));
    }
    let trash = data_dir.join("web-trash").join(definition.id);
    fs::create_dir_all(&trash).map_err(internal_error)?;
    let destination = trash.join(format!(
        "{}-{account}.json",
        chrono::Utc::now().timestamp_millis()
    ));
    fs::rename(&detail, destination).map_err(internal_error)?;
    update_index(definition, account, None)?;
    Ok(Json(ActionResponse {
        success: true,
        message: format!("{} account moved to web-trash", definition.label),
    }))
}

async fn update_tags(
    State(state): State<AppState>,
    headers: HeaderMap,
    AxumPath((platform, account)): AxumPath<(String, String)>,
    Json(request): Json<TagsRequest>,
) -> Result<Json<ActionResponse>, ApiError> {
    authorize(&state, &headers)?;
    let definition = find_provider(&platform)?;
    let account = validate_account_id(&account)?;
    let tags = normalize_tags(request.tags);
    let detail = data_dir()?
        .join(definition.directory)
        .join(format!("{account}.json"));
    let mut value = read_json(&detail)?;
    value
        .as_object_mut()
        .ok_or_else(|| api_error(StatusCode::INTERNAL_SERVER_ERROR, "invalid account file"))?
        .insert(
            "tags".to_string(),
            Value::Array(tags.iter().cloned().map(Value::String).collect()),
        );
    write_json_atomic(&detail, &value)?;
    update_index(definition, account, Some(&tags))?;
    Ok(Json(ActionResponse {
        success: true,
        message: "Tags updated".to_string(),
    }))
}

fn load_account_values(definition: &ProviderDefinition) -> Result<Vec<Value>, ApiError> {
    let directory = data_dir()?.join(definition.directory);
    if !directory.is_dir() {
        return Ok(Vec::new());
    }
    let mut values = Vec::new();
    for entry in fs::read_dir(directory).map_err(internal_error)? {
        let path = entry.map_err(internal_error)?.path();
        if path.extension().and_then(|value| value.to_str()) != Some("json") {
            continue;
        }
        if let Ok(value) = read_json(&path) {
            values.push(value);
        }
    }
    values.sort_by_key(|value| {
        std::cmp::Reverse(
            number(value, &["last_used", "updated_at", "created_at"]).unwrap_or_default(),
        )
    });
    Ok(values)
}

fn account_view(definition: &ProviderDefinition, value: &Value) -> Option<AccountView> {
    let id = text(
        value,
        &["id", "account_id", "user_id", "github_login", "email"],
    )?;
    let email = text(
        value,
        &[
            "email",
            "github_email",
            "user_email",
            "account_email",
            "login",
        ],
    )
    .unwrap_or_else(|| id.clone());
    Some(AccountView {
        id,
        email,
        name: text(
            value,
            &[
                "name",
                "github_name",
                "display_name",
                "username",
                "github_login",
            ],
        ),
        plan: text(
            value,
            &[
                "membership_type",
                "copilot_plan",
                "plan",
                "plan_name",
                "subscription_type",
                "sku",
            ],
        ),
        tags: strings(value.get("tags")),
        status: text(value, &["status", "subscription_status", "state"]),
        last_used: number(value, &["last_used", "updated_at", "created_at"]).unwrap_or_default(),
        can_switch: definition.can_switch,
    })
}

fn provider_matches(provider: &str, value: &Value) -> bool {
    if !provider.starts_with("trae") {
        return true;
    }
    let marker = text(value, &["platform", "product", "variant"])
        .unwrap_or_default()
        .to_ascii_lowercase();
    match provider {
        "trae-solo-cn" => marker.contains("solo") && marker.contains("cn"),
        "trae-solo" => marker.contains("solo") && !marker.contains("cn"),
        "trae-cn" => marker.contains("cn") && !marker.contains("solo"),
        "trae" => marker.is_empty() || (!marker.contains("solo") && !marker.contains("cn")),
        _ => true,
    }
}

fn update_index(
    definition: &ProviderDefinition,
    account_id: &str,
    tags: Option<&[String]>,
) -> Result<(), ApiError> {
    let path = data_dir()?.join(definition.index_file);
    if !path.is_file() {
        return Ok(());
    }
    let mut value = read_json(&path)?;
    let Some(accounts) = value.get_mut("accounts").and_then(Value::as_array_mut) else {
        return Ok(());
    };
    if let Some(tags) = tags {
        for account in accounts
            .iter_mut()
            .filter(|value| text(value, &["id", "account_id"]).as_deref() == Some(account_id))
        {
            if let Some(object) = account.as_object_mut() {
                object.insert(
                    "tags".to_string(),
                    Value::Array(tags.iter().cloned().map(Value::String).collect()),
                );
            }
        }
    } else {
        accounts.retain(|value| text(value, &["id", "account_id"]).as_deref() != Some(account_id));
    }
    write_json_atomic(&path, &value)
}

fn read_json(path: &Path) -> Result<Value, ApiError> {
    let content = fs::read_to_string(path).map_err(internal_error)?;
    serde_json::from_str(&content)
        .map_err(|error| api_error(StatusCode::INTERNAL_SERVER_ERROR, error.to_string()))
}

fn write_json_atomic(path: &Path, value: &Value) -> Result<(), ApiError> {
    let content = serde_json::to_vec_pretty(value).map_err(internal_error)?;
    let temporary = path.with_extension("json.web.tmp");
    fs::write(&temporary, content).map_err(internal_error)?;
    fs::rename(temporary, path).map_err(internal_error)
}

fn text(value: &Value, keys: &[&str]) -> Option<String> {
    keys.iter().find_map(|key| {
        value
            .get(*key)
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string)
    })
}

fn number(value: &Value, keys: &[&str]) -> Option<i64> {
    keys.iter().find_map(|key| {
        value.get(*key).and_then(|value| {
            value
                .as_i64()
                .or_else(|| value.as_u64().and_then(|value| i64::try_from(value).ok()))
        })
    })
}

fn strings(value: Option<&Value>) -> Vec<String> {
    value
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
}

fn normalize_tags(tags: Vec<String>) -> Vec<String> {
    let mut output = Vec::new();
    for tag in tags {
        let tag = tag.trim();
        if !tag.is_empty()
            && !output
                .iter()
                .any(|current: &String| current.eq_ignore_ascii_case(tag))
        {
            output.push(tag.to_string());
        }
    }
    output
}

fn find_provider(platform: &str) -> Result<&'static ProviderDefinition, ApiError> {
    let platform = platform.trim().to_ascii_lowercase().replace('_', "-");
    PROVIDERS
        .iter()
        .find(|provider| provider.id == platform)
        .ok_or_else(|| api_error(StatusCode::BAD_REQUEST, "unsupported provider"))
}

fn validate_account_id(account: &str) -> Result<&str, ApiError> {
    let account = account.trim();
    if account.is_empty()
        || account.contains('/')
        || account.contains('\\')
        || account.contains("..")
    {
        Err(api_error(StatusCode::BAD_REQUEST, "invalid account id"))
    } else {
        Ok(account)
    }
}

fn data_dir() -> Result<PathBuf, ApiError> {
    config::get_data_dir().map_err(|error| api_error(StatusCode::INTERNAL_SERVER_ERROR, error))
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

fn internal_error(error: impl std::fmt::Display) -> ApiError {
    api_error(StatusCode::INTERNAL_SERVER_ERROR, error.to_string())
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
        .find(|path| path.join("index.html").is_file())
        .map(|path| path.canonicalize().unwrap_or(path))
        .context(
            "Cockpit WebUI files not found; extract webui next to the binary or pass --web-root",
        )
}

async fn shutdown_signal() {
    let _ = tokio::signal::ctrl_c().await;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exposes_all_desktop_providers() {
        assert_eq!(PROVIDERS.len(), 18);
        let serialized = serde_json::to_string(&PROVIDERS).unwrap();
        assert!(!serialized.contains("cursor_accounts"));
        assert!(!serialized.contains("index_file"));
    }

    #[test]
    fn rejects_path_traversal_account_ids() {
        assert!(validate_account_id("../secret").is_err());
        assert!(validate_account_id("safe-id").is_ok());
    }

    #[test]
    fn normalizes_duplicate_tags() {
        assert_eq!(
            normalize_tags(vec![" Work ".into(), "work".into(), "VIP".into()]),
            vec!["Work", "VIP"]
        );
    }

    #[test]
    fn extracts_only_safe_account_metadata() {
        let value = serde_json::json!({"id":"one","email":"a@example.com","access_token":"secret","tags":["vip"]});
        let view = account_view(find_provider("cursor").unwrap(), &value).unwrap();
        let serialized = serde_json::to_string(&view).unwrap();
        assert!(!serialized.contains("secret"));
        assert!(serialized.contains("a@example.com"));
    }
}
