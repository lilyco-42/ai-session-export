use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard};

use crate::adapters;
use crate::dto::{
    ExportFormat, ExportRequest, ExportResponse, SessionSummary, SourceQuery, SourceSummary,
};
use crate::error::{AppError, AppResult};
use crate::model::{SessionRecord, SourceState};

pub struct AppState {
    pub sources: Mutex<Vec<SourceState>>,
    pub export_dir: PathBuf,
}

impl AppState {
    pub fn try_new() -> AppResult<Self> {
        let sources = adapters::load_all()?;
        let export_dir = adapters::home_dir()
            .join(".local")
            .join("share")
            .join("ai-session-export")
            .join("out");
        Self::with_sources(sources, export_dir)
    }

    pub fn with_sources(sources: Vec<SourceState>, export_dir: PathBuf) -> AppResult<Self> {
        std::fs::create_dir_all(&export_dir)
            .map_err(|error| AppError::export(format!("{}: {error}", export_dir.display())))?;
        Ok(Self {
            sources: Mutex::new(sources),
            export_dir,
        })
    }

    fn lock(&self) -> AppResult<MutexGuard<'_, Vec<SourceState>>> {
        self.sources
            .lock()
            .map_err(|_| AppError::source("source state lock poisoned"))
    }

    pub fn sources(&self) -> AppResult<Vec<SourceSummary>> {
        Ok(self
            .lock()?
            .iter()
            .map(|state| state.summary.clone())
            .collect())
    }

    pub fn sessions(&self, query: &SourceQuery) -> AppResult<Vec<SessionSummary>> {
        let wanted = parse_source(query.source.as_deref())?;
        Ok(self
            .lock()?
            .iter()
            .filter(|state| wanted.is_none_or(|tool| state.summary.id == tool))
            .flat_map(|state| state.sessions.iter().map(|session| session.summary.clone()))
            .collect())
    }

    pub fn export(&self, request: &ExportRequest) -> AppResult<ExportResponse> {
        if request.ids.is_empty() {
            return Err(AppError::invalid("ids must not be empty"));
        }
        let states = self.lock()?;
        let records = request
            .ids
            .iter()
            .map(|id| adapters::find_session(&states, id).ok_or_else(|| AppError::not_found(id)))
            .collect::<AppResult<Vec<_>>>()?;
        let name = export_name(&request.ids, request.format);
        let path = self.export_dir.join(&name);
        let body = render(records, request.format);
        std::fs::write(&path, body)
            .map_err(|error| AppError::export(format!("{}: {error}", path.display())))?;
        Ok(ExportResponse {
            download_url: format!("/exports/{name}"),
        })
    }
}

pub fn parse_source(value: Option<&str>) -> AppResult<Option<&str>> {
    match value.unwrap_or("all") {
        "all" => Ok(None),
        source if adapters::valid_source_id(source) => Ok(Some(source)),
        source => Err(AppError::invalid(format!("invalid source id: {source}"))),
    }
}

fn export_name(ids: &[String], format: ExportFormat) -> String {
    let stem = ids
        .first()
        .map(String::as_str)
        .unwrap_or("sessions")
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
                ch
            } else {
                '_'
            }
        })
        .take(60)
        .collect::<String>();
    let suffix = if ids.len() == 1 {
        String::new()
    } else {
        format!("-{}", ids.len())
    };
    let ext = match format {
        ExportFormat::Markdown => "md",
        ExportFormat::Jsonl => "jsonl",
        ExportFormat::Raw => "txt",
        ExportFormat::Html => "html",
    };
    format!("{stem}{suffix}.{ext}")
}

fn render(records: Vec<&SessionRecord>, format: ExportFormat) -> String {
    match format {
        ExportFormat::Markdown => render_markdown(records),
        ExportFormat::Jsonl => render_jsonl(records),
        ExportFormat::Raw => render_raw(records),
        ExportFormat::Html => render_html(records),
    }
}

fn render_markdown(records: Vec<&SessionRecord>) -> String {
    let mut out = String::new();
    for record in records {
        out.push_str(&format!(
            "# {}\n\n- source: {}\n- updated: {}\n- raw: {}\n\n",
            record.summary.title,
            record.summary.source.as_str(),
            record.summary.updated_at,
            record.raw_ref
        ));
        for message in &record.messages {
            out.push_str(&format!("## {}\n\n{}\n\n", message.role, message.text));
        }
    }
    out
}

fn render_jsonl(records: Vec<&SessionRecord>) -> String {
    let mut out = String::new();
    for record in records {
        let value = serde_json::json!({
            "id": record.summary.id,
            "source": record.summary.source.as_str(),
            "title": record.summary.title,
            "updatedAt": record.summary.updated_at,
            "messages": record.messages.iter().map(|message| serde_json::json!({
                "role": message.role,
                "kind": message.kind,
                "text": message.text,
            })).collect::<Vec<_>>(),
        });
        out.push_str(&value.to_string());
        out.push('\n');
    }
    out
}

fn render_raw(records: Vec<&SessionRecord>) -> String {
    let mut out = String::new();
    for record in records {
        out.push_str(&format!("SOURCE_REF\t{}\n", record.raw_ref));
        for message in &record.messages {
            out.push_str(&format!(
                "{}\t{}\t{}\n",
                message.role,
                message.kind,
                message.text.replace('\n', "\\n")
            ));
        }
    }
    out
}

fn render_html(records: Vec<&SessionRecord>) -> String {
    let mut out = String::from("<!doctype html><html lang=\"zh-CN\"><meta charset=\"utf-8\"><title>AI Session Export</title><body>");
    for record in records {
        out.push_str(&format!(
            "<h1>{}</h1><p>{} · {}</p>",
            escape_html(&record.summary.title),
            record.summary.source.as_str(),
            record.summary.updated_at
        ));
        for message in &record.messages {
            out.push_str(&format!(
                "<section><h3>{}</h3><pre>{}</pre></section>",
                escape_html(&message.role),
                escape_html(&message.text)
            ));
        }
    }
    out.push_str("</body></html>");
    out
}

fn escape_html(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('\"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dto::{ExportFormat, SourceStatus, ToolId};
    use crate::model::{ChatMessage, SessionRecord, SourceState};

    fn state() -> (tempfile::TempDir, AppState) {
        let dir = tempfile::tempdir().unwrap();
        let source = SourceState::new(
            ToolId::Agy,
            vec![SessionRecord {
                summary: SessionSummary {
                    id: "test-session".into(),
                    source: ToolId::Agy.as_str().into(),
                    title: "测试会话".into(),
                    updated_at: "2026-01-01T00:00:00Z".into(),
                },
                messages: vec![ChatMessage {
                    role: "user".into(),
                    kind: "text".into(),
                    text: "hello".into(),
                }],
                raw_ref: "memory://test".into(),
            }],
        );
        let state = AppState::with_sources(vec![source], dir.path().to_path_buf()).unwrap();
        (dir, state)
    }

    #[test]
    fn parses_known_source() {
        assert_eq!(parse_source(Some("agy")).unwrap(), Some("agy"));
        assert_eq!(parse_source(None).unwrap(), None);
        assert!(parse_source(Some("../unknown")).is_err());
    }

    #[test]
    fn exports_selected_session() {
        let (_dir, state) = state();
        let response = state
            .export(&ExportRequest {
                ids: vec!["test-session".into()],
                format: ExportFormat::Markdown,
            })
            .unwrap();
        assert_eq!(response.download_url, "/exports/test-session.md");
        let body = std::fs::read_to_string(state.export_dir.join("test-session.md")).unwrap();
        assert!(body.contains("测试会话"));
        assert!(body.contains("hello"));
    }

    #[test]
    fn rejects_unknown_session() {
        let (_dir, state) = state();
        assert!(state
            .export(&ExportRequest {
                ids: vec!["missing".into()],
                format: ExportFormat::Jsonl,
            })
            .is_err());
    }

    #[test]
    fn source_summary_serializes_camel_case() {
        let summary = SourceSummary {
            id: ToolId::Agy.as_str().into(),
            label: "AGY".into(),
            status: SourceStatus::Ready,
            session_count: 26,
        };
        let value = serde_json::to_value(summary).unwrap();
        assert_eq!(value["sessionCount"], 26);
    }
}
