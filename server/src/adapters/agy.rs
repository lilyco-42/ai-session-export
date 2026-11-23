use std::collections::BTreeMap;

use crate::adapters::{extract_text, format_millis, home_dir, parse_line, read_text_lines};
use crate::dto::{SessionSummary, ToolId};
use crate::error::{AppError, AppResult};
use crate::model::{ChatMessage, SessionRecord};

pub fn millis_to_rfc3339(value: i64) -> String {
    let secs = value.div_euclid(1000);
    let millis = value.rem_euclid(1000);
    let days = secs.div_euclid(86_400);
    let day_secs = secs.rem_euclid(86_400);
    let (year, month, day) = civil_from_days(days);
    format!(
        "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}.{:03}Z",
        day_secs / 3600,
        day_secs / 60 % 60,
        day_secs % 60,
        millis
    )
}

fn civil_from_days(days: i64) -> (i64, i64, i64) {
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let year = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = mp + if mp < 10 { 3 } else { -9 };
    (year + i64::from(month <= 2), month, day)
}

pub fn load() -> AppResult<Vec<SessionRecord>> {
    let root = home_dir().join(".gemini").join("antigravity-cli");
    let metadata_path = root.join("cache").join("conversation_metadata.json");
    let meta_raw = std::fs::read_to_string(&metadata_path)
        .map_err(|error| AppError::source(format!("{}: {error}", metadata_path.display())))?;
    let meta: serde_json::Value = serde_json::from_str(&meta_raw)
        .map_err(|error| AppError::source(format!("metadata JSON: {error}")))?;
    let mut messages: BTreeMap<String, Vec<ChatMessage>> = BTreeMap::new();
    let history = root.join("history.jsonl");
    if history.exists() {
        for line in read_text_lines(&history)? {
            let Some(value) = parse_line(&line) else {
                continue;
            };
            let Some(id) = value
                .get("conversationId")
                .and_then(serde_json::Value::as_str)
            else {
                continue;
            };
            let role = match value.get("type").and_then(serde_json::Value::as_str) {
                Some("user") => "user",
                Some("assistant") => "assistant",
                _ => "user",
            };
            let text = extract_text(value.get("display").unwrap_or(&serde_json::Value::Null));
            messages
                .entry(id.to_string())
                .or_default()
                .push(ChatMessage {
                    role: role.into(),
                    kind: "text".into(),
                    text,
                });
        }
    }
    let mut sessions = Vec::new();
    if let Some(conversations) = meta
        .get("conversations")
        .and_then(serde_json::Value::as_object)
    {
        for (id, conversation) in conversations {
            let summary_data = conversation.get("summary").unwrap_or(conversation);
            let title = summary_data
                .get("Title")
                .and_then(serde_json::Value::as_str)
                .filter(|title| !title.is_empty())
                .map(str::to_string)
                .unwrap_or_else(|| {
                    crate::adapters::title_from_messages(
                        messages.get(id).map(Vec::as_slice).unwrap_or(&[]),
                    )
                });
            let updated = summary_data
                .get("UpdatedAt")
                .and_then(serde_json::Value::as_str)
                .and_then(chrono_parse_millis)
                .or_else(|| {
                    conversation
                        .get("last_modified_time")
                        .and_then(serde_json::Value::as_str)
                        .and_then(chrono_parse_millis)
                })
                .unwrap_or(0);
            let db_path = root.join("conversations").join(format!("{id}.db"));
            sessions.push(SessionRecord {
                summary: SessionSummary {
                    id: id.clone(),
                    source: ToolId::Agy,
                    title,
                    updated_at: format_millis(Some(updated)),
                },
                messages: messages.get(id).cloned().unwrap_or_default(),
                raw_ref: db_path.to_string_lossy().into_owned(),
            });
        }
    }
    sessions.sort_by(|left, right| {
        crate::adapters::compare_truncated(&left.summary.updated_at, &right.summary.updated_at, 40)
    });
    Ok(sessions)
}

fn chrono_parse_millis(value: &str) -> Option<i64> {
    if let Ok(parsed) = value.parse::<i64>() {
        return Some(parsed);
    }
    chrono::DateTime::parse_from_rfc3339(value)
        .ok()
        .map(|parsed| parsed.timestamp_millis())
}
