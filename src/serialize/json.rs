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
        Self { data: PhantomData }
    }
}

impl<'a, D> SerDe<'a, D> for JsonSerDe<D>
where
    D: Serialize + for<'d> Deserialize<'d>,
{
    fn serialize(&self, data: &D) -> Result<Vec<u8>, Error> {
        serde_json::to_vec(data).map_err(Error::JsonWrite)
    }

    fn deserialize(&self, bytes: &[u8]) -> Result<D, Error> {
        serde_json::from_slice::<D>(bytes).map_err(Error::JsonRead)
    }
}
