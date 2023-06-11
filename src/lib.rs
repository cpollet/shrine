use std::error::Error as StdError;

use std::path::PathBuf;

pub mod bytes;
pub mod controller;
pub mod encrypt;
pub mod git;
pub mod serialize;
pub mod shrine;
pub mod utils;

static SHRINE_FILENAME: &str = "shrine";

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Could not perform git action")]
    Git(#[from] git2::Error),

    #[error("The input file `{1}` is not a valid dotenv file: {0}")]
    InvalidDotEnv(Box<dyn StdError>, PathBuf),

    #[error("Unsupported shrine version: {0}")]
    UnsupportedVersion(u8),

    #[error("Could not read from stdin")]
    ReadStdIn(#[source] std::io::Error),

    #[error("Could not read shrine")]
    IoRead(#[source] std::io::Error),
    #[error("Could not write shrine")]
    IoWrite(#[source] std::io::Error),

    #[error("Could not read shrine")]
    Read(),

    #[error("Could not read shrine")]
    CryptoRead,
    #[error("Could not write shrine")]
    CryptoWrite,

    #[error("Could not read shrine")]
    BsonRead(#[from] bson::de::Error),
    #[error("Could not write shrine")]
    BsonWrite(#[from] bson::ser::Error),

    #[error("Could not read shrine")]
    JsonRead(#[source] serde_json::Error),
    #[error("Could not write shrine")]
    JsonWrite(#[source] serde_json::Error),

    #[error("Could not read shrine")]
    MessagePackRead(#[from] rmp_serde::decode::Error),
    #[error("Could not write shrine")]
    MessagePackWrite(#[from] rmp_serde::encode::Error),

    #[error("Shrine file `{0}` already exists")]
    FileAlreadyExists(String),

    #[error("File `{0}` not found")]
    FileNotFound(PathBuf),

    #[error("Could not import file")]
    Import(#[source] std::io::Error),

    #[error("Key `{0}` does not exist")]
    KeyNotFound(String),
    #[error("Key `{0}` is a secret in `{1}`")]
    KeyIsASecret(String, String),
    #[error("Key `{0}` is an index in `{1}`")]
    KeyIsAnIndex(String, String),
    #[error("Key is empty in `{0}`")]
    EmptyKey(String),

    #[error("Pattern is invalid")]
    InvalidPattern(regex::Error),

    #[error("The password is invalid")]
    InvalidPassword,
}
