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
    /// No encryption, return the input
    fn encrypt(&self, cleartext: &[u8]) -> Result<Vec<u8>, Error> {
        Ok(cleartext.to_vec())
    }

    /// No decryption, return the input
    fn decrypt(&self, ciphertext: &[u8]) -> Result<Vec<u8>, Error> {
        eprintln!("WARNING: the shrine is not encrypted!");
        Ok(ciphertext.to_vec())
    }
}

#[cfg(test)]
mod tests {
    use crate::encrypt::plain::Plain;
    use crate::encrypt::EncDec;

    #[test]
    fn encrypt() {
        let plain = Plain::new();
        let cipher = plain.encrypt("clear".as_bytes()).unwrap();
        assert_eq!(cipher, "clear".as_bytes());
    }

    #[test]
    fn decrypt() {
        let plain = Plain::new();
        let clear = plain.decrypt("cipher".as_bytes()).unwrap();
        assert_eq!(clear, "cipher".as_bytes());
    }
}
