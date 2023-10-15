use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;

use crate::values::bytes::SecretBytes;
use crate::values::password::ShrinePassword;
use crate::values::secret::Mode;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub mod client;
#[cfg(unix)]
pub mod server;

#[derive(Deserialize, Serialize, Debug)]
#[cfg(unix)]
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

#[cfg(unix)]
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

#[cfg(unix)]
impl From<ErrorResponse> for Response {
    fn from(value: ErrorResponse) -> Self {
        (value.status_code(), Json(value)).into_response()
    }
}

#[cfg(unix)]
#[derive(Serialize, Deserialize)]
pub struct SetPasswordRequest {
    pub uuid: Uuid,
    pub password: ShrinePassword,
}

#[cfg(unix)]
#[derive(Serialize, Deserialize)]
pub struct SetSecretRequest {
    pub secret: SecretBytes,
    pub mode: Mode,
}

#[cfg(unix)]
#[derive(Debug, Serialize, Deserialize)]
pub struct GetSecretsRequest {
    pub regexp: Option<String>,
}
