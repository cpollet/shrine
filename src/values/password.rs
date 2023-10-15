use secrecy::{CloneableSecret, ExposeSecret, Secret, SerializableSecret, Zeroize};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct ShrinePassword(Secret<Inner>);

impl ShrinePassword {
    pub fn expose_secret(&self) -> &str {
        self.0.expose_secret().0.as_str()
    }
    pub fn expose_secret_as_bytes(&self) -> &[u8] {
        self.0.expose_secret().0.as_bytes()
    }
}

impl<S> From<S> for ShrinePassword
where
    S: Into<String>,
{
    fn from(value: S) -> Self {
        Self(Secret::new(Inner(value.into())))
    }
}

#[derive(Serialize, Deserialize, Clone)]
struct Inner(String);

impl SerializableSecret for Inner {}

impl CloneableSecret for Inner {}

impl Zeroize for Inner {
    fn zeroize(&mut self) {
        self.0.zeroize()
    }
}
