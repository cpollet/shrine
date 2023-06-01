use secrecy::{CloneableSecret, DebugSecret, ExposeSecret, Secret, SerializableSecret, Zeroize};
use serde::{Deserialize, Serialize};

/// A wrapper around [`secrecy::Secret`] to represent secret bytes. The bytes themselves are
/// [`bytes::BytesMut`] instances.
///
/// ```
/// # use shrine::bytes::SecretBytes;
/// let secret_bytes = SecretBytes::from("my_secret");
/// let debug_str = format!("{:?}", secret_bytes);
/// assert!(!debug_str.contains("my_secret"));
/// ```
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SecretBytes(Secret<InnerBytes>);

impl SecretBytes {
    /// Exposes the secret bytes.
    ///
    /// ```
    /// # use shrine::bytes::SecretBytes;
    /// let secret_bytes = SecretBytes::from("my_secret");
    ///
    /// let bytes = secret_bytes.expose_secret_as_bytes();
    /// assert_eq!(bytes, "my_secret".as_bytes());
    /// ```
    pub fn expose_secret_as_bytes(&self) -> &[u8] {
        self.0.expose_secret().as_ref()
    }
}

impl SerializableSecret for SecretBytes {}

impl<T> From<T> for SecretBytes
where
    T: Into<bytes::BytesMut>,
{
    fn from(value: T) -> Self {
        Self(Secret::new(InnerBytes(value.into())))
    }
}

#[derive(Serialize, Deserialize, Clone)]
struct InnerBytes(bytes::BytesMut);

impl SerializableSecret for InnerBytes {}

impl DebugSecret for InnerBytes {}

impl CloneableSecret for InnerBytes {}

impl Zeroize for InnerBytes {
    fn zeroize(&mut self) {
        self.0.zeroize()
    }
}

impl AsRef<[u8]> for InnerBytes {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}
