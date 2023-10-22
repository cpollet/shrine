use crate::format::Format;
use crate::shrine::encryption::EncryptionAlgorithm;
use crate::shrine::local::{Aes, Clear, Closed, InMemoryShrine, LocalShrine, Secrets};
use crate::shrine::serialization::SerializationFormat;
use crate::Error;
use secrecy::zeroize::Zeroizing;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

#[derive(Debug, Default)]
pub struct Format1 {
    serialization: SerializationFormat,
}

impl Format for Format1 {
    fn version(&self) -> u8 {
        1
    }

    fn is_readonly(&self) -> bool {
        false
    }

    fn serialization_format(&self) -> SerializationFormat {
        self.serialization
    }

    fn set_serialization_format(&mut self, format: SerializationFormat) {
        self.serialization = format;
    }

    fn deserialize_secret(&self, bytes: Zeroizing<Vec<u8>>) -> Result<Secrets, Error> {
        self.serialization_format().serializer().deserialize(&bytes)
    }

    fn serialize_secrets(&self, secrets: &Secrets) -> Result<Zeroizing<Vec<u8>>, Error> {
        self.serialization_format()
            .serializer()
            .serialize(secrets)
            .map(Zeroizing::new)
    }

    fn serialize(&self, uuid: Uuid, encryption: EncryptionAlgorithm, payload: &[u8]) -> Vec<u8> {
        let mut vec = Vec::<u8>::with_capacity(6 + 1 + 1 + 1 + payload.len());
        vec.extend_from_slice(b"shrine".as_slice());
        vec.push(1);
        vec.extend_from_slice(uuid.as_ref());
        vec.push(match encryption {
            EncryptionAlgorithm::Plain => 0,
            EncryptionAlgorithm::Aes => 1,
        });
        vec.push(match self.serialization {
            SerializationFormat::Bson => 0,
            SerializationFormat::Json => 1,
            SerializationFormat::MessagePack => 2,
        });
        vec.extend_from_slice(payload);
        vec
    }
}

impl Format1 {
    pub fn read(uuid: Uuid, bytes: &[u8]) -> Result<InMemoryShrine, Error> {
        let (enc, bytes) = Self::encryption(bytes)?;
        let (ser, bytes) = Self::serialization(bytes)?;

        let mut vec = Vec::with_capacity(bytes.len());
        vec.extend_from_slice(bytes);
        let payload = Closed::new(vec);

        let format = Arc::new(Mutex::new(Format1 { serialization: ser }));

        match enc {
            EncryptionAlgorithm::Aes => Ok(InMemoryShrine::Aes(LocalShrine::new_closed(
                uuid,
                payload,
                Aes::no_password(),
                format,
            ))),
            EncryptionAlgorithm::Plain => Ok(InMemoryShrine::Clear(LocalShrine::new_closed(
                uuid, payload, Clear, format,
            ))),
        }
    }

    fn encryption(bytes: &[u8]) -> Result<(EncryptionAlgorithm, &[u8]), Error> {
        if bytes.is_empty() {
            return Err(Error::InvalidFormat(
                "No encryption information found".to_string(),
            ));
        }

        match bytes[0] {
            0 => Ok((EncryptionAlgorithm::Plain, &bytes[1..])),
            1 => Ok((EncryptionAlgorithm::Aes, &bytes[1..])),
            _ => Err(Error::InvalidFormat("Unknown encryption".to_string())),
        }
    }

    fn serialization(bytes: &[u8]) -> Result<(SerializationFormat, &[u8]), Error> {
        if bytes.is_empty() {
            return Err(Error::InvalidFormat(
                "No serialization information found".to_string(),
            ));
        }

        match bytes[0] {
            0 => Ok((SerializationFormat::Bson, &bytes[1..])),
            1 => Ok((SerializationFormat::Json, &bytes[1..])),
            2 => Ok((SerializationFormat::MessagePack, &bytes[1..])),
            _ => Err(Error::InvalidFormat("Unknown serialization".to_string())),
        }
    }
}
