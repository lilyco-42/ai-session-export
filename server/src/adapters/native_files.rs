use std::path::Path;

use serde_json::Value;

use super::{extract_text, file_paths, home_dir, parse_line, read_lines, title_from_messages};
use crate::dto::{SessionSummary, ToolId};
use crate::error::{AppError, AppResult};
use crate::model::{ChatMessage, SessionRecord};

pub fn load(tool: ToolId) -> AppResult<Vec<SessionRecord>> {
    let home = home_dir();
    let (root, extension) = match tool {
        ToolId::Claude => (home.join(".claude/projects"), "jsonl"),
        ToolId::Pi => (home.join(".pi/agent/sessions"), "jsonl"),
        ToolId::Gemini => (home.join(".gemini/tmp"), "json"),
        _ => return Ok(Vec::new()),
    };
    let mut paths = Vec::new();
    file_paths(&root, extension, &mut paths)?;
    paths.sort();
    let mut records = std::collections::BTreeMap::new();
    for path in paths {
        if let Some(record) = load_file(tool, &path)? {
            records.insert(record.summary.id.clone(), record);
        }
    }
    let mut records: Vec<_> = records.into_values().collect();
    records.sort_by(|left, right| right.summary.updated_at.cmp(&left.summary.updated_at));
    Ok(records)
}

fn rows(tool: ToolId, path: &Path) -> AppResult<Vec<Value>> {
    if tool != ToolId::Gemini {
        return Ok(read_lines(path)?
            .iter()
            .filter_map(|line| parse_line(line))
            .collect());
    }
    if path
        .parent()
        .and_then(Path::file_name)
        .and_then(|name| name.to_str())
        != Some("chats")
    {
        return Ok(Vec::new());
    }
    let text = std::fs::read_to_string(path)
        .map_err(|error| AppError::source(format!("{}: {error}", path.display())))?;
    let Some(mut record) = parse_line(&text) else {
        return Ok(Vec::new());
    };
    let mut messages = record["messages"]
        .take()
        .as_array()
        .cloned()
        .unwrap_or_default();
    messages.insert(0, record);
    Ok(messages)
}

fn load_file(tool: ToolId, path: &Path) -> AppResult<Option<SessionRecord>> {
    let rows = rows(tool, path)?;
    let Some(id) = rows.iter().find_map(|row| match tool {
        ToolId::Pi if row["type"] == "session" => row["id"].as_str(),
        ToolId::Claude | ToolId::Gemini => row["sessionId"].as_str(),
        _ => None,
    }) else {
        return Ok(None);
    };
    let messages: Vec<_> = rows
        .iter()
        .filter_map(|row| {
            let payload = if tool == ToolId::Gemini {
                row
            } else {
                &row["message"]
            };
            let role = match payload["role"].as_str().or_else(|| row["type"].as_str()) {
                Some("user") => "user",
                Some("assistant" | "gemini") => "assistant",
                _ => return None,
            };
            let text = extract_text(&payload["content"]);
            (!text.trim().is_empty()).then(|| ChatMessage {
                role: role.into(),
                kind: "text".into(),
                text,
            })
        })
        .collect();
    if messages.is_empty() {
        return Ok(None);
    }
    let updated_at = rows
        .iter()
        .filter_map(|row| {
            row["lastUpdated"]
                .as_str()
                .or_else(|| row["timestamp"].as_str())
        })
        .max()
        .unwrap_or_default()
        .into();
    Ok(Some(SessionRecord {
        summary: SessionSummary {
            id: format!("{}:{id}", tool.as_str()),
            source: tool.as_str().into(),
            title: title_from_messages(&messages),
            updated_at,
        },
        messages,
        raw_ref: path.display().to_string(),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn imports_gemini_and_pi_conversations() {
        let temp = tempfile::tempdir().unwrap();
        let chats = temp.path().join("chats");
        std::fs::create_dir(&chats).unwrap();
        let gemini = chats.join("session.json");
        std::fs::write(&gemini, r#"{"sessionId":"g1","lastUpdated":"2026-11-24T12:02:00Z","messages":[{"type":"user","content":[{"text":"hi"}]},{"type":"gemini","content":"hello"}]}"#).unwrap();
        let pi = temp.path().join("session.jsonl");
        std::fs::write(&pi, concat!(
            r#"{"type":"session","id":"p1","timestamp":"2026-11-24T12:00:00Z"}"#, "\n",
            r#"{"type":"message","message":{"role":"user","content":[{"type":"text","text":"hi"}]}}"#, "\n",
            r#"{"type":"message","message":{"role":"assistant","content":[{"type":"text","text":"hello"}]}}"#
        )).unwrap();
        for (tool, path) in [(ToolId::Gemini, gemini), (ToolId::Pi, pi)] {
            let record = load_file(tool, &path).unwrap().unwrap();
            assert_eq!(record.messages.len(), 2);
            assert_eq!(record.messages[0].text, "hi");
            assert_eq!(record.messages[1].role, "assistant");
            assert_eq!(record.messages[1].text, "hello");
        }
    }
}
