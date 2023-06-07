pub mod bson;
pub mod json;
pub mod message_pack;

use crate::Error;
use serde::{Deserialize, Serialize};

/// Serializer / deserializer trait.
pub trait SerDe<'d, D>
where
    D: Serialize + Deserialize<'d>,
{
    fn serialize(&self, data: &D) -> Result<Vec<u8>, Error>;

    fn deserialize(&self, bytes: &[u8]) -> Result<D, Error>;
}
