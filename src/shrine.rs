use crate::bytes::SecretBytes;
use crate::serialize::{Error, SerDe};

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Holds the secrets
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Shrine {
    /// Secrets and data private to the shrine.
    private: HashMap<String, SecretBytes>,
    /// Actual user-defined secrets.
    secrets: HashMap<String, SecretBytes>,
}

impl Shrine {
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets a secret.
    pub fn set<K, V>(&mut self, key: K, value: V)
    where
        K: Into<String>,
        V: Into<SecretBytes>,
    {
        self.secrets.insert(key.into(), value.into());
    }

    /// Gets a secret's value
    pub fn get<'k, K>(&self, key: K) -> Option<&SecretBytes>
    where
        K: Into<&'k str>,
    {
        self.secrets.get(key.into())
    }

    /// Removes a secret.
    pub fn remove<'k, K>(&mut self, key: K)
    where
        K: Into<&'k str>,
    {
        self.secrets.remove(key.into());
    }

    /// Returns the count of secrets in the shrine.
    ///
    /// ```
    /// # use shrine::shrine::Shrine;
    /// let mut model = Shrine::new();
    /// model.set("key", "value");
    /// assert_eq!(1, model.len());
    /// ```
    pub fn len(&self) -> u64 {
        self.secrets.len() as u64
    }

    /// Returns whether the shrine has secrets or not.
    pub fn is_empty(&self) -> bool {
        self.secrets.is_empty()
    }

    /// Serialises the `Secrets`, using the provided serializer.
    ///
    /// ```
    /// # use shrine::shrine::Shrine;
    /// # use shrine::serialize::json::JsonSerDe;
    /// let serializer = Box::new(JsonSerDe::new());
    ///
    /// let mut secrets = Shrine::new();
    /// secrets.set("key", "val");
    ///
    /// let bytes = secrets.as_bytes(serializer).unwrap();
    ///
    /// assert_eq!(
    ///     "{\"private\":{},\"secrets\":{\"key\":[118,97,108]}}".to_string(),
    ///     String::from_utf8(bytes).unwrap()
    /// )
    /// ```
    pub fn as_bytes(&self, serializer: Box<dyn SerDe<Self>>) -> Result<Vec<u8>, Error> {
        serializer.serialize(self)
    }

    /// Deserializes bytes into the `Secrets`, using the provided deserializer.
    ///
    /// ```
    /// # use shrine::shrine::Shrine;
    /// # use shrine::serialize::json::JsonSerDe;
    /// let serializer = Box::new(JsonSerDe::new());
    ///
    /// let bytes = "{\"private\":{},\"secrets\":{\"key\":[118,97,108]}}".as_bytes();
    /// let secrets = Shrine::from_bytes(bytes, serializer).unwrap();
    ///
    /// assert_eq!(
    ///     1,
    ///     secrets.len()
    /// )
    /// ```
    pub fn from_bytes(bytes: &[u8], serializer: Box<dyn SerDe<Self>>) -> Result<Self, Error> {
        serializer.deserialize(bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add() {
        let mut model = Shrine::new();

        model.set("key", "value1");
        assert_eq!(model.len(), 1);

        model.set("key", "value2");
        assert_eq!(model.len(), 1);
    }

    #[test]
    fn remove() {
        let mut model = Shrine::new();
        model.set("key", "value");

        model.remove("key2");
        assert_eq!(model.len(), 1);

        model.remove("key");
        assert_eq!(model.len(), 0);

        model.remove("key");
        assert_eq!(model.len(), 0);
    }
}
