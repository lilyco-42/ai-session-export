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
    about = "List AI sessions by source: all, codearts, opencode, agy, codex, dsh (read-only)",
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
    reg
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_has_three_commands() {
        let reg = build_registry();
        let names: Vec<String> = reg.iter().map(|c| c.name.clone()).collect();
        assert_eq!(
            names,
            vec!["ai-session-export", "ai-session-list", "ai-session-sources"]
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
