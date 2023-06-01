use crate::serialize::{Error, SerDe};
use bson::{Bson, RawDocumentBuf};
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
                _ => Err(Error::Serialization),
            },
            Err(_) => Err(Error::Serialization),
        }
    }

    fn deserialize(&self, bytes: &[u8]) -> Result<D, Error> {
        match bson::from_slice::<D>(bytes) {
            Ok(data) => Ok(data),
            Err(_) => Err(Error::Deserialization),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::serialize::bson::BsonSerDe;
    use crate::serialize::SerDe;
    use crate::shrine::Shrine;

    #[test]
    fn serde() {
        let mut shrine = Shrine::new();
        shrine.set("key", "val");

        let serde = BsonSerDe::new();

        let bytes = serde.serialize(&shrine).unwrap();
        let shrine = serde.deserialize(bytes.as_slice()).unwrap();

        assert_eq!(
            "val".as_bytes(),
            shrine.get("key").unwrap().expose_secret_as_bytes()
        )
    }
}
