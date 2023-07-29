use crate::serialize::{Error, SerDe};
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;

pub struct MessagePackSerDe<D>
where
    D: Serialize + for<'d> Deserialize<'d>,
{
    data: PhantomData<D>,
}

impl<D> MessagePackSerDe<D>
where
    D: Serialize + for<'d> Deserialize<'d>,
{
    pub fn new() -> Self {
        Self::default()
    }
}

impl<D> Default for MessagePackSerDe<D>
where
    D: Serialize + for<'d> Deserialize<'d>,
{
    fn default() -> Self {
        Self { data: PhantomData }
    }
}

impl<'a, D> SerDe<'a, D> for MessagePackSerDe<D>
where
    D: Serialize + for<'d> Deserialize<'d>,
{
    fn serialize(&self, data: &D) -> Result<Vec<u8>, Error> {
        rmp_serde::to_vec(data).map_err(Error::MessagePackWrite)
    }

    fn deserialize(&self, bytes: &[u8]) -> Result<D, Error> {
        rmp_serde::from_slice::<D>(bytes).map_err(Error::MessagePackRead)
    }
}
