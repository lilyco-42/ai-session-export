//! Read the active branch of a user-provided ChatGPT data export.
//! Never read browser profiles, authentication data, or remote APIs.
use std::{collections::{BTreeMap, HashSet}, path::Path};
use serde_json::Value;
use crate::{dto::SessionSummary, error::{AppError, AppResult}, model::{ChatMessage, SessionRecord}};
use super::{home_dir, title_from_messages};

pub fn load() -> AppResult<Vec<SessionRecord>> {
    let path = std::env::var_os("AI_SESSION_CHATGPT_EXPORT").map(std::path::PathBuf::from)
        .unwrap_or_else(|| home_dir().join(".local/share/ai-session-export/imports/conversations.json"));
    if !path.is_file() { return Ok(Vec::new()); }
    load_file(&path)
}

fn load_file(path: &Path) -> AppResult<Vec<SessionRecord>> {
    let file = std::fs::File::open(path).map_err(|e| AppError::source(e.to_string()))?;
    let rows: Vec<Value> = serde_json::from_reader(std::io::BufReader::new(file))
        .map_err(|e| AppError::source(format!("invalid ChatGPT export: {e}")))?;
    let mut sessions = BTreeMap::new();
    for row in rows {
        let Some(id) = row["conversation_id"].as_str().or_else(|| row["id"].as_str()) else { continue };
        if id.is_empty() { continue; }
        let Some(mapping) = row["mapping"].as_object() else { continue };
        let Some(mut node) = row["current_node"].as_str() else { continue };
        let mut seen = HashSet::new();
        let mut messages = Vec::new();
        loop {
            if !seen.insert(node) { return Err(AppError::source("cycle in ChatGPT export")); }
            let Some(entry) = mapping.get(node) else { break };
            let msg = &entry["message"];
            if msg["metadata"]["is_visually_hidden_from_conversation"].as_bool() != Some(true) {
                if let Some(role @ ("user" | "assistant")) = msg["author"]["role"].as_str() {
                    let content = &msg["content"];
                    if matches!(content["content_type"].as_str(), Some("text" | "multimodal_text")) {
                        let text = content["parts"].as_array().map(|parts| parts.iter()
                            .filter_map(Value::as_str).collect::<Vec<_>>().join("\n")).unwrap_or_default();
                        if !text.trim().is_empty() {
                            messages.push(ChatMessage { role: role.into(), kind: "text".into(), text });
                        }
                    }
                }
            }
            match entry["parent"].as_str() { Some(parent) => node = parent, None => break }
        }
        messages.reverse();
        let updated_at = row["update_time"].as_f64().or_else(|| row["create_time"].as_f64())
            .filter(|t| t.is_finite())
            .and_then(|t| chrono::DateTime::from_timestamp_millis((t * 1000.0) as i64))
            .map(|t| t.to_rfc3339_opts(chrono::SecondsFormat::Millis, true)).unwrap_or_default();
        let id = format!("chatgpt:{id}");
        sessions.insert(id.clone(), SessionRecord {
            summary: SessionSummary { id, source: "chatgpt".into(),
                title: row["title"].as_str().filter(|s| !s.trim().is_empty())
                    .map(str::to_owned).unwrap_or_else(|| title_from_messages(&messages)), updated_at },
            messages, raw_ref: path.display().to_string(),
        });
    }
    let mut records: Vec<_> = sessions.into_values().collect();
    records.sort_by(|a,b| b.summary.updated_at.cmp(&a.summary.updated_at));
    Ok(records)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn imports_only_visible_active_branch() {
        let d = tempfile::tempdir().unwrap();
        let p = d.path().join("conversations.json");
        std::fs::write(&p, r#"[{"id":"one","title":"History","update_time":1700000000,"current_node":"a","mapping":{"u":{"parent":null,"message":{"author":{"role":"user"},"content":{"content_type":"text","parts":["Hello"]}}},"a":{"parent":"u","message":{"author":{"role":"assistant"},"content":{"content_type":"text","parts":["Current"]}}},"other":{"parent":"u","message":{"author":{"role":"assistant"},"content":{"content_type":"text","parts":["Old branch"]}}}}}]"#).unwrap();
        let records = load_file(&p).unwrap();
        assert_eq!(records[0].summary.id, "chatgpt:one");
        assert_eq!(records[0].messages.len(), 2);
        assert_eq!(records[0].messages[1].text, "Current");
    }
}
