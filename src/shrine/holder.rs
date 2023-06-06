use crate::bytes::SecretBytes;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Holds the secrets
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Holder {
    /// Secrets and data private to the shrine.
    private: HashMap<String, String>, // fixme should this be secret as well?
    /// Actual user-defined secrets.
    secrets: HashMap<String, SecretBytes>,
}

/// Holds the secrets.
impl Holder {
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

    /// Returns all the keys, sorted in alphabetical order.
    pub fn keys(&self) -> Vec<String> {
        let mut keys = self
            .secrets
            .keys()
            .map(|k| k.to_string())
            .collect::<Vec<String>>();
        keys.sort_unstable();
        keys
    }

    /// Removes a secret.
    pub fn remove<'k, K>(&mut self, key: K)
    where
        K: Into<&'k str>,
    {
        self.secrets.remove(key.into());
    }

    /// Returns the count of secrets in the holder.
    pub fn len(&self) -> u64 {
        self.secrets.len() as u64
    }

    /// Returns whether the holder has secrets or not.
    pub fn is_empty(&self) -> bool {
        self.secrets.is_empty()
    }

    /// Sets a private value.
    pub fn set_private<K, V>(&mut self, key: K, value: V)
    where
        K: Into<String>,
        V: Into<String>,
    {
        self.private.insert(key.into(), value.into());
    }

    /// Gets a private value.
    pub fn get_private<'k, K>(&self, key: K) -> Option<&String>
    where
        K: Into<&'k str>,
    {
        self.private.get(key.into())
    }

    /// Removes a private value.
    pub fn remove_private<'k, K>(&mut self, key: K)
    where
        K: Into<&'k str>,
    {
        self.private.remove(key.into());
    }

    /// Returns all the private keys, sorted in alphabetical order.
    pub fn keys_private(&self) -> Vec<String> {
        let mut keys = self
            .private
            .keys()
            .map(|k| k.to_string())
            .collect::<Vec<String>>();
        keys.sort_unstable();
        keys
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_get_len() {
        let mut holder = Holder::new();

        holder.set("key", "value1");
        assert_eq!(holder.len(), 1);
        assert_eq!(
            holder.get("key").unwrap().expose_secret_as_bytes(),
            "value1".as_bytes()
        );

        holder.set("key", "value2");
        assert_eq!(holder.len(), 1);
        assert_eq!(
            holder.get("key").unwrap().expose_secret_as_bytes(),
            "value2".as_bytes()
        );

        assert!(holder.get("unknown").is_none());
    }

    #[test]
    fn keys() {
        let mut holder = Holder::new();

        holder.set("key1", "val1");
        holder.set("abc", "val2");

        let keys = holder.keys();
        assert_eq!(keys.len(), 2);
        assert_eq!(keys.get(0).unwrap(), "abc");
        assert_eq!(keys.get(1).unwrap(), "key1");
    }

    #[test]
    fn is_empty() {
        let mut holder = Holder::new();

        assert!(holder.is_empty());

        holder.set("k", "v");

        assert!(!holder.is_empty());
    }

    #[test]
    fn remove() {
        let mut holder = Holder::new();
        holder.set("key", "value");

        holder.remove("key2");
        assert_eq!(holder.len(), 1);

        holder.remove("key");
        assert_eq!(holder.len(), 0);

        holder.remove("key");
        assert_eq!(holder.len(), 0);
    }

    #[cfg(test)]
    mod bson {
        use crate::serialize::bson::BsonSerDe;
        use crate::serialize::SerDe;
        use crate::shrine::holder::Holder;

        #[test]
        fn serde() {
            let mut holder = Holder::new();
            holder.set("key", "val");

            let serde = BsonSerDe::new();

            let bytes = serde.serialize(&holder).unwrap();
            let holder = serde.deserialize(bytes.as_slice()).unwrap();

            assert_eq!(
                "val".as_bytes(),
                holder.get("key").unwrap().expose_secret_as_bytes()
            )
        }
    }

    #[cfg(test)]
    mod json {
        use crate::serialize::json::JsonSerDe;
        use crate::serialize::SerDe;
        use crate::shrine::holder::Holder;

        #[test]
        fn serde() {
            let mut holder = Holder::new();
            holder.set("key", "val");

            let serde = JsonSerDe::new();

            let bytes = serde.serialize(&holder).unwrap();
            let holder = serde.deserialize(bytes.as_slice()).unwrap();

            assert_eq!(
                "val".as_bytes(),
                holder.get("key").unwrap().expose_secret_as_bytes()
            )
        }
    }

    #[cfg(test)]
    mod message_page {
        use crate::serialize::message_pack::MessagePackSerDe;
        use crate::serialize::SerDe;
        use crate::shrine::holder::Holder;

        #[test]
        fn serde() {
            let mut holder = Holder::new();
            holder.set("key", "val");

            let serde = MessagePackSerDe::new();

            let bytes = serde.serialize(&holder).unwrap();
            let holder = serde.deserialize(bytes.as_slice()).unwrap();

            assert_eq!(
                "val".as_bytes(),
                holder.get("key").unwrap().expose_secret_as_bytes()
            )
        }
    }
}
