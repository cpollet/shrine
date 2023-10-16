use crate::values::bytes::SecretBytes;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

#[derive(Debug, Serialize, Deserialize)]
pub struct Secret {
    value: SecretBytes,
    mode: Mode,
    created_by: String,
    created_at: DateTime<Utc>,
    updated_by: Option<String>,
    updated_at: Option<DateTime<Utc>>,
}

impl Secret {
    pub fn new(value: SecretBytes, mode: Mode) -> Self {
        Self {
            value,
            mode,
            created_by: format!("{}@{}", whoami::username(), whoami::hostname()),
            created_at: Utc::now(),
            updated_by: None,
            updated_at: None,
        }
    }

    pub fn mode(&self) -> Mode {
        self.mode
    }

    pub fn value(&self) -> &SecretBytes {
        &self.value
    }

    pub fn created_by(&self) -> &str {
        &self.created_by
    }

    pub fn created_at(&self) -> &DateTime<Utc> {
        &self.created_at
    }

    pub fn updated_by(&self) -> Option<&str> {
        match &self.updated_by {
            None => None,
            Some(s) => Some(s.as_ref()),
        }
    }

    pub fn updated_at(&self) -> Option<&DateTime<Utc>> {
        self.updated_at.as_ref()
    }

    pub fn update_with(&mut self, data: SecretBytes, mode: Mode) -> &mut Self {
        self.value = data;
        self.mode = mode;
        self.updated_by = Some(format!("{}@{}", whoami::username(), whoami::hostname()));
        self.updated_at = Some(Utc::now());
        self
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub enum Mode {
    Binary,
    Text,
}

impl Display for Mode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Mode::Binary => write!(f, "bin"),
            Mode::Text => write!(f, "txt"),
        }
    }
}
