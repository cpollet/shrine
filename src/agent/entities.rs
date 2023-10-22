use crate::values::secret::Mode;
use base64::Engine;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Secret {
    pub value: String,
    pub mode: Mode,
    pub created_by: String,
    pub created_at: DateTime<Utc>,
    pub updated_by: Option<String>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl From<&crate::values::secret::Secret> for Secret {
    fn from(value: &crate::values::secret::Secret) -> Self {
        Self {
            value: match value.mode() {
                Mode::Binary => base64::engine::general_purpose::STANDARD
                    .encode(value.value().expose_secret_as_bytes()),
                Mode::Text => {
                    String::from_utf8_lossy(value.value().expose_secret_as_bytes()).to_string()
                }
            },
            mode: value.mode(),
            created_by: value.created_by().to_string(),
            created_at: *value.created_at(),
            updated_by: value.updated_by().map(|v| v.to_string()),
            updated_at: value.updated_at().copied(),
        }
    }
}
