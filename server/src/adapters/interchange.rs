use std::path::Path;

use serde::Deserialize;

use super::{home_dir, read_lines, valid_source_id};
use crate::dto::{SessionSummary, SourceStatus, ToolId};
use crate::error::{AppError, AppResult};
use crate::model::{ChatMessage, SessionRecord, SourceState};

#[derive(Deserialize)]
struct Conversation {
    id: String,
    title: String,
    #[serde(rename = "updatedAt")]
    updated_at: chrono::DateTime<chrono::FixedOffset>,
    messages: Vec<Message>,
}

#[derive(Deserialize)]
struct Message {
    role: String,
    text: String,
    #[serde(default = "text_kind")]
    kind: String,
}

fn text_kind() -> String {
    "text".into()
}

pub fn load() -> AppResult<Vec<SourceState>> {
    let root = std::env::var_os("AI_SESSION_IMPORT_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| home_dir().join(".local/share/ai-session-export/imports"));
    if !root.is_dir() {
        return Ok(Vec::new());
    }
    let mut states = Vec::new();
    let entries = std::fs::read_dir(&root)
        .map_err(|error| AppError::source(format!("{}: {error}", root.display())))?;
    for entry in entries.flatten() {
        let path = entry.path();
        let Some(id) = path.file_stem().and_then(|stem| stem.to_str()) else {
            continue;
        };
        if path.extension().and_then(|ext| ext.to_str()) != Some("jsonl")
            || !valid_source_id(id)
            || id == "all"
            || ToolId::ALL.iter().any(|tool| tool.as_str() == id)
        {
            continue;
        }
        let state = match load_file(id, &path) {
            Ok(sessions) => SourceState::named(id, id, sessions),
            Err(error) => {
                eprintln!("{error}");
                let mut state = SourceState::named(id, id, Vec::new());
                state.summary.status = SourceStatus::Error;
                state
            }
        };
        states.push(state);
    }
    states.sort_by(|left, right| left.summary.id.cmp(&right.summary.id));
    Ok(states)
}

fn load_file(source: &str, path: &Path) -> AppResult<Vec<SessionRecord>> {
    let mut sessions = std::collections::BTreeMap::new();
    for (index, line) in read_lines(path)?.iter().enumerate() {
        let record: Conversation = serde_json::from_str(line).map_err(|error| {
            AppError::source(format!("{} record {}: {error}", path.display(), index + 1))
        })?;
        let prefix = format!("{source}:");
        let raw_id = record.id.strip_prefix(&prefix).unwrap_or(&record.id);
        if raw_id.trim().is_empty() {
            return Err(AppError::source(format!(
                "{} record {} has an empty id",
                path.display(),
                index + 1
            )));
        }
        let id = format!("{prefix}{raw_id}");
        sessions.insert(
            id.clone(),
            SessionRecord {
                summary: SessionSummary {
                    id,
                    source: source.into(),
                    title: record.title,
                    updated_at: record
                        .updated_at
                        .to_utc()
                        .to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
                },
                messages: record
                    .messages
                    .into_iter()
                    .map(|message| ChatMessage {
                        role: message.role,
                        text: message.text,
                        kind: message.kind,
                    })
                    .collect(),
                raw_ref: path.display().to_string(),
            },
        );
    }
    let mut records: Vec<_> = sessions.into_values().collect();
    records.sort_by(|left, right| right.summary.updated_at.cmp(&left.summary.updated_at));
    Ok(records)
}

#[cfg(test)]
mod tests {
    #[test]
    fn reads_documented_custom_session() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("my-agent.jsonl");
        std::fs::write(&path, r#"{"id":"1","title":"Example","updatedAt":"2026-11-24T20:00:00+08:00","messages":[{"role":"assistant","text":"hello"}]}"#).unwrap();
        let sessions = super::load_file("my-agent", &path).unwrap();
        assert_eq!(sessions[0].summary.id, "my-agent:1");
        assert_eq!(sessions[0].summary.updated_at, "2026-11-24T12:00:00.000Z");
        assert_eq!(sessions[0].messages[0].text, "hello");
    }
}
