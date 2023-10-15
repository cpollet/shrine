use crate::Error;
pub mod aes;
pub mod plain;

/// Encryption / decryption trait
pub trait EncDec {
    fn encrypt(&self, cleartext: &[u8]) -> Result<Vec<u8>, Error>;

    fn decrypt(&self, ciphertext: &[u8]) -> Result<Vec<u8>, Error>;
}
