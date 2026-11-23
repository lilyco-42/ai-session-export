use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;

use crate::dto::ApiErrorShape;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("source read failed: {0}")]
    Source(String),
    #[error("session not found: {0}")]
    NotFound(String),
    #[error("invalid request: {0}")]
    Invalid(String),
    #[error("export failed: {0}")]
    Export(String),
}

impl AppError {
    pub fn source(message: impl Into<String>) -> Self {
        Self::Source(message.into())
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Self::NotFound(message.into())
    }

    pub fn invalid(message: impl Into<String>) -> Self {
        Self::Invalid(message.into())
    }

    pub fn export(message: impl Into<String>) -> Self {
        Self::Export(message.into())
    }

    fn shape(&self) -> ApiErrorShape {
        match self {
            Self::Source(reason) => ApiErrorShape {
                error_code: "E_SOURCE_READ".into(),
                reason_zh: format!("读取会话源失败：{reason}"),
                reason_en: format!("Failed to read session source: {reason}"),
                repair_zh: "请确认本机会话目录存在并具有只读权限。".into(),
            },
            Self::NotFound(reason) => ApiErrorShape {
                error_code: "E_SESSION_NOT_FOUND".into(),
                reason_zh: format!("找不到会话：{reason}"),
                reason_en: format!("Session was not found: {reason}"),
                repair_zh: "请刷新会话列表后重新选择。".into(),
            },
            Self::Invalid(reason) => ApiErrorShape {
                error_code: "E_INVALID_REQUEST".into(),
                reason_zh: format!("请求参数不合法：{reason}"),
                reason_en: format!("Request is invalid: {reason}"),
                repair_zh: "请检查 ids/format/source 字段。".into(),
            },
            Self::Export(reason) => ApiErrorShape {
                error_code: "E_EXPORT_FAILED".into(),
                reason_zh: format!("导出失败：{reason}"),
                reason_en: format!("Export failed: {reason}"),
                repair_zh: "请确认输出目录可写后重试。".into(),
            },
        }
    }

    fn status(&self) -> StatusCode {
        match self {
            Self::NotFound(_) => StatusCode::NOT_FOUND,
            Self::Invalid(_) => StatusCode::BAD_REQUEST,
            Self::Source(_) | Self::Export(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = self.status();
        let body = self.shape();
        (status, Json(body)).into_response()
    }
}

pub type AppResult<T> = Result<T, AppError>;
