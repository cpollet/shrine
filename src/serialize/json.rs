use crate::serialize::{Error, SerDe};
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;

pub struct JsonSerDe<D>
where
    D: Serialize + for<'d> Deserialize<'d>,
{
    data: PhantomData<D>,
}

impl<D> JsonSerDe<D>
where
    D: Serialize + for<'d> Deserialize<'d>,
{
    pub fn new() -> Self {
        Self::default()
    }
}

impl<D> Default for JsonSerDe<D>
where
    D: Serialize + for<'d> Deserialize<'d>,
{
    fn default() -> Self {
        Self {
            data: PhantomData::default(),
        }
    }
}

impl<'a, D> SerDe<'a, D> for JsonSerDe<D>
where
    D: Serialize + for<'d> Deserialize<'d>,
{
    fn serialize(&self, data: &D) -> Result<Vec<u8>, Error> {
        match serde_json::to_vec(data) {
            Ok(bytes) => Ok(bytes),
            Err(e) => Err(Error::Serialization(e.to_string())),
        }
    }

    fn deserialize(&self, bytes: &[u8]) -> Result<D, Error> {
        match serde_json::from_slice::<D>(bytes) {
            Ok(data) => Ok(data),
            Err(e) => Err(Error::Deserialization(e.to_string())),
        }
    }
}
