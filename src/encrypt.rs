use std::fmt::{Display, Formatter};
use thiserror::Error;

pub mod aes;
pub mod plain;

/// Encryption / decryption trait
pub trait EncDec {
    fn encrypt(&self, cleartext: &[u8]) -> Result<Vec<u8>, Error>;

    fn decrypt(&self, ciphertext: &[u8]) -> Result<Vec<u8>, Error>;
}

#[derive(Debug, Error)]
pub enum Error {
    Encrypt(String),
    Decrypt(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
