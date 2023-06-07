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
    ///
    /// ```
    /// # use shrine::encrypt::EncDec;
    /// # use shrine::encrypt::plain::Plain;
    /// let plain = Plain::new();
    ///
    /// let clear = "clear";
    /// let cipher = plain.encrypt(clear.as_ref()).unwrap();
    ///
    /// assert_eq!(clear.as_ref(), cipher)
    /// ```
    fn encrypt(&self, cleartext: &[u8]) -> Result<Vec<u8>, Error> {
        Ok(cleartext.to_vec())
    }

    /// No decryption, return the input
    ///
    /// ```
    /// # use shrine::encrypt::EncDec;
    /// # use shrine::encrypt::plain::Plain;
    /// let plain = Plain::new();
    ///
    /// let cipher = "cipher";
    /// let clear = plain.decrypt(cipher.as_ref()).unwrap();
    ///
    /// assert_eq!(cipher.as_ref(), clear)
    /// ```
    fn decrypt(&self, ciphertext: &[u8]) -> Result<Vec<u8>, Error> {
        Ok(ciphertext.to_vec())
    }
}
