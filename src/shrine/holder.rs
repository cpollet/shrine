pub mod node;

use crate::shrine::holder::node::Node;
use crate::Error;
use serde::{Deserialize, Serialize};

/// Holds the secrets
#[derive(Serialize, Deserialize, Debug)]
pub struct Holder<T> {
    /// Secrets and data private to the shrine.
    private: Node<T>,
    /// Actual user-defined secrets.
    secrets: Node<T>,
}

impl<T> Default for Holder<T> {
    fn default() -> Self {
        Self {
            private: Node::default(),
            secrets: Node::default(),
        }
    }
}

/// Holds the secrets.
impl<T> Holder<T> {
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets a secret.
    pub fn set<V>(&mut self, key: &str, value: V) -> Result<(), Error>
    where
        V: Into<T>,
    {
        self.secrets.set(key, value.into())
    }

    /// Gets a secret's value
    pub fn get(&self, key: &str) -> Result<&T, Error> {
        self.secrets.get(key)
    }

    pub fn get_mut(&mut self, key: &str) -> Result<&mut T, Error> {
        self.secrets.get_mut(key)
    }

    /// Returns all the keys, sorted in alphabetical order.
    pub fn keys(&self) -> Vec<String> {
        self.secrets.keys()
    }

    /// Removes a secret.
    pub fn remove(&mut self, key: &str) -> bool {
        self.secrets.remove(key)
    }

    /// Returns the count of secrets in the holder.
    pub fn len(&self) -> u64 {
        self.secrets.len()
    }

    /// Returns whether the holder has secrets or not.
    pub fn is_empty(&self) -> bool {
        match &self.secrets {
            Node::Index(index) => index.is_empty(),
            Node::Secret(_) => panic!("root is not an index"),
        }
    }

    /// Sets a private value.
    pub fn set_private<V>(&mut self, key: &str, value: V) -> Result<(), Error>
    where
        V: Into<T>,
    {
        self.private.set(key, value.into())
    }

    /// Gets a private value.
    pub fn get_private(&self, key: &str) -> Result<&T, Error> {
        self.private.get(key)
    }

    /// Removes a private value.
    pub fn remove_private(&mut self, key: &str) {
        self.private.remove(key);
    }

    /// Returns all the private keys, sorted in alphabetical order.
    pub fn keys_private(&self) -> Vec<String> {
        self.private.keys()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_get_len() {
        let mut holder = Holder::<String>::new();

        holder.set("key", "value1").unwrap();
        assert_eq!(holder.len(), 1);
        assert_eq!(holder.get("key").unwrap(), "value1");

        holder.set("key", "value2").unwrap();

        assert_eq!(holder.len(), 1);
        assert_eq!(holder.get("key").unwrap(), "value2");

        assert_eq!(
            holder.get("unknown").unwrap_err().to_string(),
            "Key `unknown` does not exist"
        )
    }

    #[test]
    fn len_nested() {
        let mut holder = Holder::<String>::new();

        holder.set("a/b", "v").unwrap();
        holder.set("a/c", "v").unwrap();
        holder.set("a/d/a", "v").unwrap();

        assert_eq!(holder.len(), 3);
    }

    #[test]
    fn set_get_nested() {
        let mut holder = Holder::<String>::new();

        holder.set("a/b", "v").unwrap();

        assert_eq!(
            holder.get("a").unwrap_err().to_string(),
            "Key `a` does not exist"
        );
        assert_eq!(
            holder.get("a/b/c").unwrap_err().to_string(),
            "Key `a/b/c` does not exist"
        );

        assert_eq!(holder.get("a/b").unwrap(), "v");
    }

    #[test]
    fn set_key_is_secret() {
        let mut holder = Holder::<String>::new();

        holder.set("a/b", "v").unwrap();
        assert_eq!(
            holder.set("a/b/c", "v").unwrap_err().to_string(),
            "Key `c` is a secret in `a/b/`"
        );
    }

    #[test]
    fn set_key_is_index() {
        let mut holder = Holder::<String>::new();

        holder.set("a/b", "v").unwrap();
        assert_eq!(
            holder.set("a", "v").unwrap_err().to_string(),
            "Key `a` is an index in ``"
        );

        holder.set("1/2/3", "v").unwrap();
        assert_eq!(
            holder.set("1/2", "v").unwrap_err().to_string(),
            "Key `2` is an index in `1/`"
        );
    }

    #[test]
    fn set_end_with_slash() {
        let mut holder = Holder::<String>::new();
        assert_eq!(
            holder.set("a/", "v").unwrap_err().to_string(),
            "Key is empty in `a/`"
        );
    }

    #[test]
    fn get_end_with_slash() {
        let holder = Holder::<String>::new();
        assert_eq!(
            holder.get("a/").unwrap_err().to_string(),
            "Key is empty in `a/`"
        );
    }

    #[test]
    fn set_replace() {
        let mut holder = Holder::<String>::new();

        holder.set("a/b", "1").unwrap();
        holder.set("a/b", "2").unwrap();

        assert_eq!(holder.get("a/b").unwrap(), "2");
    }

    #[test]
    fn keys() {
        let mut holder = Holder::<String>::new();

        holder.set("key", "v").unwrap();
        holder.set("a/b/c", "v").unwrap();
        holder.set("a/b/d", "v").unwrap();
        holder.set("1/2", "v").unwrap();

        let keys = holder.keys();
        assert_eq!(keys.len(), 4);
        assert_eq!(keys.get(0).unwrap(), "1/2");
        assert_eq!(keys.get(1).unwrap(), "a/b/c");
        assert_eq!(keys.get(2).unwrap(), "a/b/d");
        assert_eq!(keys.get(3).unwrap(), "key");
    }

    #[test]
    fn is_empty() {
        let mut holder = Holder::<String>::new();

        assert!(holder.is_empty());

        holder.set("k", "v").unwrap();

        assert!(!holder.is_empty());
    }

    #[test]
    fn remove() {
        let mut holder = Holder::<String>::new();
        holder.set("a/b/c", "v").unwrap();
        holder.set("a/b/d", "v").unwrap();
        holder.set("a/e", "v").unwrap();

        holder.remove("a/e");
        assert_eq!(holder.len(), 2);

        holder.remove("a/b/c");
        assert_eq!(holder.len(), 1);

        holder.remove("a/b/d");
        assert_eq!(holder.len(), 0);

        holder.remove("a/b/d");
        assert_eq!(holder.len(), 0);
    }

    #[cfg(test)]
    mod bson {
        use crate::serialize::bson::BsonSerDe;
        use crate::serialize::SerDe;
        use crate::shrine::holder::Holder;

        #[test]
        fn serde() {
            let mut holder = Holder::<String>::new();
            holder.set("key", "val").unwrap();

            let serde = BsonSerDe::new();

            let bytes = serde.serialize(&holder).unwrap();
            let holder = serde.deserialize(bytes.as_slice()).unwrap();

            assert_eq!("val", holder.get("key").unwrap())
        }
    }

    #[cfg(test)]
    mod json {
        use crate::serialize::json::JsonSerDe;
        use crate::serialize::SerDe;
        use crate::shrine::holder::Holder;

        #[test]
        fn serde() {
            let mut holder = Holder::<String>::new();
            holder.set("key", "val").unwrap();

            let serde = JsonSerDe::new();

            let bytes = serde.serialize(&holder).unwrap();
            let holder = serde.deserialize(bytes.as_slice()).unwrap();

            assert_eq!("val", holder.get("key").unwrap())
        }
    }

    #[cfg(test)]
    mod message_pack {
        use crate::serialize::message_pack::MessagePackSerDe;
        use crate::serialize::SerDe;
        use crate::shrine::holder::Holder;

        #[test]
        fn serde() {
            let mut holder = Holder::<String>::new();
            holder.set("key", "val").unwrap();

            let serde = MessagePackSerDe::new();

            let bytes = serde.serialize(&holder).unwrap();
            let holder = serde.deserialize(bytes.as_slice()).unwrap();

            assert_eq!("val", holder.get("key").unwrap())
        }
    }
}
