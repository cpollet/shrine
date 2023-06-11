use crate::bytes::SecretBytes;
use std::borrow::Borrow;

use crate::Error;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::hash::Hash;
use std::rc::Rc;

/// Holds the secrets
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Holder {
    /// Secrets and data private to the shrine.
    private: HashMap<String, String>, // fixme should this be secret as well?
    /// Actual user-defined secrets.
    secrets: Node,
}

/// Holds the secrets.
impl Holder {
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets a secret.
    pub fn set<V>(&mut self, key: &str, value: V) -> Result<(), Error>
    where
        V: Into<SecretBytes>,
    {
        self.secrets.set(key, value.into())
    }

    /// Gets a secret's value
    pub fn get(&self, key: &str) -> Result<&SecretBytes, Error> {
        self.secrets.get(key)
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

#[derive(Debug, Serialize, Deserialize)]
enum Node {
    Index(HashMap<String, Box<Node>>),
    Secret(SecretBytes),
}

impl Default for Node {
    fn default() -> Self {
        Self::Index(HashMap::new())
    }
}

impl Node {
    fn set(&mut self, key: &str, value: SecretBytes) -> Result<(), Error> {
        self.set_inner(key, value, key, 0)
    }

    fn set_inner(
        &mut self,
        key: &str,
        value: SecretBytes,
        full_key: &str,
        matched: usize,
    ) -> Result<(), Error> {
        if let Node::Index(index) = self {
            match key.split_once('/') {
                Some((_, "")) => Err(Error::EmptyKey(full_key.to_string())),
                Some((head, tail)) => index.get_or_default_mut(head).set_inner(
                    tail,
                    value,
                    full_key,
                    matched + head.len() + 1,
                ),
                None => {
                    match index.get(key).map(|e| &**e) {
                        None | Some(Node::Secret(_)) => {
                            index.insert(key.to_string(), Box::new(Node::Secret(value)))
                        }
                        Some(Node::Index(_)) => {
                            return Err(Error::KeyIsAnIndex(
                                key.to_string(),
                                full_key[0..matched].to_string(),
                            ))
                        }
                    };
                    Ok(())
                }
            }
        } else {
            Err(Error::KeyIsASecret(
                key.to_string(),
                full_key[0..matched].to_string(),
            ))
        }
    }

    pub fn get(&self, key: &str) -> Result<&SecretBytes, Error> {
        self.get_inner(key, key)
    }

    fn get_inner(&self, key: &str, full_key: &str) -> Result<&SecretBytes, Error> {
        match key.split_once('/') {
            Some((_, "")) => Err(Error::EmptyKey(full_key.to_string())),
            Some((head, tail)) => match self {
                Node::Secret(_) => Err(Error::KeyNotFound(full_key.to_string())),
                Node::Index(index) => match index.get(head) {
                    None => Err(Error::KeyNotFound(full_key.to_string())),
                    Some(node) => node.get_inner(tail, full_key),
                },
            },
            None => match self {
                Node::Secret(_) => Err(Error::KeyNotFound(full_key.to_string())),
                Node::Index(index) => match index.get(key).map(|e| &**e) {
                    Some(Node::Secret(bytes)) => Ok(bytes),
                    _ => Err(Error::KeyNotFound(full_key.to_string())),
                },
            },
        }
    }

    fn keys(&self) -> Vec<String> {
        match &self {
            Node::Secret(_) => panic!("Node::Secret.keys() called"),
            Node::Index(_) => {
                let mut keys = self.keys_inner(Rc::new(String::new()));
                keys.sort_unstable();
                keys
            }
        }
    }

    fn keys_inner(&self, prefix: Rc<String>) -> Vec<String> {
        match &self {
            Node::Secret(_) => vec![prefix.to_string()],
            Node::Index(index) => index
                .keys()
                .flat_map(|k| {
                    let prefix = match prefix.as_str() {
                        "" => Rc::new(k.to_string()),
                        prefix => Rc::new(format!("{}/{}", prefix, k)),
                    };
                    index.get(k).expect("we have it").keys_inner(prefix)
                })
                .collect(),
        }
    }

    fn remove(&mut self, key: &str) -> bool {
        if let Node::Index(index) = self {
            match key.split_once('/') {
                Some((_, "")) => false,
                Some((head, tail)) => {
                    if let Some(node) = index.get_mut(head) {
                        node.remove(tail)
                    } else {
                        false
                    }
                }
                None => match index.get(key).map(|e| &**e) {
                    Some(Node::Secret(_)) => {
                        index.remove(key);
                        true
                    }
                    _ => false,
                },
            }
        } else {
            panic!("Node::Secret.remove() called")
        }
    }

    fn len(&self) -> u64 {
        match &self {
            Node::Secret(_) => 1,
            Node::Index(index) => index.values().map(|v| v.len()).sum(),
        }
    }
}

trait GetOrDefault<K, V>
where
    K: Hash + Eq,
    V: Default,
{
    fn get_or_default_mut<'q, Q: ?Sized>(&mut self, key: &'q Q) -> &mut V
    where
        K: Borrow<Q> + From<&'q Q>,
        Q: Hash + Eq;
}

impl<K, V> GetOrDefault<K, V> for HashMap<K, V>
where
    K: Hash + Eq,
    V: Default,
{
    fn get_or_default_mut<'q, Q: ?Sized>(&mut self, key: &'q Q) -> &mut V
    where
        K: Borrow<Q> + From<&'q Q>,
        Q: Hash + Eq,
    {
        if self.get(key).is_none() {
            self.insert(key.into(), V::default());
        }
        self.get_mut(key).expect("we just inserted it")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_get_len() {
        let mut holder = Holder::new();

        holder.set("key", "value1").unwrap();
        assert_eq!(holder.len(), 1);
        assert_eq!(
            holder.get("key").unwrap().expose_secret_as_bytes(),
            "value1".as_bytes()
        );

        holder.set("key", "value2").unwrap();

        assert_eq!(holder.len(), 1);
        assert_eq!(
            holder.get("key").unwrap().expose_secret_as_bytes(),
            "value2".as_bytes()
        );

        assert_eq!(
            holder.get("unknown").unwrap_err().to_string(),
            "Key `unknown` does not exist"
        )
    }

    #[test]
    fn len_nested() {
        let mut holder = Holder::new();

        holder.set("a/b", "v").unwrap();
        holder.set("a/c", "v").unwrap();
        holder.set("a/d/a", "v").unwrap();

        assert_eq!(holder.len(), 3);
    }

    #[test]
    fn set_get_nested() {
        let mut holder = Holder::new();

        holder.set("a/b", "v").unwrap();

        assert_eq!(
            holder.get("a").unwrap_err().to_string(),
            "Key `a` does not exist"
        );
        assert_eq!(
            holder.get("a/b/c").unwrap_err().to_string(),
            "Key `a/b/c` does not exist"
        );

        assert_eq!(
            holder.get("a/b").unwrap().expose_secret_as_bytes(),
            "v".as_bytes()
        );
    }

    #[test]
    fn set_key_is_secret() {
        let mut holder = Holder::new();

        holder.set("a/b", "v").unwrap();
        assert_eq!(
            holder.set("a/b/c", "v").unwrap_err().to_string(),
            "Key `c` is a secret in `a/b/`"
        );
    }

    #[test]
    fn set_key_is_index() {
        let mut holder = Holder::new();

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
        let mut holder = Holder::new();
        assert_eq!(
            holder.set("a/", "v").unwrap_err().to_string(),
            "Key is empty in `a/`"
        );
    }

    #[test]
    fn get_end_with_slash() {
        let holder = Holder::new();
        assert_eq!(
            holder.get("a/").unwrap_err().to_string(),
            "Key is empty in `a/`"
        );
    }

    #[test]
    fn set_replace() {
        let mut holder = Holder::new();

        holder.set("a/b", "1").unwrap();
        holder.set("a/b", "2").unwrap();

        assert_eq!(
            holder.get("a/b").unwrap().expose_secret_as_bytes(),
            "2".as_bytes()
        );
    }

    #[test]
    fn keys() {
        let mut holder = Holder::new();

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
        let mut holder = Holder::new();

        assert!(holder.is_empty());

        holder.set("k", "v").unwrap();

        assert!(!holder.is_empty());
    }

    #[test]
    fn remove() {
        let mut holder = Holder::new();
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
            let mut holder = Holder::new();
            holder.set("key", "val").unwrap();

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
            holder.set("key", "val").unwrap();

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
            holder.set("key", "val").unwrap();

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
