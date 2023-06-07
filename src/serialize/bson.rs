use crate::serialize::{Error, SerDe};
use bson::{Bson, RawDocumentBuf};
use serde::ser::Error as BsonError;
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;

pub struct BsonSerDe<D>
where
    D: Serialize + for<'d> Deserialize<'d>,
{
    data: PhantomData<D>,
}

impl<D> BsonSerDe<D>
where
    D: Serialize + for<'d> Deserialize<'d>,
{
    pub fn new() -> Self {
        Self::default()
    }
}

impl<D> Default for BsonSerDe<D>
where
    D: Serialize + for<'d> Deserialize<'d>,
{
    fn default() -> Self {
        Self {
            data: PhantomData::default(),
        }
    }
}

impl<'a, D> SerDe<'a, D> for BsonSerDe<D>
where
    D: Serialize + for<'d> Deserialize<'d>,
{
    fn serialize(&self, data: &D) -> Result<Vec<u8>, Error> {
        match bson::to_bson(data) {
            Ok(data) => match data {
                Bson::Document(d) => Ok(RawDocumentBuf::from_document(&d)
                    .unwrap()
                    .as_bytes()
                    .to_vec()),
                _ => Err(Error::BsonWrite(bson::ser::Error::custom(
                    "Unexpected Bson alternative",
                ))),
            },
            Err(e) => Err(Error::BsonWrite(e)), // todo can we do better?
        }
    }

    fn deserialize(&self, bytes: &[u8]) -> Result<D, Error> {
        bson::from_slice::<D>(bytes).map_err(Error::BsonRead) // todo can we do better?
    }
}
