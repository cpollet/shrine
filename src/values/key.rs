use crate::values::secret::{Mode, Secret};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Key {
    pub key: String,
    pub mode: Mode,
    pub created_by: String,
    pub created_at: DateTime<Utc>,
    pub updated_by: Option<String>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl From<(String, &Secret)> for Key {
    fn from((key, secret): (String, &Secret)) -> Self {
        Self {
            key,
            mode: secret.mode(),
            created_by: secret.created_by().to_string(),
            created_at: *secret.created_at(),
            updated_by: secret.updated_by().map(|s| s.to_string()),
            updated_at: secret.updated_at().copied(),
        }
    }
}
