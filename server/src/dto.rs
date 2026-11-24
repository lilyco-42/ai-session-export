use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ToolId {
    Codearts,
    Opencode,
    Agy,
    Codex,
    Dsh,
    Claude,
    Gemini,
    Pi,
}

impl ToolId {
    pub const ALL: [ToolId; 8] = [
        ToolId::Codearts,
        ToolId::Opencode,
        ToolId::Agy,
        ToolId::Codex,
        ToolId::Dsh,
        ToolId::Claude,
        ToolId::Gemini,
        ToolId::Pi,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            ToolId::Codearts => "codearts",
            ToolId::Opencode => "opencode",
            ToolId::Agy => "agy",
            ToolId::Codex => "codex",
            ToolId::Dsh => "dsh",
            ToolId::Claude => "claude",
            ToolId::Gemini => "gemini",
            ToolId::Pi => "pi",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            ToolId::Codearts => "CodeArts / CodeBuddy",
            ToolId::Opencode => "OpenCode",
            ToolId::Agy => "AGY / Antigravity",
            ToolId::Codex => "Codex CLI",
            ToolId::Dsh => "DeepSeek Harness",
            ToolId::Claude => "Claude Code",
            ToolId::Gemini => "Gemini CLI",
            ToolId::Pi => "Pi",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SourceStatus {
    Ready,
    Missing,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceSummary {
    pub id: String,
    pub label: String,
    pub status: SourceStatus,
    pub session_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionSummary {
    pub id: String,
    pub source: String,
    pub title: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExportRequest {
    pub ids: Vec<String>,
    pub format: ExportFormat,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ExportFormat {
    Markdown,
    Jsonl,
    Raw,
    Html,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportResponse {
    pub download_url: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ApiErrorShape {
    pub error_code: String,
    pub reason_zh: String,
    pub reason_en: String,
    pub repair_zh: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceQuery {
    pub source: Option<String>,
}
