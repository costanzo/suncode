mod agent;
mod config;
mod context;
mod credentials;
mod discovery;
mod domain;
mod llm;
mod model_provider;
mod persistence;
mod policy;
mod tools;

use agent::{Agent, AgentError, TurnResponse};
use axum::{
    body::Body,
    extract::{Path, Query, State},
    http::{header, HeaderMap, Request, StatusCode},
    response::{
        sse::{Event, KeepAlive, Sse},
        IntoResponse,
    },
    routing::{get, post},
    Json, Router,
};
use config::Config;
use credentials::CredentialStore;
use domain::{ProjectRecord, SessionEvent, SessionRecord};
use http_body_util::BodyExt;
use persistence::{PersistenceError, Store};
use rand::{distr::Alphanumeric, Rng};
use serde::Deserialize;
use serde_json::{json, Value};
use std::{
    convert::Infallible,
    ffi::{CStr, CString},
    os::raw::{c_char, c_void},
    ptr,
    sync::{Arc, Mutex},
    thread::JoinHandle,
};
use tokio::sync::broadcast;
use tokio_util::sync::CancellationToken;
use tower::ServiceExt;
use tower_http::trace::TraceLayer;

#[derive(Clone)]
struct AppState {
    store: Store,
    operations: Arc<suncode_operations::Operations>,
    token: Arc<String>,
    active_project: Arc<Mutex<Option<String>>>,
    events: broadcast::Sender<SessionEvent>,
    credentials: CredentialStore,
    agent: Agent,
    providers: Arc<model_provider::ModelProviderRegistry>,
}

#[derive(Debug, Deserialize)]
struct EventQuery {
    after: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct ProjectInput {
    path: String,
    display_name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SessionInput {
    title: Option<String>,
    model: Option<String>,
}

#[derive(Debug, Deserialize)]
struct TurnInput {
    input: Value,
    idempotency_key: String,
    model: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ApprovalInput {
    decision: String,
}

#[derive(Debug, Deserialize)]
struct CredentialInput {
    api_key: String,
}

#[derive(Debug, Deserialize)]
struct RenameInput {
    title: String,
}

#[derive(Debug, Deserialize)]
struct RestoreInput {
    session_id: String,
}

#[derive(Debug, Deserialize)]
struct SettingsQuery {
    project_id: Option<String>,
    session_id: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SettingInput {
    scope: String,
    project_id: Option<String>,
    session_id: Option<String>,
    key: String,
    value: Value,
}

pub async fn run_http_adapter() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    let config = Config::load().map_err(std::io::Error::other)?;
    let mut discovery = discovery::RuntimeDiscovery::acquire(&config.data_dir)?;
    let token = std::env::var("SUNCODE_RUNTIME_TOKEN").unwrap_or_else(|_| random_token());
    let state = build_state(&config, token.clone()).await?;
    let router = app(state.clone());
    let listener = tokio::net::TcpListener::bind((config.host.as_str(), config.port)).await?;
    let address = listener.local_addr()?;
    let endpoint = format!("http://{address}");
    discovery.publish(endpoint.clone(), token)?;
    println!("SUNCODE_RUNTIME_URL={endpoint}");
    axum::serve(listener, router)
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    Ok(())
}

async fn build_state(
    config: &Config,
    token: String,
) -> Result<AppState, Box<dyn std::error::Error>> {
    let store = Store::open(&config.database_path)?;
    let operations = Arc::new(suncode_operations::Operations::new(
        config.data_dir.join("operations"),
    )?);
    let (events, _) = broadcast::channel(256);
    let credentials = CredentialStore::load(store.clone(), config.non_interactive);
    let providers = Arc::new(model_provider::ModelProviderRegistry::new(
        config.deepseek_endpoint.clone(),
        config.deepseek_model.clone(),
        config.zhipu_endpoint.clone(),
        config.zhipu_model.clone(),
        config.openai_endpoint.clone(),
        config.openai_model.clone(),
        credentials.clone(),
    ));
    let agent = Agent::new(
        store.clone(),
        providers.clone(),
        operations.clone(),
        events.clone(),
        config.non_interactive,
    );
    let state = AppState {
        store,
        operations,
        token: Arc::new(token),
        active_project: Arc::new(Mutex::new(None)),
        events,
        credentials,
        agent,
        providers,
    };
    state
        .agent
        .recover()
        .await
        .map_err(|error| std::io::Error::other(error.message))?;
    Ok(state)
}

async fn shutdown_signal() {
    #[cfg(unix)]
    {
        let mut terminate =
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
                .expect("termination signal handler");
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {},
            _ = terminate.recv() => {},
        }
    }
    #[cfg(not(unix))]
    {
        let _ = tokio::signal::ctrl_c().await;
    }
}

fn app(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/diagnostics", get(diagnostics))
        .route("/models", get(models))
        .route("/settings", get(settings).put(set_setting))
        .route("/credentials", get(credentials))
        .route(
            "/credentials/deepseek",
            post(set_deepseek).delete(delete_deepseek),
        )
        .route(
            "/credentials/{provider}",
            post(set_credential).delete(delete_credential),
        )
        .route("/projects", get(projects).post(open_project))
        .route("/projects/{project_id}/open", post(reopen_project))
        .route(
            "/projects/{project_id}/sessions",
            get(project_sessions).post(create_session),
        )
        .route(
            "/sessions/{session_id}",
            axum::routing::patch(rename_session).delete(archive_session),
        )
        .route("/sessions/{session_id}/reopen", post(reopen_session))
        .route(
            "/sessions/{session_id}/checkpoints",
            get(session_checkpoints),
        )
        .route("/checkpoints/{manifest_id}", get(checkpoint_manifest))
        .route(
            "/checkpoints/{manifest_id}/restore",
            post(restore_checkpoint),
        )
        .route("/sessions/{session_id}/turns", post(start_turn))
        .route(
            "/sessions/{session_id}/turns/{turn_id}/cancel",
            post(cancel_turn),
        )
        .route("/sessions/{session_id}/events", get(session_events))
        .route("/sessions/{session_id}/snapshot", get(session_snapshot))
        .route(
            "/sessions/{session_id}/events/stream",
            get(session_event_stream),
        )
        .route(
            "/approvals/{approval_id}",
            get(get_approval).post(resolve_approval),
        )
        .with_state(state)
        .layer(TraceLayer::new_for_http())
}

fn authorized(headers: &HeaderMap, state: &AppState) -> Result<(), ApiError> {
    let Some(value) = headers
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
    else {
        return Err(ApiError::new(
            StatusCode::UNAUTHORIZED,
            "unauthenticated",
            "runtime credential required",
        ));
    };
    let Some(provided) = value.strip_prefix("Bearer ") else {
        return Err(ApiError::new(
            StatusCode::UNAUTHORIZED,
            "unauthenticated",
            "runtime credential required",
        ));
    };
    if provided.as_bytes() != state.token.as_bytes() {
        return Err(ApiError::new(
            StatusCode::UNAUTHORIZED,
            "unauthenticated",
            "runtime credential required",
        ));
    }
    Ok(())
}

async fn health(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Value>, ApiError> {
    authorized(&headers, &state)?;
    Ok(Json(
        json!({"ok": true, "runtime": "ready", "database": state.store.health().map_err(ApiError::from)?}),
    ))
}

async fn diagnostics(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Value>, ApiError> {
    authorized(&headers, &state)?;
    let database = state.store.health().map_err(ApiError::from)?;
    Ok(Json(
        json!({"health": {"ok": true, "runtime": "ready", "database": database}, "recovery": {"status": "ready", "pending_operations": 0}, "credentials": state.credentials.state(), "active_project_id": state.active_project.lock().ok().and_then(|value| value.clone())}),
    ))
}

async fn models(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Value>, ApiError> {
    authorized(&headers, &state)?;
    let mut models = state.providers.models();
    for model in &mut models {
        let configured = match model.provider {
            "deepseek" => state
                .credentials
                .configured(credentials::ProviderKind::DeepSeek),
            "zhipu" => state
                .credentials
                .configured(credentials::ProviderKind::Zhipu),
            "openai" => state
                .credentials
                .configured(credentials::ProviderKind::OpenAI),
            _ => false,
        };
        model.availability = if configured {
            "configured"
        } else {
            "unconfigured"
        };
    }
    Ok(Json(json!({"models": models})))
}

async fn credentials(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Value>, ApiError> {
    authorized(&headers, &state)?;
    Ok(Json(json!({"credentials": state.credentials.state()})))
}

async fn settings(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<SettingsQuery>,
) -> Result<Json<Value>, ApiError> {
    authorized(&headers, &state)?;
    Ok(Json(
        json!({"settings": state.store.settings(query.project_id.as_deref(), query.session_id.as_deref()).map_err(ApiError::from)?}),
    ))
}

async fn set_setting(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<SettingInput>,
) -> Result<Json<Value>, ApiError> {
    authorized(&headers, &state)?;
    let scope_id =
        match input.scope.as_str() {
            "user" => "default",
            "project" => input.project_id.as_deref().ok_or_else(|| {
                ApiError::bad_request("invalid_arguments", "project_id is required")
            })?,
            "session" => input.session_id.as_deref().ok_or_else(|| {
                ApiError::bad_request("invalid_arguments", "session_id is required")
            })?,
            _ => {
                return Err(ApiError::bad_request(
                    "invalid_arguments",
                    "scope is invalid",
                ))
            }
        };
    state
        .store
        .set_setting(&input.scope, scope_id, &input.key, &input.value)
        .map_err(ApiError::from)?;
    Ok(Json(
        json!({"saved":true,"key":input.key,"scope":input.scope,"scope_id":scope_id}),
    ))
}

async fn set_deepseek(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<CredentialInput>,
) -> Result<Json<Value>, ApiError> {
    authorized(&headers, &state)?;
    state
        .credentials
        .set(credentials::ProviderKind::DeepSeek, &input.api_key)
        .map_err(|error| {
            ApiError::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                "credential_unavailable",
                &error,
            )
        })?;
    Ok(Json(json!({"provider":"deepseek","configured":true})))
}
async fn delete_deepseek(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Value>, ApiError> {
    authorized(&headers, &state)?;
    state
        .credentials
        .delete(credentials::ProviderKind::DeepSeek)
        .map_err(|error| {
            ApiError::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                "credential_unavailable",
                &error,
            )
        })?;
    Ok(Json(json!({"provider":"deepseek","configured":false})))
}

async fn set_credential(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(provider): Path<String>,
    Json(input): Json<CredentialInput>,
) -> Result<Json<Value>, ApiError> {
    authorized(&headers, &state)?;
    let provider = parse_provider(&provider)?;
    state
        .credentials
        .set(provider, &input.api_key)
        .map_err(|error| {
            ApiError::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                "credential_unavailable",
                &error,
            )
        })?;
    Ok(Json(
        json!({"provider": provider.as_str(), "configured": true}),
    ))
}

async fn delete_credential(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(provider): Path<String>,
) -> Result<Json<Value>, ApiError> {
    authorized(&headers, &state)?;
    let provider = parse_provider(&provider)?;
    state.credentials.delete(provider).map_err(|error| {
        ApiError::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "credential_unavailable",
            &error,
        )
    })?;
    Ok(Json(
        json!({"provider": provider.as_str(), "configured": false}),
    ))
}

fn parse_provider(provider: &str) -> Result<credentials::ProviderKind, ApiError> {
    match provider {
        "deepseek" => Ok(credentials::ProviderKind::DeepSeek),
        "zhipu" => Ok(credentials::ProviderKind::Zhipu),
        "openai" => Ok(credentials::ProviderKind::OpenAI),
        _ => Err(ApiError::bad_request(
            "invalid_arguments",
            "provider is not supported",
        )),
    }
}

async fn projects(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Value>, ApiError> {
    authorized(&headers, &state)?;
    Ok(Json(
        json!({"projects": state.store.projects(false).map_err(ApiError::from)?}),
    ))
}

async fn open_project(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<ProjectInput>,
) -> Result<(StatusCode, Json<ProjectRecord>), ApiError> {
    authorized(&headers, &state)?;
    let result = state
        .operations
        .open_project(std::path::Path::new(&input.path))
        .map_err(operation_error)?;
    let root = result
        .get("canonical_path")
        .and_then(Value::as_str)
        .ok_or_else(|| ApiError::internal("project/open did not return a canonical path"))?;
    let display_name = input
        .display_name
        .as_deref()
        .or_else(|| result.get("display_name").and_then(Value::as_str))
        .unwrap_or("Project");
    let project = state
        .store
        .project(root, display_name)
        .map_err(ApiError::from)?;
    if let Ok(mut active) = state.active_project.lock() {
        *active = Some(project.project_id.clone());
    }
    Ok((StatusCode::CREATED, Json(project)))
}

async fn reopen_project(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(project_id): Path<String>,
) -> Result<Json<ProjectRecord>, ApiError> {
    authorized(&headers, &state)?;
    let project = state
        .store
        .project_by_id(&project_id)
        .map_err(ApiError::from)?
        .ok_or_else(|| ApiError::not_found("project not found"))?;
    state
        .operations
        .open_project(std::path::Path::new(&project.canonical_root))
        .map_err(operation_error)?;
    if let Ok(mut active) = state.active_project.lock() {
        *active = Some(project.project_id.clone());
    }
    Ok(Json(project))
}

async fn project_sessions(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(project_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    authorized(&headers, &state)?;
    if state
        .store
        .project_by_id(&project_id)
        .map_err(ApiError::from)?
        .is_none()
    {
        return Err(ApiError::not_found("project not found"));
    }
    Ok(Json(
        json!({"project_id": project_id, "sessions": state.store.sessions_for_project(&project_id, true).map_err(ApiError::from)?}),
    ))
}

async fn create_session(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(project_id): Path<String>,
    Json(input): Json<SessionInput>,
) -> Result<(StatusCode, Json<SessionRecord>), ApiError> {
    authorized(&headers, &state)?;
    if let Some(model) = &input.model {
        if state.providers.provider(model).is_none() {
            return Err(ApiError::bad_request(
                "model_unavailable",
                "model is not advertised",
            ));
        }
    }
    let session = state
        .store
        .create_session(&project_id, input.title.as_deref(), input.model.as_deref())
        .map_err(ApiError::from)?;
    Ok((StatusCode::CREATED, Json(session)))
}

async fn rename_session(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(session_id): Path<String>,
    Json(input): Json<RenameInput>,
) -> Result<Json<SessionRecord>, ApiError> {
    authorized(&headers, &state)?;
    if input.title.trim().is_empty() {
        return Err(ApiError::bad_request(
            "invalid_arguments",
            "title is required",
        ));
    }
    Ok(Json(
        state
            .store
            .rename_session(&session_id, input.title.trim())
            .map_err(ApiError::from)?,
    ))
}
async fn archive_session(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(session_id): Path<String>,
) -> Result<Json<SessionRecord>, ApiError> {
    authorized(&headers, &state)?;
    Ok(Json(
        state
            .store
            .set_session_archived(&session_id, true)
            .map_err(ApiError::from)?,
    ))
}
async fn reopen_session(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(session_id): Path<String>,
) -> Result<Json<SessionRecord>, ApiError> {
    authorized(&headers, &state)?;
    Ok(Json(
        state
            .store
            .set_session_archived(&session_id, false)
            .map_err(ApiError::from)?,
    ))
}

async fn start_turn(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(session_id): Path<String>,
    Json(input): Json<TurnInput>,
) -> Result<(StatusCode, Json<Value>), ApiError> {
    authorized(&headers, &state)?;
    let text = input
        .input
        .as_str()
        .or_else(|| {
            input
                .input
                .pointer("/content/0/text")
                .and_then(Value::as_str)
        })
        .ok_or_else(|| ApiError::bad_request("invalid_arguments", "input is required"))?;
    if input.idempotency_key.is_empty() {
        return Err(ApiError::bad_request(
            "invalid_arguments",
            "idempotency_key is required",
        ));
    }
    match state
        .agent
        .submit(
            &session_id,
            &input.idempotency_key,
            text,
            input.model.as_deref(),
        )
        .await
    {
        Ok(response @ TurnResponse::Completed { .. }) => Ok((
            StatusCode::OK,
            Json(serde_json::to_value(response).unwrap_or(Value::Null)),
        )),
        Ok(response @ TurnResponse::AwaitingApproval { .. }) => Ok((
            StatusCode::ACCEPTED,
            Json(serde_json::to_value(response).unwrap_or(Value::Null)),
        )),
        Err(error) if error.code == "approval_required" => Ok((
            StatusCode::ACCEPTED,
            Json(
                json!({"status":"awaiting_approval","turn_id":error.details.get("turn_id"),"tool_call_id":error.details.get("tool_call_id"),"approval_id":error.details.get("approval_id")}),
            ),
        )),
        Err(error) => Err(agent_error(error)),
    }
}
async fn cancel_turn(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((session_id, turn_id)): Path<(String, String)>,
) -> Result<(StatusCode, Json<Value>), ApiError> {
    authorized(&headers, &state)?;
    let _ = session_id;
    if !state.agent.cancel(&turn_id) {
        return Err(ApiError::new(
            StatusCode::CONFLICT,
            "conflict",
            "turn is not running",
        ));
    }
    Ok((
        StatusCode::ACCEPTED,
        Json(json!({"turn_id":turn_id,"status":"cancellation_requested"})),
    ))
}
async fn get_approval(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(approval_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    authorized(&headers, &state)?;
    let value = state
        .store
        .approval(&approval_id)
        .map_err(ApiError::from)?
        .ok_or_else(|| ApiError::not_found("approval not found"))?;
    Ok(Json(serde_json::to_value(value).unwrap_or(Value::Null)))
}
async fn resolve_approval(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(approval_id): Path<String>,
    Json(input): Json<ApprovalInput>,
) -> Result<Json<Value>, ApiError> {
    authorized(&headers, &state)?;
    if !["deny", "allow_once"].contains(&input.decision.as_str()) {
        return Err(ApiError::bad_request(
            "invalid_arguments",
            "invalid approval decision",
        ));
    }
    if !state
        .agent
        .resolve_approval(&approval_id, &input.decision)
        .await
        .map_err(agent_error)?
    {
        return Err(ApiError::new(
            StatusCode::CONFLICT,
            "conflict",
            "approval is missing or already resolved",
        ));
    }
    Ok(Json(
        json!({"approval_id":approval_id,"decision":input.decision}),
    ))
}

async fn session_checkpoints(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(session_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    authorized(&headers, &state)?;
    Ok(Json(
        json!({"session_id":session_id,"checkpoints":state.store.manifests(&session_id).map_err(ApiError::from)?}),
    ))
}
async fn checkpoint_manifest(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(manifest_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    authorized(&headers, &state)?;
    let manifest = state
        .store
        .manifest(&manifest_id)
        .map_err(ApiError::from)?
        .ok_or_else(|| ApiError::not_found("checkpoint not found"))?;
    Ok(Json(
        json!({"manifest":manifest,"items":state.store.checkpoint_items(&manifest_id).map_err(ApiError::from)?}),
    ))
}
async fn restore_checkpoint(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(manifest_id): Path<String>,
    Json(input): Json<RestoreInput>,
) -> Result<Json<Value>, ApiError> {
    authorized(&headers, &state)?;
    let manifest = state
        .store
        .manifest(&manifest_id)
        .map_err(ApiError::from)?
        .ok_or_else(|| ApiError::not_found("checkpoint not found"))?;
    if manifest.session_id != input.session_id {
        return Err(ApiError::new(
            StatusCode::FORBIDDEN,
            "scope_denied",
            "checkpoint does not belong to session",
        ));
    }
    if manifest.status != "available" {
        return Err(ApiError::new(
            StatusCode::CONFLICT,
            "checkpoint_unavailable",
            "checkpoint is not available",
        ));
    }
    let session = state
        .store
        .session_by_id(&input.session_id)
        .map_err(ApiError::from)?
        .ok_or_else(|| ApiError::not_found("session not found"))?;
    let project = state
        .store
        .project_by_id(session.project_id.as_deref().unwrap_or(""))
        .map_err(ApiError::from)?
        .ok_or_else(|| ApiError::not_found("project not found"))?;
    state
        .store
        .set_manifest_status(&manifest_id, "restoring")
        .map_err(ApiError::from)?;
    let items = state
        .store
        .checkpoint_items(&manifest_id)
        .map_err(ApiError::from)?;
    let mut restored = 0;
    for item in &items {
        if item.status != "available" {
            continue;
        }
        match state.operations.execute_in_project(
            std::path::Path::new(&project.canonical_root),
            "checkpoint/restore",
            json!({"checkpoint_id":item.checkpoint_id}),
        ) {
            Ok(result) => {
                restored += 1;
                emit_event(
                    &state,
                    &input.session_id,
                    "checkpoint.item_restored",
                    json!({"manifest_id":manifest_id,"checkpoint_id":item.checkpoint_id,"path":result.get("path")}),
                )?
            }
            Err(error) => {
                let status = if restored > 0 { "partial" } else { "conflict" };
                state
                    .store
                    .set_manifest_status(&manifest_id, status)
                    .map_err(ApiError::from)?;
                emit_event(
                    &state,
                    &input.session_id,
                    "checkpoint.restore_failed",
                    json!({"manifest_id":manifest_id,"status":status,"code":error.get("code")}),
                )?;
                return Err(ApiError::new(
                    StatusCode::CONFLICT,
                    error
                        .get("code")
                        .and_then(Value::as_str)
                        .unwrap_or("restore_conflict"),
                    error
                        .get("message")
                        .and_then(Value::as_str)
                        .unwrap_or("checkpoint restore failed"),
                ));
            }
        }
    }
    state
        .store
        .set_manifest_status(&manifest_id, "restored")
        .map_err(ApiError::from)?;
    emit_event(
        &state,
        &input.session_id,
        "checkpoint.restored",
        json!({"manifest_id":manifest_id,"restored_items":restored}),
    )?;
    Ok(Json(
        json!({"manifest_id":manifest_id,"status":"restored","restored_items":restored}),
    ))
}

async fn session_events(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(session_id): Path<String>,
    Query(query): Query<EventQuery>,
) -> Result<Json<Value>, ApiError> {
    authorized(&headers, &state)?;
    if state
        .store
        .session_by_id(&session_id)
        .map_err(ApiError::from)?
        .is_none()
    {
        return Err(ApiError::not_found("session not found"));
    }
    let events = state
        .store
        .events(&session_id, query.after.unwrap_or(0))
        .map_err(ApiError::from)?;
    Ok(Json(json!({"session_id": session_id, "events": events})))
}

async fn session_snapshot(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(session_id): Path<String>,
    Query(query): Query<EventQuery>,
) -> Result<Json<Value>, ApiError> {
    authorized(&headers, &state)?;
    let session = state
        .store
        .session_by_id(&session_id)
        .map_err(ApiError::from)?
        .ok_or_else(|| ApiError::not_found("session not found"))?;
    let events = state
        .store
        .events(&session_id, query.after.unwrap_or(0))
        .map_err(ApiError::from)?;
    let latest_sequence = events
        .last()
        .map(|event| event.content_sequence)
        .unwrap_or(query.after.unwrap_or(0));
    let messages = state.store.messages(&session_id).map_err(ApiError::from)?;
    Ok(Json(
        json!({"session":session,"messages":messages,"events":events,"latest_sequence":latest_sequence,"replay_available":true}),
    ))
}

async fn session_event_stream(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(session_id): Path<String>,
    Query(query): Query<EventQuery>,
) -> Result<Sse<impl futures_util::Stream<Item = Result<Event, Infallible>>>, ApiError> {
    authorized(&headers, &state)?;
    if state
        .store
        .session_by_id(&session_id)
        .map_err(ApiError::from)?
        .is_none()
    {
        return Err(ApiError::not_found("session not found"));
    }
    let after = query.after.unwrap_or(0);
    let replay = state
        .store
        .events(&session_id, after)
        .map_err(ApiError::from)?;
    let mut receiver = state.events.subscribe();
    let stream = async_stream::stream! {
        for event in replay { yield Ok(event_to_sse(event)); }
        loop {
            match receiver.recv().await {
                Ok(event) if event.session_id == session_id => yield Ok(event_to_sse(event)),
                Ok(_) => {},
                Err(broadcast::error::RecvError::Lagged(_)) => {},
                Err(broadcast::error::RecvError::Closed) => break,
            }
        }
    };
    Ok(Sse::new(stream).keep_alive(KeepAlive::default()))
}

fn event_to_sse(event: SessionEvent) -> Event {
    Event::default()
        .id(event.content_sequence.to_string())
        .json_data(event)
        .unwrap_or_else(|_| {
            Event::default()
                .event("error")
                .data("event serialization failed")
        })
}
fn random_token() -> String {
    rand::rng()
        .sample_iter(&Alphanumeric)
        .take(64)
        .map(char::from)
        .collect()
}

#[derive(Debug, Clone)]
pub struct RuntimeResponse {
    pub status: u16,
    pub body: Value,
}

impl RuntimeResponse {
    fn envelope(self) -> Value {
        if (200..300).contains(&self.status) {
            json!({"ok": true, "status": self.status, "body": self.body})
        } else {
            json!({"ok": false, "status": self.status, "error": self.body})
        }
    }
}

pub struct RuntimeSdk {
    _discovery: Option<discovery::RuntimeDiscovery>,
    runtime: tokio::runtime::Runtime,
    state: AppState,
    router: Router,
}

impl RuntimeSdk {
    pub fn open_default() -> Result<Self, String> {
        let config = Config::load()?;
        let discovery = discovery::RuntimeDiscovery::acquire(&config.data_dir)
            .map_err(|error| format!("runtime lock unavailable: {error}"))?;
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .thread_name("suncode-sdk")
            .build()
            .map_err(|error| format!("tokio runtime unavailable: {error}"))?;
        let state = runtime
            .block_on(build_state(&config, random_token()))
            .map_err(|error| error.to_string())?;
        let router = app(state.clone());
        Ok(Self {
            _discovery: Some(discovery),
            runtime,
            state,
            router,
        })
    }

    pub fn request_json(&self, method: &str, path: &str, body: &Value) -> RuntimeResponse {
        if path.contains("/events/stream") {
            return RuntimeResponse {
                status: StatusCode::BAD_REQUEST.as_u16(),
                body: json!({
                    "code": "invalid_arguments",
                    "message": "use the SDK event subscription API for live events"
                }),
            };
        }
        let request = match Request::builder()
            .method(method)
            .uri(path)
            .header(
                header::AUTHORIZATION,
                format!("Bearer {}", self.state.token),
            )
            .header(header::CONTENT_TYPE, "application/json")
            .header(header::ACCEPT, "application/json")
            .body(Body::from(body.to_string()))
        {
            Ok(request) => request,
            Err(error) => {
                return RuntimeResponse {
                    status: StatusCode::BAD_REQUEST.as_u16(),
                    body: json!({"code":"invalid_arguments","message":error.to_string()}),
                }
            }
        };
        self.runtime.block_on(async {
            let response = match self.router.clone().oneshot(request).await {
                Ok(response) => response,
                Err(error) => {
                    return RuntimeResponse {
                        status: StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
                        body: json!({"code":"runtime_unavailable","message":format!("{error:?}")}),
                    }
                }
            };
            let status = response.status().as_u16();
            let bytes = match response.into_body().collect().await {
                Ok(value) => value.to_bytes(),
                Err(error) => {
                    return RuntimeResponse {
                        status: StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
                        body: json!({"code":"runtime_unavailable","message":error.to_string()}),
                    }
                }
            };
            let body = if bytes.is_empty() {
                json!({})
            } else {
                serde_json::from_slice(&bytes)
                    .unwrap_or_else(|_| json!({"raw": String::from_utf8_lossy(&bytes)}))
            };
            RuntimeResponse { status, body }
        })
    }

    pub fn subscribe_session_events(
        &self,
        session_id: String,
        after: i64,
        callback: SuncodeEventCallback,
        user_data: *mut c_void,
    ) -> Result<RuntimeSubscription, String> {
        let replay = self
            .state
            .store
            .events(&session_id, after)
            .map_err(|error| error.to_string())?;
        let mut receiver = self.state.events.subscribe();
        let cancellation = CancellationToken::new();
        let cancellation_for_thread = cancellation.clone();
        let handle = self.runtime.handle().clone();
        let user_data = user_data as usize;
        let join = std::thread::spawn(move || {
            for event in replay {
                if cancellation_for_thread.is_cancelled() {
                    return;
                }
                emit_sdk_event(callback, user_data, &event);
            }
            loop {
                let next = handle.block_on(async {
                    tokio::select! {
                        _ = cancellation_for_thread.cancelled() => None,
                        value = receiver.recv() => Some(value),
                    }
                });
                match next {
                    None => break,
                    Some(Ok(event)) if event.session_id == session_id => {
                        emit_sdk_event(callback, user_data, &event)
                    }
                    Some(Ok(_)) => {}
                    Some(Err(broadcast::error::RecvError::Lagged(_))) => {}
                    Some(Err(broadcast::error::RecvError::Closed)) => break,
                }
            }
        });
        Ok(RuntimeSubscription {
            cancellation,
            join: Mutex::new(Some(join)),
        })
    }

    #[cfg(test)]
    fn from_state_for_test(state: AppState) -> Self {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .thread_name("suncode-sdk-test")
            .build()
            .unwrap();
        let router = app(state.clone());
        Self {
            _discovery: None,
            runtime,
            state,
            router,
        }
    }
}

pub type SuncodeEventCallback = unsafe extern "C" fn(*const c_char, *mut c_void);

pub struct RuntimeSubscription {
    cancellation: CancellationToken,
    join: Mutex<Option<JoinHandle<()>>>,
}

impl RuntimeSubscription {
    fn close(&self) {
        self.cancellation.cancel();
        if let Ok(mut join) = self.join.lock() {
            if let Some(join) = join.take() {
                let _ = join.join();
            }
        }
    }
}

impl Drop for RuntimeSubscription {
    fn drop(&mut self) {
        self.close();
    }
}

pub struct SuncodeRuntimeHandle {
    sdk: RuntimeSdk,
}

pub struct SuncodeRuntimeSubscriptionHandle {
    _subscription: RuntimeSubscription,
}

/// # Safety
///
/// `error_out`, when non-null, must be writable memory for one C string pointer. The returned
/// handle must be released with `suncode_runtime_sdk_close`.
#[no_mangle]
pub unsafe extern "C" fn suncode_runtime_sdk_open_default(
    error_out: *mut *mut c_char,
) -> *mut SuncodeRuntimeHandle {
    write_error_out(error_out, ptr::null_mut());
    match RuntimeSdk::open_default() {
        Ok(sdk) => Box::into_raw(Box::new(SuncodeRuntimeHandle { sdk })),
        Err(error) => {
            write_error_out(error_out, into_c_string(error));
            ptr::null_mut()
        }
    }
}

/// # Safety
///
/// `handle` must be a pointer returned by `suncode_runtime_sdk_open_default` and not already
/// released. It may be null.
#[no_mangle]
pub unsafe extern "C" fn suncode_runtime_sdk_close(handle: *mut SuncodeRuntimeHandle) {
    if !handle.is_null() {
        drop(Box::from_raw(handle));
    }
}

/// # Safety
///
/// `handle` must be a valid SDK handle. `method`, `path`, and `body_json`, when non-null, must be
/// valid nul-terminated UTF-8 strings. The returned string must be released with
/// `suncode_runtime_sdk_string_free`.
#[no_mangle]
pub unsafe extern "C" fn suncode_runtime_sdk_request_json(
    handle: *mut SuncodeRuntimeHandle,
    method: *const c_char,
    path: *const c_char,
    body_json: *const c_char,
) -> *mut c_char {
    let Some(handle) = handle.as_ref() else {
        return runtime_error_envelope(
            StatusCode::INTERNAL_SERVER_ERROR,
            "runtime_unavailable",
            "runtime handle is null",
        );
    };
    let method = match c_string(method, "method") {
        Ok(value) => value,
        Err(error) => {
            return runtime_error_envelope(StatusCode::BAD_REQUEST, "invalid_arguments", &error)
        }
    };
    let path = match c_string(path, "path") {
        Ok(value) => value,
        Err(error) => {
            return runtime_error_envelope(StatusCode::BAD_REQUEST, "invalid_arguments", &error)
        }
    };
    let body = if body_json.is_null() {
        json!({})
    } else {
        match CStr::from_ptr(body_json).to_str() {
            Ok("") => json!({}),
            Ok(value) => match serde_json::from_str::<Value>(value) {
                Ok(value) => value,
                Err(error) => {
                    return runtime_error_envelope(
                        StatusCode::BAD_REQUEST,
                        "invalid_arguments",
                        &format!("body_json is invalid: {error}"),
                    )
                }
            },
            Err(error) => {
                return runtime_error_envelope(
                    StatusCode::BAD_REQUEST,
                    "invalid_arguments",
                    &format!("body_json is not UTF-8: {error}"),
                )
            }
        }
    };
    into_c_string(
        handle
            .sdk
            .request_json(&method, &path, &body)
            .envelope()
            .to_string(),
    )
}

/// # Safety
///
/// `handle` must be a valid SDK handle. `session_id` must be a valid nul-terminated UTF-8 string.
/// `callback` must remain callable until the subscription is closed. `error_out`, when non-null,
/// must be writable memory for one C string pointer.
#[no_mangle]
pub unsafe extern "C" fn suncode_runtime_sdk_subscribe_session(
    handle: *mut SuncodeRuntimeHandle,
    session_id: *const c_char,
    after: i64,
    callback: Option<SuncodeEventCallback>,
    user_data: *mut c_void,
    error_out: *mut *mut c_char,
) -> *mut SuncodeRuntimeSubscriptionHandle {
    write_error_out(error_out, ptr::null_mut());
    let Some(handle) = handle.as_ref() else {
        write_error_out(
            error_out,
            into_c_string("runtime handle is null".to_string()),
        );
        return ptr::null_mut();
    };
    let Some(callback) = callback else {
        write_error_out(error_out, into_c_string("callback is null".to_string()));
        return ptr::null_mut();
    };
    let session_id = match c_string(session_id, "session_id") {
        Ok(value) => value,
        Err(error) => {
            write_error_out(error_out, into_c_string(error));
            return ptr::null_mut();
        }
    };
    match handle
        .sdk
        .subscribe_session_events(session_id, after, callback, user_data)
    {
        Ok(subscription) => Box::into_raw(Box::new(SuncodeRuntimeSubscriptionHandle {
            _subscription: subscription,
        })),
        Err(error) => {
            write_error_out(error_out, into_c_string(error));
            ptr::null_mut()
        }
    }
}

/// # Safety
///
/// `subscription` must be a pointer returned by `suncode_runtime_sdk_subscribe_session` and not
/// already released. It may be null.
#[no_mangle]
pub unsafe extern "C" fn suncode_runtime_sdk_subscription_close(
    subscription: *mut SuncodeRuntimeSubscriptionHandle,
) {
    if !subscription.is_null() {
        drop(Box::from_raw(subscription));
    }
}

/// # Safety
///
/// `value` must be a string pointer returned by this SDK. It may be null.
#[no_mangle]
pub unsafe extern "C" fn suncode_runtime_sdk_string_free(value: *mut c_char) {
    if !value.is_null() {
        drop(CString::from_raw(value));
    }
}

fn emit_sdk_event(callback: SuncodeEventCallback, user_data: usize, event: &SessionEvent) {
    let Ok(value) = serde_json::to_string(event) else {
        return;
    };
    let Ok(value) = CString::new(value) else {
        return;
    };
    unsafe { callback(value.as_ptr(), user_data as *mut c_void) };
}

fn c_string(pointer: *const c_char, name: &str) -> Result<String, String> {
    if pointer.is_null() {
        return Err(format!("{name} is null"));
    }
    unsafe {
        CStr::from_ptr(pointer)
            .to_str()
            .map(str::to_string)
            .map_err(|error| format!("{name} is not UTF-8: {error}"))
    }
}

unsafe fn write_error_out(error_out: *mut *mut c_char, value: *mut c_char) {
    if !error_out.is_null() {
        *error_out = value;
    }
}

fn into_c_string(value: String) -> *mut c_char {
    CString::new(value)
        .unwrap_or_else(|_| CString::new("string contained an interior nul byte").unwrap())
        .into_raw()
}

fn runtime_error_envelope(status: StatusCode, code: &str, message: &str) -> *mut c_char {
    into_c_string(
        json!({
            "ok": false,
            "status": status.as_u16(),
            "error": {"code": code, "message": message}
        })
        .to_string(),
    )
}

fn operation_error(error: Value) -> ApiError {
    ApiError::bad_request(
        error
            .get("code")
            .and_then(Value::as_str)
            .unwrap_or("operation_failed"),
        error
            .get("message")
            .and_then(Value::as_str)
            .unwrap_or("operation failed"),
    )
}
fn agent_error(error: AgentError) -> ApiError {
    let status = match error.code.as_str() {
        "not_found" => StatusCode::NOT_FOUND,
        "authorization_denied" | "scope_denied" => StatusCode::FORBIDDEN,
        "idempotency_conflict" | "conflict" | "checkpoint_unavailable" | "restore_conflict" => {
            StatusCode::CONFLICT
        }
        "invalid_arguments" | "model_unavailable" => StatusCode::BAD_REQUEST,
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    };
    ApiError {
        status,
        body: json!({"code":error.code,"message":error.message,"details":error.details}),
    }
}
fn emit_event(
    state: &AppState,
    session_id: &str,
    event_type: &str,
    payload: Value,
) -> Result<(), ApiError> {
    let event = state
        .store
        .append_content(session_id, event_type, &payload)
        .map_err(ApiError::from)?;
    let _ = state.events.send(event);
    Ok(())
}

#[derive(Debug)]
struct ApiError {
    status: StatusCode,
    body: Value,
}
impl ApiError {
    fn new(status: StatusCode, code: &str, message: &str) -> Self {
        Self {
            status,
            body: json!({"code": code, "message": message}),
        }
    }
    fn internal(message: &str) -> Self {
        Self::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "runtime_unavailable",
            message,
        )
    }
    fn bad_request(code: &str, message: &str) -> Self {
        Self::new(StatusCode::BAD_REQUEST, code, message)
    }
    fn not_found(message: &str) -> Self {
        Self::new(StatusCode::NOT_FOUND, "not_found", message)
    }
}
impl From<PersistenceError> for ApiError {
    fn from(error: PersistenceError) -> Self {
        Self::internal(&error.to_string())
    }
}
impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        (self.status, Json(self.body)).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Body, http::Request};
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    fn test_state(directory: &std::path::Path) -> AppState {
        let store = Store::open_memory().unwrap();
        let operations =
            Arc::new(suncode_operations::Operations::new(directory.join("operations")).unwrap());
        let credentials = CredentialStore::memory(Some("test-key"), None, None);
        let (events, _) = broadcast::channel(16);
        let providers = Arc::new(model_provider::ModelProviderRegistry::new(
            "http://127.0.0.1:1".into(),
            "deepseek-v4-flash".into(),
            "http://127.0.0.1:2".into(),
            "glm-5.2".into(),
            "http://127.0.0.1:3".into(),
            "gpt-5.6-sol".into(),
            credentials.clone(),
        ));
        let agent = Agent::new(
            store.clone(),
            providers.clone(),
            operations.clone(),
            events.clone(),
            false,
        );
        AppState {
            store,
            operations,
            token: Arc::new("test-token".into()),
            active_project: Arc::new(Mutex::new(None)),
            events,
            credentials,
            agent,
            providers,
        }
    }

    #[test]
    fn token_is_not_empty() {
        assert!(!random_token().is_empty());
    }
    #[test]
    fn operation_error_is_redacted_to_contract() {
        let error = operation_error(json!({"code":"scope_denied","message":"outside"}));
        assert_eq!(error.status, StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn router_authenticates_and_serves_project_session_dtos() {
        let directory = tempfile::tempdir().unwrap();
        let router = app(test_state(directory.path()));
        let credentials_response = router
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/credentials")
                    .header(header::AUTHORIZATION, "Bearer test-token")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(credentials_response.status(), StatusCode::OK);
        let credentials: Value = serde_json::from_slice(
            &credentials_response
                .into_body()
                .collect()
                .await
                .unwrap()
                .to_bytes(),
        )
        .unwrap();
        assert_eq!(credentials["credentials"].as_array().unwrap().len(), 3);

        let models_response = router
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/models")
                    .header(header::AUTHORIZATION, "Bearer test-token")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(models_response.status(), StatusCode::OK);
        let models: Value = serde_json::from_slice(
            &models_response
                .into_body()
                .collect()
                .await
                .unwrap()
                .to_bytes(),
        )
        .unwrap();
        let models = models["models"].as_array().unwrap();
        assert_eq!(models.len(), 3);
        assert!(models.iter().any(|model| model["id"] == "deepseek-v4-flash"
            && model["provider"] == "deepseek"
            && model["availability"] == "configured"));
        assert!(models.iter().any(|model| model["id"] == "glm-5.2"
            && model["provider"] == "zhipu"
            && model["availability"] == "unconfigured"));
        assert!(models.iter().any(|model| model["id"] == "gpt-5.6-sol"
            && model["provider"] == "openai"
            && model["availability"] == "unconfigured"));

        let unauthorized = router
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(unauthorized.status(), StatusCode::UNAUTHORIZED);

        let project_response = router
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/projects")
                    .header(header::AUTHORIZATION, "Bearer test-token")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(json!({"path":directory.path()}).to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(project_response.status(), StatusCode::CREATED);
        let project: Value = serde_json::from_slice(
            &project_response
                .into_body()
                .collect()
                .await
                .unwrap()
                .to_bytes(),
        )
        .unwrap();
        assert!(project.get("projectId").and_then(Value::as_str).is_some());

        let session_response = router
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!(
                        "/projects/{}/sessions",
                        project["projectId"].as_str().unwrap()
                    ))
                    .header(header::AUTHORIZATION, "Bearer test-token")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(json!({"title":"First"}).to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(session_response.status(), StatusCode::CREATED);
        let session: Value = serde_json::from_slice(
            &session_response
                .into_body()
                .collect()
                .await
                .unwrap()
                .to_bytes(),
        )
        .unwrap();
        assert_eq!(session["projectId"], project["projectId"]);
        assert!(session.get("sessionId").and_then(Value::as_str).is_some());
    }

    #[test]
    fn sdk_dispatches_in_process_requests() {
        let directory = tempfile::tempdir().unwrap();
        let sdk = RuntimeSdk::from_state_for_test(test_state(directory.path()));
        let response = sdk.request_json("GET", "/health", &json!({}));
        assert_eq!(response.status, StatusCode::OK.as_u16());
        assert_eq!(response.body["ok"], true);
    }

    unsafe extern "C" fn collect_event(event_json: *const c_char, user_data: *mut c_void) {
        let sender = &*(user_data as *const std::sync::mpsc::Sender<String>);
        let value = CStr::from_ptr(event_json).to_string_lossy().to_string();
        let _ = sender.send(value);
    }

    #[test]
    fn sdk_replays_subscribed_session_events() {
        let directory = tempfile::tempdir().unwrap();
        let state = test_state(directory.path());
        let root = directory.path().to_str().unwrap();
        let project = state.store.project(root, "Test").unwrap();
        let session = state
            .store
            .create_session(
                &project.project_id,
                Some("First"),
                Some("deepseek-v4-flash"),
            )
            .unwrap();
        let expected = state
            .store
            .append_content(
                &session.session_id,
                "turn.state",
                &json!({"turn_id":"turn-1","state":"preparing"}),
            )
            .unwrap();
        let sdk = RuntimeSdk::from_state_for_test(state);
        let (sender, receiver) = std::sync::mpsc::channel::<String>();
        let subscription = sdk
            .subscribe_session_events(
                session.session_id.clone(),
                0,
                collect_event,
                &sender as *const _ as *mut c_void,
            )
            .unwrap();
        let value = receiver
            .recv_timeout(std::time::Duration::from_secs(2))
            .unwrap();
        let event: SessionEvent = serde_json::from_str(&value).unwrap();
        assert_eq!(event.content_sequence, expected.content_sequence);
        subscription.close();
    }
}
