use crate::adapters::{extract_text, format_millis, home_dir, parse_line, read_text_lines, ToolId};
use crate::dto::SessionSummary;
use crate::error::{AppError, AppResult};
use crate::model::{ChatMessage, SessionRecord};

pub fn load() -> AppResult<Vec<SessionRecord>> {
    let root = home_dir().join(".codebuddy").join("projects");
    if !root.exists() {
        return Ok(Vec::new());
    }
    let mut files = Vec::new();
    collect_jsonl(&root, &mut files)?;
    let mut sessions = Vec::new();
    for path in files {
        let lines = read_text_lines(&path)?;
        let mut messages = Vec::new();
        let mut id = path
            .file_stem()
            .and_then(|value| value.to_str())
            .unwrap_or("codebuddy")
            .to_string();
        let mut updated = 0_i64;
        for line in lines {
            let Some(value) = parse_line(&line) else {
                continue;
            };
            if let Some(session_id) = value.get("sessionId").and_then(serde_json::Value::as_str) {
                id = session_id.to_string();
            }
            if let Some(timestamp) = value.get("timestamp").and_then(serde_json::Value::as_i64) {
                updated = timestamp;
            }
            let role = match value.get("role").and_then(serde_json::Value::as_str) {
                Some(role) => role.to_string(),
                None => continue,
            };
            let text = extract_text(
                value
                    .get("message")
                    .unwrap_or(value.get("content").unwrap_or(&serde_json::Value::Null)),
            );
            if !text.trim().is_empty() {
                messages.push(ChatMessage {
                    role,
                    kind: "text".into(),
                    text,
                });
            }
        }
        let title = crate::adapters::title_from_messages(&messages);
        sessions.push(SessionRecord {
            summary: SessionSummary {
                id,
                source: ToolId::Codearts.as_str().into(),
                title,
                updated_at: format_millis(Some(updated)),
            },
            messages,
            raw_ref: path.to_string_lossy().into_owned(),
        });
    }
    Ok(sessions)
}

fn collect_jsonl(dir: &std::path::Path, output: &mut Vec<std::path::PathBuf>) -> AppResult<()> {
    let entries = std::fs::read_dir(dir)
        .map_err(|error| AppError::source(format!("{}: {error}", dir.display())))?;
    for entry in entries {
        let entry =
            entry.map_err(|error| AppError::source(format!("codebuddy dir entry: {error}")))?;
        let path = entry.path();
        if path.is_dir() {
            collect_jsonl(&path, output)?;
        } else if path.extension().and_then(|value| value.to_str()) == Some("jsonl") {
            output.push(path);
        }
    }
    Ok(())
}
