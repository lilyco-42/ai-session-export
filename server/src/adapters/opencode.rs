use crate::adapters::{title_from_messages, ToolId};
use crate::dto::SessionSummary;
use crate::error::{AppError, AppResult};
use crate::model::{ChatMessage, SessionRecord};
use rusqlite::OpenFlags;

pub fn load() -> AppResult<Vec<SessionRecord>> {
    let path = crate::adapters::home_dir()
        .join(".local")
        .join("share")
        .join("opencode")
        .join("opencode.db");
    if !path.exists() {
        return Ok(Vec::new());
    }
    let connection = rusqlite::Connection::open_with_flags(&path, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .map_err(|error| AppError::source(format!("opencode db: {error}")))?;
    let mut statement = connection
        .prepare("SELECT id, COALESCE(title, slug, id), COALESCE(time_updated, time_created, 0) FROM session_v2 ORDER BY time_updated DESC")
        .map_err(|error| AppError::source(format!("opencode session query: {error}")))?;
    let rows = statement
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, i64>(2)?,
            ))
        })
        .map_err(|error| AppError::source(format!("opencode sessions: {error}")))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| AppError::source(format!("opencode session row: {error}")))?;
    let mut sessions = Vec::new();
    for (id, title, updated) in rows {
        let mut message_stmt = connection
            .prepare("SELECT data FROM part WHERE session_id = ?1 ORDER BY time_created")
            .map_err(|error| AppError::source(format!("opencode part query: {error}")))?;
        let messages = message_stmt
            .query_map([&id], |row| row.get::<_, String>(0))
            .map_err(|error| AppError::source(format!("opencode parts: {error}")))?
            .filter_map(Result::ok)
            .filter_map(|value| serde_json::from_str::<serde_json::Value>(&value).ok())
            .map(|value| ChatMessage {
                role: value
                    .get("role")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or("unknown")
                    .into(),
                kind: value
                    .get("type")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or("text")
                    .into(),
                text: crate::adapters::extract_text(
                    value
                        .get("text")
                        .unwrap_or(value.get("content").unwrap_or(&serde_json::Value::Null)),
                ),
            })
            .collect::<Vec<_>>();
        let title = if title.trim().is_empty() {
            title_from_messages(&messages)
        } else {
            title
        };
        sessions.push(SessionRecord {
            summary: SessionSummary {
                id,
                source: ToolId::Opencode,
                title,
                updated_at: crate::adapters::format_millis(Some(updated)),
            },
            messages,
            raw_ref: path.to_string_lossy().into_owned(),
        });
    }
    Ok(sessions)
}
