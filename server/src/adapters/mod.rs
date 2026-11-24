mod agy;
mod codebuddy;
mod codex;
mod dsh;
mod interchange;
mod native_files;
mod opencode;

use std::path::{Path, PathBuf};

use crate::dto::ToolId;
use crate::error::{AppError, AppResult};
use crate::model::{ChatMessage, SessionRecord, SourceState};

pub fn home_dir() -> PathBuf {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("USERPROFILE").map(PathBuf::from))
        .unwrap_or_else(|| PathBuf::from("."))
}

pub fn text_from_content(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::String(text) => text.clone(),
        serde_json::Value::Array(items) => items
            .iter()
            .map(text_from_content)
            .filter(|text| !text.is_empty())
            .collect::<Vec<_>>()
            .join("\n"),
        serde_json::Value::Object(map) => {
            if let Some(text) = map.get("text").and_then(serde_json::Value::as_str) {
                return text.to_string();
            }
            map.get("content")
                .map(text_from_content)
                .unwrap_or_default()
        }
        _ => String::new(),
    }
}

pub fn read_lines(path: &Path) -> AppResult<Vec<String>> {
    let data = std::fs::read_to_string(path)
        .map_err(|error| AppError::source(format!("{}: {error}", path.display())))?;
    Ok(data
        .lines()
        .map(str::to_string)
        .filter(|line| !line.trim().is_empty())
        .collect())
}

pub fn parse_json_line(line: &str) -> Option<serde_json::Value> {
    serde_json::from_str(line).ok()
}

pub fn updated_at_from_millis(value: Option<i64>) -> String {
    value
        .map(agy::millis_to_rfc3339)
        .unwrap_or_else(|| "1970-01-01T00:00:00Z".into())
}

pub fn min_len(left: &str, right: &str, limit: usize) -> std::cmp::Ordering {
    right.chars().take(limit).cmp(left.chars().take(limit))
}

pub fn load_source(tool: ToolId) -> AppResult<SourceState> {
    let sessions = match tool {
        ToolId::Agy => agy::load(),
        ToolId::Opencode => opencode::load(),
        ToolId::Codex => codex::load(),
        ToolId::Codearts => codebuddy::load(),
        ToolId::Dsh => dsh::load(),
        ToolId::Claude | ToolId::Gemini | ToolId::Pi => native_files::load(tool),
    }?;
    Ok(SourceState::new(tool, sessions))
}

pub fn load_all() -> AppResult<Vec<SourceState>> {
    let mut states: Vec<_> = ToolId::ALL
        .iter()
        .copied()
        .map(|tool| {
            load_source(tool).unwrap_or_else(|error| {
                eprintln!("{error}");
                let mut state = SourceState::new(tool, Vec::new());
                state.summary.status = crate::dto::SourceStatus::Error;
                state
            })
        })
        .collect();
    match interchange::load() {
        Ok(custom) => states.extend(custom),
        Err(error) => eprintln!("{error}"),
    }
    Ok(states)
}

pub fn valid_source_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 64
        && id
            .bytes()
            .all(|b| matches!(b, b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_'))
}

fn file_paths(dir: &Path, extension: &str, output: &mut Vec<PathBuf>) -> AppResult<()> {
    if !dir.is_dir() {
        return Ok(());
    }
    let entries = std::fs::read_dir(dir)
        .map_err(|error| AppError::source(format!("{}: {error}", dir.display())))?;
    for entry in entries.flatten() {
        let Ok(kind) = entry.file_type() else {
            continue;
        };
        let path = entry.path();
        if kind.is_dir() {
            file_paths(&path, extension, output)?;
        } else if kind.is_file()
            && path.extension().and_then(|value| value.to_str()) == Some(extension)
        {
            output.push(path);
        }
    }
    Ok(())
}

pub fn find_session<'a>(sources: &'a [SourceState], id: &str) -> Option<&'a SessionRecord> {
    sources
        .iter()
        .flat_map(|source| source.sessions.iter())
        .find(|session| session.summary.id == id)
}

pub fn title_from_messages(messages: &[ChatMessage]) -> String {
    messages
        .iter()
        .find(|message| message.role == "user" && !message.text.trim().is_empty())
        .map(|message| {
            let compact = message.text.replace('\n', " ");
            compact.chars().take(60).collect::<String>()
        })
        .unwrap_or_else(|| "未命名会话".into())
}

pub use min_len as compare_truncated;
pub use parse_json_line as parse_line;
pub use read_lines as read_text_lines;
pub use text_from_content as extract_text;
pub use updated_at_from_millis as format_millis;
