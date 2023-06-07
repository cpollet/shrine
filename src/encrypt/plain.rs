use crate::encrypt::EncDec;
use crate::Error;

#[derive(Default)]
pub struct Plain {}

impl Plain {
    pub fn new() -> Self {
        Self::default()
    }
}

impl EncDec for Plain {
    fn encrypt(&self, cleartext: &[u8]) -> Result<Vec<u8>, Error> {
        Ok(cleartext.to_vec())
    }

    fn decrypt(&self, ciphertext: &[u8]) -> Result<Vec<u8>, Error> {
        Ok(ciphertext.to_vec())
    }
}
