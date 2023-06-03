use aes_gcm_siv::aead::rand_core::RngCore;
use aes_gcm_siv::aead::{Aead, OsRng, Payload};
use aes_gcm_siv::{Aes256GcmSiv, Key, KeyInit, Nonce};

use pbkdf2::pbkdf2_hmac_array;
use secrecy::{ExposeSecret, Secret};
use sha2::Sha256;

use crate::encrypt::{EncDec, Error};

pub struct Aes<'pwd> {
    password: &'pwd Secret<String>,
    aad: Option<String>,
}

impl<'pwd> Aes<'pwd> {
    pub fn new(password: &'pwd Secret<String>, aad: Option<String>) -> Self {
        Self { password, aad }
    }
}

const KEY_SALT_LEN: usize = 128 / 8;
const NONCE_LEN: usize = 96 / 8;

impl<'pwd> EncDec for Aes<'pwd> {
    fn encrypt(&self, cleartext: &[u8]) -> Result<Vec<u8>, Error> {
        let mut salt = [0u8; KEY_SALT_LEN];
        OsRng.fill_bytes(&mut salt);

        let mut nonce = [0u8; NONCE_LEN];
        OsRng.fill_bytes(&mut nonce);

        let cipher = self.cipher(&salt);

        let ciphertext = cipher
            .encrypt(Nonce::from_slice(&nonce), self.payload(cleartext))
            .map_err(|e| Error::Encrypt(e.to_string()))?;

        let mut bytes = Vec::with_capacity(KEY_SALT_LEN + NONCE_LEN + ciphertext.len());

        bytes.extend(&salt);
        bytes.extend(&nonce);
        bytes.extend(ciphertext);

        Ok(bytes)
    }

    fn decrypt(&self, ciphertext: &[u8]) -> Result<Vec<u8>, Error> {
        if ciphertext.len() < KEY_SALT_LEN + NONCE_LEN {
            return Err(Error::Decrypt("Invalid ciphertext".to_string()));
        }

        let salt = &ciphertext[0..KEY_SALT_LEN];
        let nonce = &ciphertext[KEY_SALT_LEN..KEY_SALT_LEN + NONCE_LEN];
        let ciphertext = &ciphertext[KEY_SALT_LEN + NONCE_LEN..];

        let cipher = self.cipher(salt);
        cipher
            .decrypt(Nonce::from_slice(nonce), self.payload(ciphertext))
            .map_err(|e| Error::Decrypt(e.to_string()))
    }
}

// https://cheatsheetseries.owasp.org/cheatsheets/Password_Storage_Cheat_Sheet.html#pbkdf2
#[cfg(debug_assertions)]
const PBKDF2_ROUNDS: u32 = 1;
#[cfg(not(debug_assertions))]
const PBKDF2_ROUNDS: u32 = 600_000;

impl<'pwd> Aes<'pwd> {
    fn cipher(&self, salt: &[u8]) -> Aes256GcmSiv {
        let key = pbkdf2_hmac_array::<Sha256, 32>(
            self.password.expose_secret().as_bytes(),
            salt,
            PBKDF2_ROUNDS,
        );
        let key = Key::<Aes256GcmSiv>::from_slice(&key);

        Aes256GcmSiv::new(key)
    }

    fn payload<'msg, 'aad>(&'aad self, msg: &'msg [u8]) -> Payload<'msg, 'aad> {
        let aad: &[u8] = match &self.aad {
            None => &[],
            Some(aad) => aad.as_bytes(),
        };

        Payload { msg, aad }
    }
}
