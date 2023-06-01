pub mod bson;
pub mod json;
pub mod message_pack;

use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use thiserror::Error;

/// Serializer / deserializer trait.
pub trait SerDe<'d, D>
where
    D: Serialize + Deserialize<'d>,
{
    fn serialize(&self, data: &D) -> Result<Vec<u8>, Error>;

    fn deserialize(&self, bytes: &[u8]) -> Result<D, Error>;
}

#[derive(Debug, Error)]
pub enum Error {
    Serialization(String),
    Deserialization(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
