use std::sync::Arc;

use ai_session_export_server::dto::{ExportRequest, SourceQuery};
use ai_session_export_server::error::AppError;
use ai_session_export_server::lilyco_commands;
use ai_session_export_server::service::AppState;
use axum::extract::{Path, Query, State};
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use tower_http::services::ServeDir;

fn wants_lilyco(args: &[String]) -> bool {
    args.iter().any(|arg| {
        arg == "--mcp"
            || arg == "--schema"
            || arg == "--help"
            || arg == "-h"
            || arg.starts_with("ai-session-")
            || arg.starts_with("ai-task-")
    })
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if wants_lilyco(&args) {
        lilyco::run_registry("ai-session-export", lilyco_commands::build_registry());
        return;
    }
    let state = match AppState::try_new() {
        Ok(state) => Arc::new(state),
        Err(error) => {
            eprintln!("startup failed: {error}");
            std::process::exit(1);
        }
    };
    let web_dir = std::env::var_os("AI_SESSION_WEB_DIR")
        .map(std::path::PathBuf::from)
        .or_else(|| {
            let exe = std::env::current_exe().ok()?;
            let exe = exe.canonicalize().unwrap_or(exe);
            let bundled = exe.parent()?.join("web").join("dist");
            bundled.join("index.html").is_file().then_some(bundled)
        })
        .unwrap_or_else(|| "web/dist".into());
    let app = router(state, web_dir);
    let listener = match tokio::net::TcpListener::bind("127.0.0.1:8788").await {
        Ok(listener) => listener,
        Err(error) => {
            eprintln!("bind failed: {error}");
            std::process::exit(1);
        }
    };
    println!("ai-session-export-server listening on http://127.0.0.1:8788");
    if let Err(error) = axum::serve(listener, app).await {
        eprintln!("server failed: {error}");
        std::process::exit(1);
    }
}

fn router(state: Arc<AppState>, web_dir: std::path::PathBuf) -> Router {
    Router::new()
        .route("/api/sources", get(sources))
        .route("/api/sessions", get(sessions))
        .route("/api/session", get(session_detail))
        .route("/api/search", get(search))
        .route("/api/tasks", get(tasks))
        .route("/api/task", get(task_status))
        .route("/api/resume", get(resume_plan))
        .route("/api/export", post(export))
        .route("/exports/{name}", get(download))
        .fallback_service(ServeDir::new(web_dir))
        .with_state(state)
}

#[derive(serde::Deserialize)]
struct DetailQuery {
    id: String,
    #[serde(default)]
    offset: usize,
    #[serde(default = "default_page")]
    limit: usize,
}
fn default_page() -> usize { 100 }
fn default_limit() -> usize { 50 }

#[derive(serde::Deserialize)]
struct SearchQuery {
    query: String,
    source: Option<String>,
    #[serde(default = "default_limit")]
    limit: usize,
}

#[derive(serde::Deserialize)]
struct TaskQuery {
    source: Option<String>,
    #[serde(default = "default_limit")]
    limit: usize,
}

#[derive(serde::Deserialize)]
struct IdQuery { id: String }

async fn session_detail(State(state): State<Arc<AppState>>, Query(q): Query<DetailQuery>)
    -> Result<Json<ai_session_export_server::inspection::SessionDetail>, AppError> {
    Ok(Json(state.session_detail(&q.id, q.offset, q.limit)?))
}

async fn search(State(state): State<Arc<AppState>>, Query(q): Query<SearchQuery>)
    -> Result<Json<Vec<ai_session_export_server::dto::SessionSummary>>, AppError> {
    Ok(Json(state.search(&q.query, &SourceQuery { source: q.source }, q.limit)?))
}

async fn tasks(State(state): State<Arc<AppState>>, Query(q): Query<TaskQuery>)
    -> Result<Json<Vec<ai_session_export_server::inspection::TaskObservation>>, AppError> {
    Ok(Json(state.tasks(&SourceQuery { source: q.source }, q.limit)?))
}

async fn task_status(State(state): State<Arc<AppState>>, Query(q): Query<IdQuery>)
    -> Result<Json<ai_session_export_server::inspection::TaskObservation>, AppError> {
    Ok(Json(state.task_status(&q.id)?))
}

async fn resume_plan(State(state): State<Arc<AppState>>, Query(q): Query<IdQuery>)
    -> Result<Json<ai_session_export_server::inspection::ResumePlan>, AppError> {
    Ok(Json(state.resume_plan(&q.id)?))
}

async fn sources(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<ai_session_export_server::dto::SourceSummary>>, AppError> {
    Ok(Json(state.sources()?))
}

async fn sessions(
    State(state): State<Arc<AppState>>,
    Query(query): Query<SourceQuery>,
) -> Result<Json<Vec<ai_session_export_server::dto::SessionSummary>>, AppError> {
    Ok(Json(state.sessions(&query)?))
}

async fn export(
    State(state): State<Arc<AppState>>,
    Json(request): Json<ExportRequest>,
) -> Result<Json<ai_session_export_server::dto::ExportResponse>, AppError> {
    Ok(Json(state.export(&request)?))
}

async fn download(State(state): State<Arc<AppState>>, Path(name): Path<String>) -> Response {
    if name.contains('/') || name.contains('\\') || name.contains("..") {
        return (StatusCode::BAD_REQUEST, "invalid file name").into_response();
    }
    let path = state.export_dir.join(&name);
    match std::fs::read(&path) {
        Ok(bytes) => {
            let content_type = match path.extension().and_then(|value| value.to_str()) {
                Some("md") => "text/markdown; charset=utf-8",
                Some("jsonl") => "application/x-ndjson; charset=utf-8",
                Some("html") => "text/html; charset=utf-8",
                _ => "text/plain; charset=utf-8",
            };
            (
                StatusCode::OK,
                [
                    (header::CONTENT_TYPE, content_type),
                    (header::CONTENT_DISPOSITION, "attachment"),
                ],
                bytes,
            )
                .into_response()
        }
        Err(_) => (StatusCode::NOT_FOUND, "export not found").into_response(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn inspection_endpoints_return_fixture_data() {
        use ai_session_export_server::{dto::SessionSummary, model::{ChatMessage, SessionRecord, SourceState}};
        use axum::{body::{Body, to_bytes}, http::Request};
        use tower::ServiceExt;
        let temp = tempfile::tempdir().unwrap();
        let record = SessionRecord { summary: SessionSummary { id: "chatgpt:fixture".into(), source: "chatgpt".into(), title: "Example".into(), updated_at: "2026-01-01T00:00:00Z".into() }, messages: vec![ChatMessage { role: "user".into(), kind: "text".into(), text: "test needle".into() }], raw_ref: "fixture".into() };
        let state = AppState::with_sources(vec![SourceState::named("chatgpt", "ChatGPT", vec![record])],temp.path().join("out")).unwrap();
        let app = router(Arc::new(state), temp.path().to_path_buf());
        for (uri, key) in [("/api/session?id=chatgpt%3Afixture", "messages"), ("/api/task?id=chatgpt%3Afixture", "status"), ("/api/resume?id=chatgpt%3Afixture", "supported")] {
            let response=app.clone().oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap()).await.unwrap();
            assert_eq!(response.status(), StatusCode::OK);
            let value: serde_json::Value=serde_json::from_slice(&to_bytes(response.into_body(), 65536).await.unwrap()).unwrap();
            assert!(value.get(key).is_some());
        }
        let response=app.clone().oneshot(Request::builder().uri("/api/search?query=needle").body(Body::empty()).unwrap()).await.unwrap();
        assert_eq!(response.status(),StatusCode::OK);
        let value: serde_json::Value=serde_json::from_slice(&to_bytes(response.into_body(),65536).await.unwrap()).unwrap();
        assert_eq!(value.as_array().unwrap().len(),1);
        let response=app.oneshot(Request::builder().uri("/api/session?id=missing").body(Body::empty()).unwrap()).await.unwrap();
        assert_eq!(response.status(),StatusCode::NOT_FOUND);
    }

    #[test]
    fn lilyco_mode_detects_cli_and_mcp_requests() {
        assert!(wants_lilyco(&["--mcp".into()]));
        assert!(wants_lilyco(&["ai-session-sources".into()]));
        assert!(wants_lilyco(&["ai-task-list".into()]));
        assert!(!wants_lilyco(&[]));
    }
}
