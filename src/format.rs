pub mod format1;

use crate::format::format1::Format1;
use crate::shrine::encryption::EncryptionAlgorithm;
use crate::shrine::local::{InMemoryShrine, Secrets};
use crate::shrine::serialization::SerializationFormat;
use crate::Error;
use secrecy::zeroize::Zeroizing;
use std::fmt::Debug;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

pub fn default() -> Arc<Mutex<dyn Format>> {
    Arc::new(Mutex::new(Format1::default()))
}

pub trait Format: Debug + Send {
    fn version(&self) -> u8;

    fn serialization_format(&self) -> SerializationFormat;

    fn set_serialization_format(&mut self, format: SerializationFormat);

    fn deserialize(&self, bytes: Zeroizing<Vec<u8>>) -> Result<Secrets, Error>;

    fn serialize_secrets(&self, secrets: &Secrets) -> Result<Zeroizing<Vec<u8>>, Error>;

    fn serialize(&self, uuid: Uuid, encryption: EncryptionAlgorithm, payload: &[u8]) -> Vec<u8>;
}

pub fn read(bytes: &[u8]) -> Result<InMemoryShrine, Error> {
    let bytes = consume_marker(bytes)?;
    let (version, bytes) = version(bytes)?;
    let (uuid, bytes) = uuid(bytes)?;

    match version {
        0 => todo!(),
        1 => Format1::read(uuid, bytes),
        v => Err(Error::UnsupportedVersion(v)),
    }
}

fn consume_marker(bytes: &[u8]) -> Result<&[u8], Error> {
    if bytes.len() < 6 || &bytes[0..6] != b"shrine" {
        return Err(Error::InvalidFormat("Marker not found".to_string()));
    }

    Ok(&bytes[6..])
}

fn version(bytes: &[u8]) -> Result<(u8, &[u8]), Error> {
    if bytes.is_empty() {
        return Err(Error::InvalidFormat("Version not found".to_string()));
    }

    Ok((bytes[0], &bytes[1..]))
}

fn uuid(bytes: &[u8]) -> Result<(Uuid, &[u8]), Error> {
    if bytes.len() < 16 {
        return Err(Error::InvalidFormat("UUID not found".to_string()));
    }

    Ok((
        Uuid::from_slice(&bytes[0..16]).expect("Uuid not found"),
        &bytes[16..],
    ))
}

#[cfg(test)]
mod tests {
    use crate::shrine::local::InMemoryShrine;
    use uuid::Uuid;

    #[test]
    pub fn read() {
        let uuid = Uuid::new_v4();

        let mut bytes = Vec::new();
        bytes.extend_from_slice(b"shrine".as_slice());
        bytes.push(1);
        bytes.extend_from_slice(uuid.as_ref());
        bytes.push(0);
        bytes.push(0);

        let shrine = super::read(&bytes).unwrap();

        let shrine = match shrine {
            InMemoryShrine::Clear(s) => s,
            InMemoryShrine::Aes(_) => panic!("Expected Clear, got Aes"),
        };

        assert_eq!(shrine.uuid(), uuid);
    }
}
