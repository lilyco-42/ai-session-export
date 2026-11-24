use crate::dto::{SessionSummary, SourceSummary, ToolId};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChatMessage {
    pub role: String,
    pub kind: String,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionRecord {
    pub summary: SessionSummary,
    pub messages: Vec<ChatMessage>,
    pub raw_ref: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceState {
    pub summary: SourceSummary,
    pub sessions: Vec<SessionRecord>,
}

impl SourceState {
    pub fn new(id: ToolId, sessions: Vec<SessionRecord>) -> Self {
        Self::named(id.as_str(), id.label(), sessions)
    }

    pub fn named(id: &str, label: &str, sessions: Vec<SessionRecord>) -> Self {
        let status = if sessions.is_empty() {
            crate::dto::SourceStatus::Missing
        } else {
            crate::dto::SourceStatus::Ready
        };
        Self {
            summary: SourceSummary {
                id: id.into(),
                label: label.into(),
                status,
                session_count: sessions.len(),
            },
            sessions,
        }
    }
}
