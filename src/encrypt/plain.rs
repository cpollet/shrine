use crate::encrypt::EncDec;

#[derive(Default)]
pub struct Plain {}

impl Plain {
    pub fn new() -> Self {
        Self::default()
    }
}

impl EncDec for Plain {
    fn encrypt(&self, clear: &[u8]) -> Vec<u8> {
        clear.to_vec()
    }

    fn decrypt(&self, cipher: &[u8]) -> Vec<u8> {
        cipher.to_vec()
    }
}
