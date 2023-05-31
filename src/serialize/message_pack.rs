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
        Self {
            data: PhantomData::default(),
        }
    }
}

impl<'a, D> SerDe<'a, D> for MessagePackSerDe<D>
where
    D: Serialize + for<'d> Deserialize<'d>,
{
    fn serialize(&self, data: &D) -> Result<Vec<u8>, Error> {
        match rmp_serde::to_vec(data) {
            Ok(bytes) => Ok(bytes),
            Err(_) => Err(Error::Serialization),
        }
    }

    fn deserialize(&self, bytes: &[u8]) -> Result<D, Error> {
        match rmp_serde::from_slice::<D>(bytes) {
            Ok(data) => Ok(data),
            Err(_) => Err(Error::Deserialization),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::serialize::message_pack::MessagePackSerDe;
    use crate::serialize::SerDe;
    use crate::shrine::Shrine;
    use secrecy::ExposeSecret;

    #[test]
    fn serde() {
        let mut shrine = Shrine::new();
        shrine.set("key", "val");

        let serde = MessagePackSerDe::new();

        let bytes = serde.serialize(&shrine).unwrap();
        let shrine = serde.deserialize(bytes.as_slice()).unwrap();

        assert_eq!(
            "val".as_bytes(),
            shrine.get("key").unwrap().expose_secret().as_ref()
        )
    }
}
