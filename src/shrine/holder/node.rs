use crate::Error;
use serde::{Deserialize, Serialize};
use std::borrow::Borrow;
use std::collections::HashMap;
use std::hash::Hash;
use std::rc::Rc;

#[derive(Debug, Serialize, Deserialize)]
pub enum Node<T> {
    Index(HashMap<String, Box<Node<T>>>),
    Secret(T),
}

impl<T> Default for Node<T> {
    fn default() -> Self {
        Self::Index(HashMap::new())
    }
}

impl<T> Node<T> {
    pub fn set(&mut self, key: &str, value: T) -> Result<(), Error> {
        self.set_inner(key, value, key, 0)
    }

    fn set_inner(
        &mut self,
        key: &str,
        value: T,
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

    pub fn get(&self, key: &str) -> Result<&T, Error> {
        self.get_inner(key, key)
    }

    fn get_inner(&self, key: &str, full_key: &str) -> Result<&T, Error> {
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

    pub fn get_mut(&mut self, key: &str) -> Result<&mut T, Error> {
        self.get_mut_inner(key, key)
    }

    fn get_mut_inner(&mut self, key: &str, full_key: &str) -> Result<&mut T, Error> {
        match key.split_once('/') {
            Some((_, "")) => Err(Error::EmptyKey(full_key.to_string())),
            Some((head, tail)) => match self {
                Node::Secret(_) => Err(Error::KeyNotFound(full_key.to_string())),
                Node::Index(index) => match index.get_mut(head) {
                    None => Err(Error::KeyNotFound(full_key.to_string())),
                    Some(node) => node.get_mut_inner(tail, full_key),
                },
            },
            None => match self {
                Node::Secret(_) => Err(Error::KeyNotFound(full_key.to_string())),
                Node::Index(index) => match index.get_mut(key).map(|e| &mut **e) {
                    Some(Node::Secret(bytes)) => Ok(bytes),
                    _ => Err(Error::KeyNotFound(full_key.to_string())),
                },
            },
        }
    }

    pub fn keys(&self) -> Vec<String> {
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

    pub fn remove(&mut self, key: &str) -> bool {
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

    pub fn len(&self) -> u64 {
        match &self {
            Node::Secret(_) => 1,
            Node::Index(index) => index.values().map(|v| v.len()).sum(),
        }
    }

    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        match &self {
            Node::Secret(_) => unreachable!(),
            Node::Index(index) => index.is_empty(),
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
