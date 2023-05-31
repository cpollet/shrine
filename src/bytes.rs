use secrecy::{Secret, SerializableSecret, Zeroize};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Bytes(bytes::BytesMut);

impl Bytes {
    pub fn as_bytes(&self) -> &[u8] {
        self.as_ref()
    }
}

impl AsRef<[u8]> for Bytes {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl Zeroize for Bytes {
    fn zeroize(&mut self) {
        self.0.zeroize()
    }
}

impl SerializableSecret for Bytes {}

impl<T> From<T> for Bytes
where
    T: Into<bytes::BytesMut>,
{
    fn from(value: T) -> Self {
        Self(value.into())
    }
}

impl<'a> From<&'a Bytes> for &'a [u8] {
    fn from(value: &'a Bytes) -> Self {
        value.0.as_ref()
    }
}

pub type SecretBytes = Secret<Bytes>;
