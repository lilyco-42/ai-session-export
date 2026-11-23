use crate::adapters::{extract_text, format_millis, home_dir, title_from_messages, ToolId};
use crate::dto::SessionSummary;
use crate::error::{AppError, AppResult};
use crate::model::{ChatMessage, SessionRecord};
use rusqlite::OpenFlags;

const SESSION_LIMIT: usize = 200;
const MESSAGE_LIMIT: usize = 500;

pub fn load() -> AppResult<Vec<SessionRecord>> {
    let path = home_dir()
        .join(".local")
        .join("share")
        .join("billion-context")
        .join("sessions")
        .join("index.db");
    if !path.exists() {
        return Ok(Vec::new());
    }
    let connection = rusqlite::Connection::open_with_flags(&path, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .map_err(|error| AppError::source(format!("dsh index db: {error}")))?;
    let rows = load_session_rows(&connection)?;
    let mut sessions = Vec::with_capacity(rows.len());
    for (id, title, updated) in rows {
        let messages = load_messages(&connection, &id)?;
        let title = if title.trim().is_empty() {
            title_from_messages(&messages)
        } else {
            title
        };
        sessions.push(SessionRecord {
            summary: SessionSummary {
                id,
                source: ToolId::Dsh,
                title,
                updated_at: format_millis(Some(updated)),
            },
            messages,
            raw_ref: path.to_string_lossy().into_owned(),
        });
    }
    Ok(sessions)
}

fn load_session_rows(connection: &rusqlite::Connection) -> AppResult<Vec<(String, String, i64)>> {
    let sql = format!("SELECT s.session_id, COALESCE(s.title, ''), s.updated_at FROM sessions s ORDER BY s.updated_at DESC LIMIT {SESSION_LIMIT}");
    let mut statement = connection
        .prepare(&sql)
        .map_err(|error| AppError::source(format!("dsh session query: {error}")))?;
    let rows = statement
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, i64>(2)?,
            ))
        })
        .map_err(|error| AppError::source(format!("dsh sessions: {error}")))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| AppError::source(format!("dsh session row: {error}")))?;
    Ok(rows)
}

fn load_messages(
    connection: &rusqlite::Connection,
    session_id: &str,
) -> AppResult<Vec<ChatMessage>> {
    let sql = format!("SELECT m.role, m.kind, c.bytes FROM messages m JOIN content c ON c.hash = m.content_hash WHERE m.session_id = ?1 ORDER BY m.seq LIMIT {MESSAGE_LIMIT}");
    let mut statement = connection
        .prepare(&sql)
        .map_err(|error| AppError::source(format!("dsh message query: {error}")))?;
    let messages = statement
        .query_map([session_id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, Vec<u8>>(2)?,
            ))
        })
        .map_err(|error| AppError::source(format!("dsh messages: {error}")))?
        .filter_map(Result::ok)
        .filter_map(|(role, kind, bytes)| {
            let value = serde_json::from_slice::<serde_json::Value>(&bytes).ok()?;
            let text = extract_text(value.get("text").unwrap_or(&serde_json::Value::Null));
            if text.trim().is_empty() {
                None
            } else {
                Some(ChatMessage { role, kind, text })
            }
        })
        .collect();
    Ok(messages)
}
