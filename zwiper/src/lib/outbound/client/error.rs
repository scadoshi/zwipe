use reqwest::StatusCode;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("network error: {0}")]
    Network(String),

    #[error("{0}")]
    Unauthorized(String),

    #[error("{0}")]
    Forbidden(String),

    #[error("{0}")]
    NotFound(String),

    #[error("{0}")]
    UnprocessableEntity(String),

    #[error("{0}")]
    InternalServerError(String),

    #[error("{0}")]
    Unknown(StatusCode, String),
}

impl From<reqwest::Error> for ApiError {
    fn from(value: reqwest::Error) -> Self {
        Self::Network(value.to_string())
    }
}

impl From<serde_json::Error> for ApiError {
    fn from(value: serde_json::Error) -> Self {
        Self::Network(format!("json error: {}", value))
    }
}

impl From<(StatusCode, String)> for ApiError {
    fn from(value: (StatusCode, String)) -> Self {
        let (status, message) = value;
        match status {
            StatusCode::UNAUTHORIZED => Self::Unauthorized(message),
            StatusCode::FORBIDDEN => Self::Forbidden(message),
            StatusCode::NOT_FOUND => Self::NotFound(message),
            StatusCode::UNPROCESSABLE_ENTITY => Self::UnprocessableEntity(message),
            StatusCode::INTERNAL_SERVER_ERROR => Self::InternalServerError(message),
            _ => Self::Unknown(status, message),
        }
    }
}
