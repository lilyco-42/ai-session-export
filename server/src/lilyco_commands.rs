//! Lilyco command surface for the local read-only AI session exporter.
//!
//! The Axum server remains the data/API backend for the Vite UI. These App
//! structs expose the same service to Lilyco CLI and MCP without duplicating
//! adapters or opening source stores for write.

use lilyco::prelude::*;

use crate::dto::{ExportFormat, ExportRequest, SourceQuery};
use crate::service::AppState;

fn state() -> Result<AppState, AppError> {
    AppState::try_new().map_err(|error| AppError::Runtime(error.to_string()))
}

fn finish(ctx: &Context, result: serde_json::Value) -> Result<serde_json::Value, AppError> {
    ctx.done(result.clone(), 0);
    Ok(result)
}

#[derive(App)]
#[app(
    name = "ai-session-sources",
    about = "List local AI conversation sources and session counts (read-only)",
    run = "run_sources"
)]
pub struct Sources {
    /// Unused placeholder; Lilyco derive requires named fields.
    #[arg(about = "Unused")]
    #[allow(dead_code)]
    unused: bool,
}

fn run_sources(app: &Sources, ctx: &Context) -> Result<serde_json::Value, AppError> {
    let _ = app.unused;
    let items = serde_json::to_value(
        state()?
            .sources()
            .map_err(|e| AppError::Runtime(e.to_string()))?,
    )?;
    finish(ctx, serde_json::json!({ "sources": items }))
}

#[derive(App)]
#[app(
    name = "ai-session-list",
    about = "List AI sessions by source id, or all (read-only)",
    run = "run_list"
)]
pub struct List {
    #[arg(about = "Source filter", default = "all")]
    source: String,
}

fn run_list(app: &List, ctx: &Context) -> Result<serde_json::Value, AppError> {
    let query = SourceQuery {
        source: Some(app.source.clone()),
    };
    let items = serde_json::to_value(
        state()?
            .sessions(&query)
            .map_err(|e| AppError::Runtime(e.to_string()))?,
    )?;
    finish(ctx, serde_json::json!({ "sessions": items }))
}

#[derive(App)]
#[app(
    name = "ai-session-export",
    about = "Export selected local AI sessions to markdown, jsonl, raw, or html",
    run = "run_export"
)]
pub struct Export {
    #[arg(about = "Comma-separated session ids")]
    ids: String,

    #[arg(about = "Export format", default = "markdown")]
    format: String,
}

fn run_export(app: &Export, ctx: &Context) -> Result<serde_json::Value, AppError> {
    let ids: Vec<String> = app
        .ids
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_owned)
        .collect();
    if ids.is_empty() {
        return Err(AppError::InvalidArg("ids must not be empty".into()));
    }
    let format = match app.format.as_str() {
        "markdown" => ExportFormat::Markdown,
        "jsonl" => ExportFormat::Jsonl,
        "raw" => ExportFormat::Raw,
        "html" => ExportFormat::Html,
        other => return Err(AppError::InvalidArg(format!("unknown format: {other}"))),
    };
    let response = state()?
        .export(&ExportRequest { ids, format })
        .map_err(|error| AppError::Runtime(error.to_string()))?;
    finish(ctx, serde_json::to_value(response)?)
}

#[derive(App)]
#[app(name = "ai-session-get", about = "Read a page of messages in a local session", run = "run_get")]
pub struct Get {
    #[arg(about = "Session id")]
    id: String,
    #[arg(about = "Message offset", default = 0)]
    offset: usize,
    #[arg(about = "Page size (max 200)", default = 100)]
    limit: usize,
}

fn run_get(app: &Get, ctx: &Context) -> Result<serde_json::Value, AppError> {
    let result = state()?.session_detail(&app.id, app.offset, app.limit)
        .map_err(|e| AppError::Runtime(e.to_string()))?;
    finish(ctx, serde_json::to_value(result)?)
}

#[derive(App)]
#[app(name = "ai-session-search", about = "Search local session titles and messages", run = "run_search")]
pub struct Search {
    #[arg(about = "Search text")]
    query: String,
    #[arg(about = "Source filter", default = "all")]
    source: String,
    #[arg(about = "Maximum results", default = 50)]
    limit: usize,
}

fn run_search(app: &Search, ctx: &Context) -> Result<serde_json::Value, AppError> {
    let query = SourceQuery { source: Some(app.source.clone()) };
    let results = state()?.search(&app.query, &query, app.limit)
        .map_err(|e| AppError::Runtime(e.to_string()))?;
    finish(ctx, serde_json::json!({"sessions": results}))
}

#[derive(App)]
#[app(name = "ai-task-list", about = "List last observed task events; not live process status", run = "run_tasks")]
pub struct Tasks {
    #[arg(about = "Source filter", default = "all")]
    source: String,
    #[arg(about = "Maximum results", default = 50)]
    limit: usize,
}

fn run_tasks(app: &Tasks, ctx: &Context) -> Result<serde_json::Value, AppError> {
    let query = SourceQuery { source: Some(app.source.clone()) };
    let results = state()?.tasks(&query, app.limit)
        .map_err(|e| AppError::Runtime(e.to_string()))?;
    finish(ctx, serde_json::json!({"tasks": results}))
}

#[derive(App)]
#[app(name = "ai-task-status", about = "Read last observed task event for a session", run = "run_task")]
pub struct TaskStatus {
    #[arg(about = "Session id")]
    id: String,
}

fn run_task(app: &TaskStatus, ctx: &Context) -> Result<serde_json::Value, AppError> {
    let result = state()?.task_status(&app.id).map_err(|e| AppError::Runtime(e.to_string()))?;
    finish(ctx, serde_json::to_value(result)?)
}

#[derive(App)]
#[app(name = "ai-session-resume", about = "Return a native resume command; never execute it", run = "run_resume")]
pub struct Resume {
    #[arg(about = "Session id")]
    id: String,
}

fn run_resume(app: &Resume, ctx: &Context) -> Result<serde_json::Value, AppError> {
    let result = state()?.resume_plan(&app.id).map_err(|e| AppError::Runtime(e.to_string()))?;
    finish(ctx, serde_json::to_value(result)?)
}

pub fn build_registry() -> lilyco::__core::registry::Registry {
    let mut reg = lilyco::__core::registry::Registry::new();
    reg.register(lilyco::__core::registry::RegisteredCommand::from_app::<
        Sources,
    >())
    .expect("register ai-session-sources");
    reg.register(lilyco::__core::registry::RegisteredCommand::from_app::<List>())
        .expect("register ai-session-list");
    reg.register(lilyco::__core::registry::RegisteredCommand::from_app::<
        Export,
    >())
    .expect("register ai-session-export");
    reg.register(lilyco::__core::registry::RegisteredCommand::from_app::<Get>()).expect("register get");
    reg.register(lilyco::__core::registry::RegisteredCommand::from_app::<Search>()).expect("register search");
    reg.register(lilyco::__core::registry::RegisteredCommand::from_app::<Tasks>()).expect("register tasks");
    reg.register(lilyco::__core::registry::RegisteredCommand::from_app::<TaskStatus>()).expect("register task status");
    reg.register(lilyco::__core::registry::RegisteredCommand::from_app::<Resume>()).expect("register resume");
    reg
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_has_eight_commands() {
        let reg = build_registry();
        let names: Vec<String> = reg.iter().map(|c| c.name.clone()).collect();
        assert_eq!(
            names,
            vec!["ai-session-export", "ai-session-get", "ai-session-list", "ai-session-resume", "ai-session-search", "ai-session-sources", "ai-task-list", "ai-task-status"]
        );
    }

    #[test]
    fn parse_ids_ignores_empty_entries() {
        let ids: Vec<String> = " a, ,b "
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_owned)
            .collect();
        assert_eq!(ids, vec!["a", "b"]);
    }
}
