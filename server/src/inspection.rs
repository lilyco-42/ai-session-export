//! Read-only inspection. A log event is evidence, not live process state.
use crate::{dto::SessionSummary, error::{AppError, AppResult}, model::{ChatMessage, SessionRecord}};
use serde::Serialize;
use std::io::BufRead;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionDetail {
    pub session: SessionSummary,
    pub messages: Vec<ChatMessage>,
    pub total_messages: usize,
    pub next_offset: Option<usize>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskObservation {
    pub session: SessionSummary,
    pub status: &'static str,
    pub evidence: &'static str,
    pub observed_at: Option<String>,
    pub turn_id: Option<String>,
    pub live: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResumePlan {
    pub session: SessionSummary,
    pub supported: bool,
    pub argv: Vec<String>,
    pub explanation: &'static str,
}

pub(crate) fn detail(record: &SessionRecord, offset: usize, limit: usize) -> SessionDetail {
    let total = record.messages.len();
    let start = offset.min(total);
    let end = start.saturating_add(limit.clamp(1, 200)).min(total);
    SessionDetail { session: record.summary.clone(), messages: record.messages[start..end].to_vec(),
        total_messages: total, next_offset: (end < total).then_some(end) }
}

pub(crate) fn matches(record: &SessionRecord, needle: &str) -> bool {
    record.summary.title.to_lowercase().contains(needle)
        || record.messages.iter().any(|m| m.text.to_lowercase().contains(needle))
}

pub(crate) fn task(record: &SessionRecord) -> AppResult<TaskObservation> {
    let mut result = TaskObservation { session: record.summary.clone(), status: "unknown",
        evidence: "no_supported_task_event", observed_at: None, turn_id: None, live: false };
    if record.summary.source != "codex" { return Ok(result); }
    let file = std::fs::File::open(&record.raw_ref).map_err(|e| AppError::source(e.to_string()))?;
    for line in std::io::BufReader::new(file).lines() {
        let line = line.map_err(|e| AppError::source(e.to_string()))?;
        let Ok(row) = serde_json::from_str::<serde_json::Value>(&line) else { continue };
        if row["type"] != "event_msg" { continue; }
        let payload = &row["payload"];
        let status = match payload["type"].as_str() {
            Some("task_started") => "started",
            Some("task_complete") => "completed",
            Some("turn_aborted" | "task_aborted") => "aborted",
            _ => continue,
        };
        let turn = payload["turn_id"].as_str().map(str::to_owned);
        if status != "started" && turn.is_some() && result.turn_id.is_some() && turn != result.turn_id { continue; }
        result.status = status;
        result.evidence = "codex_event_log";
        result.observed_at = row["timestamp"].as_str().map(str::to_owned);
        if turn.is_some() || status == "started" { result.turn_id = turn; }
    }
    Ok(result)
}

pub(crate) fn resume(record: &SessionRecord) -> ResumePlan {
    let id = record.summary.id.strip_prefix("claude:").unwrap_or(&record.summary.id);
    // Native session IDs must be UUIDs. Never interpret untrusted IDs as flags or shell.
    let uuid = id.len() == 36 && id.bytes().enumerate().all(|(i,b)|
        if matches!(i,8|13|18|23) { b == b'-' } else { b.is_ascii_hexdigit() });
    let argv = if uuid {
        match record.summary.source.as_str() {
            "codex" => vec!["codex".into(), "resume".into(), id.into()],
            "claude" => vec!["claude".into(), "--resume".into(), id.into()],
            _ => Vec::new(),
        }
    } else { Vec::new() };
    let supported = !argv.is_empty();
    ResumePlan { session: record.summary.clone(), supported, argv,
        explanation: if supported { "Run the argv manually in the original agent environment; no process has been launched." }
        else { "Native resume is not supported for this source or session ID; use the exported transcript as context." } }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{model::SourceState, dto::SourceQuery, service::AppState};
    #[test]
    fn reads_search_and_task_evidence_without_claiming_live_status() {
        let d = tempfile::tempdir().unwrap();
        let p = d.path().join("rollout.jsonl");
        std::fs::write(&p, concat!(
            r#"{"type":"event_msg","timestamp":"t1","payload":{"type":"task_started","turn_id":"a"}}"#, "\n",
            r#"{"type":"event_msg","timestamp":"t2","payload":{"type":"task_complete","turn_id":"a"}}"#, "\n"
        )).unwrap();
        let id = "12345678-1234-1234-1234-123456789abc";
        let record = SessionRecord { summary: SessionSummary { id: id.into(), source:"codex".into(), title:"Example".into(),updated_at:"t2".into() },
            messages: vec![ChatMessage { role:"user".into(),kind:"text".into(),text:"Find NEEDLE".into() }],raw_ref:p.display().to_string() };
        assert!(matches(&record,"needle"));
        assert_eq!(detail(&record,0,1).total_messages,1);
        assert_eq!(detail(&record,99,1).messages.len(),0);
        let observation=task(&record).unwrap();
        assert_eq!(observation.status,"completed");
        assert!(!observation.live);
        assert_eq!(resume(&record).argv,vec!["codex","resume",id]);
        let state=AppState::with_sources(vec![SourceState::named("codex","Codex",vec![record])], d.path().join("out")).unwrap();
        assert_eq!(state.search("needle",&SourceQuery{source:None},10).unwrap().len(),1);
        assert_eq!(state.task_status(id).unwrap().status,"completed");
        assert!(state.session_detail("missing",0,10).is_err());
    }
}
