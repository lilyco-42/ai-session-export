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
    if !metadata_path.is_file() {
        return Ok(Vec::new());
    }
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
    let mut indexed =
        load_indexed(&root.join("conversation_summaries.db")).unwrap_or_else(|error| {
            eprintln!("{error}");
            BTreeMap::new()
        });
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
                    source: ToolId::Agy.as_str().into(),
                    title,
                    updated_at: format_millis(Some(updated)),
                },
                messages: indexed
                    .remove(id)
                    .filter(|items| !items.is_empty())
                    .unwrap_or_else(|| messages.remove(id).unwrap_or_default()),
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

// AGY's search index stores byte offsets for individual transcript steps.
fn append_steps(steps: &mut BTreeMap<i64, ChatMessage>, body: &str, offsets: &[u8], role: &str) {
    let Ok(offsets) = serde_json::from_slice::<Vec<(i64, usize)>>(offsets) else {
        return;
    };
    for (index, &(step, start)) in offsets.iter().enumerate() {
        let end = offsets.get(index + 1).map_or(body.len(), |entry| entry.1);
        let Some(text) = body.get(start..end).map(str::trim) else {
            continue;
        };
        if text.is_empty() || (role == "user" && text.starts_with("system: ")) {
            continue;
        }
        steps.insert(
            step,
            ChatMessage {
                role: role.into(),
                kind: "text".into(),
                text: text.into(),
            },
        );
    }
}

fn load_indexed(path: &std::path::Path) -> AppResult<BTreeMap<String, Vec<ChatMessage>>> {
    if !path.is_file() {
        return Ok(BTreeMap::new());
    }
    let connection =
        rusqlite::Connection::open_with_flags(path, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)
            .map_err(|error| AppError::source(format!("AGY index: {error}")))?;
    let mut statement = connection.prepare(
        "SELECT conversation_id, user_body, agent_body, user_step_byte_offsets, agent_step_byte_offsets
         FROM conversations_fts WHERE chunk_index >= 0 ORDER BY conversation_id, chunk_index"
    ).map_err(|error| AppError::source(format!("AGY search index: {error}")))?;
    let rows = statement
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, Vec<u8>>(3)?,
                row.get::<_, Vec<u8>>(4)?,
            ))
        })
        .map_err(|error| AppError::source(format!("AGY search rows: {error}")))?;
    let mut by_session = BTreeMap::<_, BTreeMap<i64, ChatMessage>>::new();
    for (id, user, assistant, user_offsets, assistant_offsets) in rows.flatten() {
        let steps = by_session.entry(id).or_default();
        append_steps(steps, &user, &user_offsets, "user");
        append_steps(steps, &assistant, &assistant_offsets, "assistant");
    }
    Ok(by_session
        .into_iter()
        .map(|(id, steps)| (id, steps.into_values().collect()))
        .collect())
}

#[cfg(test)]
mod tests {
    #[test]
    fn indexes_unicode_bytes_in_original_step_order() {
        let mut steps = std::collections::BTreeMap::new();
        super::append_steps(&mut steps, "你好\n下一条", b"[[2,0],[4,7]]", "user");
        super::append_steps(&mut steps, "收到", b"[[3,0]]", "assistant");
        let entries: Vec<_> = steps.into_values().map(|m| (m.role, m.text)).collect();
        assert_eq!(
            entries,
            vec![
                ("user".into(), "你好".into()),
                ("assistant".into(), "收到".into()),
                ("user".into(), "下一条".into()),
            ]
        );
    }
}
