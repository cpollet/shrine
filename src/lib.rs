use std::fmt::{Display, Formatter};
use std::io::Error as IoError;

use thiserror::Error;

pub mod bytes;
pub mod controller;
pub mod encrypt;
pub mod io;
pub mod serialize;
pub mod shrine;
pub mod shrine_file;
pub mod utils;

static SHRINE_FILENAME: &str = "shrine";

#[derive(Debug, Error)]
pub enum Error {
    FileAlreadyExists(String),
    ReadFile(IoError),
    InvalidFile(String),
    WriteFile(IoError),
    Update(String),
    KeyNotFound(String),
    InvalidPattern(regex::Error),
    InvalidPassword,
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
