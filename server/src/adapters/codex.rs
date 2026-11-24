use std::collections::BTreeMap;

use crate::adapters::{
    extract_text, home_dir, parse_line, read_text_lines, title_from_messages, ToolId,
};
use crate::dto::SessionSummary;
use crate::error::{AppError, AppResult};
use crate::model::{ChatMessage, SessionRecord};

pub fn load() -> AppResult<Vec<SessionRecord>> {
    let root = std::env::var_os("CODEX_HOME")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| home_dir().join(".codex"));
    let index_path = root.join("session_index.jsonl");
    let mut index: BTreeMap<String, (String, String)> = BTreeMap::new();
    if index_path.exists() {
        for line in read_text_lines(&index_path)? {
            let Some(value) = parse_line(&line) else {
                continue;
            };
            let Some(id) = value.get("id").and_then(serde_json::Value::as_str) else {
                continue;
            };
            let title = value
                .get("thread_name")
                .and_then(serde_json::Value::as_str)
                .unwrap_or(id)
                .to_string();
            let updated = value
                .get("updated_at")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("")
                .to_string();
            index.insert(id.to_string(), (title, updated));
        }
    }
    let mut found = Vec::new();
    collect_rollouts(&root.join("archived_sessions"), &mut found)?;
    collect_rollouts(&root.join("sessions"), &mut found)?;
    let found: BTreeMap<_, _> = found
        .into_iter()
        .map(|(id, path, time)| (id, (path, time)))
        .collect();
    let mut sessions = Vec::new();
    for (id, (path, fallback_updated)) in found {
        let messages = load_messages(&path)?;
        let (index_title, index_updated) = index
            .get(&id)
            .cloned()
            .unwrap_or_else(|| (String::new(), fallback_updated.clone()));
        let title = if index_title.trim().is_empty() {
            title_from_messages(&messages)
        } else {
            index_title
        };
        let updated_at = if index_updated.is_empty() {
            fallback_updated
        } else {
            index_updated
        };
        sessions.push(SessionRecord {
            summary: SessionSummary {
                id,
                source: ToolId::Codex.as_str().into(),
                title,
                updated_at,
            },
            messages,
            raw_ref: path.to_string_lossy().into_owned(),
        });
    }
    sessions.sort_by(|left, right| {
        crate::adapters::compare_truncated(&left.summary.updated_at, &right.summary.updated_at, 40)
    });
    Ok(sessions)
}

fn load_messages(path: &std::path::Path) -> AppResult<Vec<ChatMessage>> {
    let mut messages = Vec::new();
    for line in read_text_lines(path)? {
        let Some(value) = parse_line(&line) else {
            continue;
        };
        if value.get("type").and_then(serde_json::Value::as_str) != Some("response_item") {
            continue;
        }
        let Some(payload) = value.get("payload") else {
            continue;
        };
        if payload.get("type").and_then(serde_json::Value::as_str) != Some("message") {
            continue;
        }
        let role = payload
            .get("role")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("unknown");
        if !matches!(role, "user" | "assistant") {
            continue;
        }
        let text = extract_text(payload.get("content").unwrap_or(&serde_json::Value::Null));
        if !text.trim().is_empty() {
            messages.push(ChatMessage {
                role: role.into(),
                kind: "text".into(),
                text,
            });
        }
    }
    Ok(messages)
}

fn collect_rollouts(
    dir: &std::path::Path,
    output: &mut Vec<(String, std::path::PathBuf, String)>,
) -> AppResult<()> {
    if !dir.exists() {
        return Ok(());
    }
    let entries = std::fs::read_dir(dir)
        .map_err(|error| AppError::source(format!("{}: {error}", dir.display())))?;
    for entry in entries {
        let entry = entry.map_err(|error| AppError::source(format!("codex dir entry: {error}")))?;
        let path = entry.path();
        if entry.file_type().is_ok_and(|ty| ty.is_dir()) {
            collect_rollouts(&path, output)?;
        } else if path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.starts_with("rollout-") && name.ends_with(".jsonl"))
        {
            use std::io::BufRead;
            let Ok(file) = std::fs::File::open(&path) else {
                continue;
            };
            let Some(first) = std::io::BufReader::new(file)
                .lines()
                .next()
                .and_then(Result::ok)
                .and_then(|line| parse_line(&line))
            else {
                continue;
            };
            let payload = first.get("payload");
            let id = payload
                .and_then(|value| value.get("id"))
                .and_then(serde_json::Value::as_str)
                .or_else(|| {
                    payload
                        .and_then(|value| value.get("session_id"))
                        .and_then(serde_json::Value::as_str)
                })
                .unwrap_or("")
                .to_string();
            let updated = first
                .get("timestamp")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("")
                .to_string();
            if !id.is_empty() {
                output.push((id, path, updated));
            }
        }
    }
    Ok(())
}
