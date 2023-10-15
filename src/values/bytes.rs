use base64::Engine;
use secrecy::{CloneableSecret, DebugSecret, ExposeSecret, Secret, SerializableSecret, Zeroize};
use serde::de::{Error, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt::Formatter;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SecretBytes(Secret<Inner>);

impl SecretBytes {
    pub fn expose_secret_as_bytes(&self) -> &[u8] {
        self.0.expose_secret().as_ref()
    }
}

impl SerializableSecret for SecretBytes {}

impl<T> From<T> for SecretBytes
where
    T: Into<Vec<u8>>,
{
    fn from(value: T) -> Self {
        Self(Secret::new(Inner(value.into())))
    }
}

#[derive(Clone)]
struct Inner(Vec<u8>);

impl SerializableSecret for Inner {}

impl DebugSecret for Inner {}

impl CloneableSecret for Inner {}

impl Zeroize for Inner {
    fn zeroize(&mut self) {
        self.0.zeroize()
    }
}

impl AsRef<[u8]> for Inner {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl Serialize for Inner {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let str = base64::engine::general_purpose::STANDARD.encode(self.0.as_slice());
        serializer.serialize_str(&str)
    }
}

impl<'de> Deserialize<'de> for Inner {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(Base64Visitor {}).map(Inner)
    }
}

struct Base64Visitor {}

impl<'de> Visitor<'de> for Base64Visitor {
    type Value = Vec<u8>;

    fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
        formatter.write_str("A base64 encoded string")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: Error,
    {
        base64::engine::general_purpose::STANDARD
            .decode(v)
            .map_err(|_| Error::custom("Invalid base64 data"))
    }
}
