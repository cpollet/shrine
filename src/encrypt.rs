pub mod plain;

/// Encryption / decryption trait
pub trait EncDec {
    fn encrypt(&self, clear: &[u8]) -> Vec<u8>;

    fn decrypt(&self, cipher: &[u8]) -> Vec<u8>;
}
