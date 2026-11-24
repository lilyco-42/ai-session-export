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
    let app = Router::new()
        .route("/api/sources", get(sources))
        .route("/api/sessions", get(sessions))
        .route("/api/export", post(export))
        .route("/exports/{name}", get(download))
        .fallback_service(ServeDir::new(web_dir))
        .with_state(state);
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

    #[test]
    fn lilyco_mode_detects_cli_and_mcp_requests() {
        assert!(wants_lilyco(&["--mcp".into()]));
        assert!(wants_lilyco(&["ai-session-sources".into()]));
        assert!(!wants_lilyco(&[]));
    }
}
