use crate::bytes::SecretBytes;
use crate::shrine::{Mode, ShrinePassword};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub mod client;
pub mod server;

#[derive(Deserialize, Serialize, Debug)]
pub enum ErrorResponse {
    FileNotFound(String),
    Read(String),
    Write(String),
    Io(String),
    Unauthorized(Uuid),
    Forbidden(Uuid),
    KeyNotFound { file: String, key: String },
    Regex(String),
}

impl ErrorResponse {
    fn status_code(&self) -> StatusCode {
        match self {
            ErrorResponse::FileNotFound(_) => StatusCode::NOT_FOUND,
            ErrorResponse::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            ErrorResponse::Forbidden(_) => StatusCode::FORBIDDEN,
            ErrorResponse::KeyNotFound { .. } => StatusCode::NOT_FOUND,
            ErrorResponse::Read(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ErrorResponse::Write(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ErrorResponse::Io(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ErrorResponse::Regex(_) => StatusCode::BAD_REQUEST,
        }
    }
}

impl From<ErrorResponse> for Response {
    fn from(value: ErrorResponse) -> Self {
        (value.status_code(), Json(value)).into_response()
    }
}

#[derive(Serialize, Deserialize)]
pub struct SetPasswordRequest {
    pub uuid: Uuid,
    pub password: ShrinePassword,
}

#[derive(Serialize, Deserialize)]
pub struct SetSecretRequest {
    pub secret: SecretBytes,
    pub mode: Mode,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetSecretsRequest {
    pub regexp: Option<String>,
}
