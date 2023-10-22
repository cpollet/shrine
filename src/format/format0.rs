use crate::format::Format;
use crate::serialize::bson::BsonSerDe;
use crate::serialize::json::JsonSerDe;
use crate::serialize::message_pack::MessagePackSerDe;
use crate::serialize::SerDe;
use crate::shrine::encryption::EncryptionAlgorithm;
use crate::shrine::holder::node::Node;
use crate::shrine::holder::Holder;
use crate::shrine::local::{Aes, Clear, Closed, InMemoryShrine, LocalShrine, Secrets};
use crate::shrine::serialization::SerializationFormat;
use crate::values::bytes::SecretBytes;
use crate::values::secret::{Mode, Secret};
use crate::Error;
use borsh::{BorshDeserialize, BorshSerialize};
use chrono::{DateTime, Utc};
use secrecy::zeroize::{ZeroizeOnDrop, Zeroizing};
use secrecy::Zeroize;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

#[derive(Debug, Default)]
pub struct Format0 {
    serialization: SerializationFormat,
}

impl Format for Format0 {
    fn version(&self) -> u8 {
        0
    }

    fn is_readonly(&self) -> bool {
        true
    }

    fn serialization_format(&self) -> SerializationFormat {
        self.serialization
    }

    fn set_serialization_format(&mut self, format: SerializationFormat) {
        self.serialization = format;
    }

    fn deserialize_secret(&self, bytes: Zeroizing<Vec<u8>>) -> Result<Secrets, Error> {
        let serializer: Box<dyn SerDe<HolderV0<SecretV0>>> = match self.serialization {
            SerializationFormat::Bson => Box::new(BsonSerDe::<HolderV0<SecretV0>>::new()),
            SerializationFormat::Json => Box::new(JsonSerDe::<HolderV0<SecretV0>>::new()),
            SerializationFormat::MessagePack => {
                Box::new(MessagePackSerDe::<HolderV0<SecretV0>>::new())
            }
        };

        let holder_v0 = serializer.deserialize(bytes.as_slice())?;
        let mut holder_last = Holder::<Secret>::new();

        for key in holder_v0.secrets.keys() {
            let value = holder_v0.secrets.get(&key).expect("key exists");
            let secret = Secret::new(
                SecretBytes::from(value.value.as_ref()),
                value.mode, // todo add dates
            );
            holder_last.set(&key, secret)?;
        }

        for (k, v) in holder_v0.private.iter() {
            let secret = Secret::new(SecretBytes::from(v.as_bytes()), Mode::Text);
            holder_last.set_private(k, secret)?;
        }

        Ok(holder_last)
    }

    fn serialize_secrets(&self, _secrets: &Secrets) -> Result<Zeroizing<Vec<u8>>, Error> {
        unimplemented!("This format is not supported anymore")
    }

    fn serialize(&self, _uuid: Uuid, _encryption: EncryptionAlgorithm, _payload: &[u8]) -> Vec<u8> {
        unimplemented!("This format is not supported anymore")
    }
}

impl Format0 {
    pub fn read(uuid: Uuid, bytes: &[u8]) -> Result<InMemoryShrine, Error> {
        let shrine = ShrineV0::try_from_slice(bytes).map_err(Error::IoRead)?;

        let payload = Closed::new(shrine.payload);
        let format = Arc::new(Mutex::new(Format0 {
            serialization: shrine.serialization_format,
        }));

        Ok(match shrine.encryption_algorithm {
            EncryptionAlgorithm::Aes => InMemoryShrine::Aes(LocalShrine::new_closed(
                uuid,
                payload,
                Aes::no_password(),
                format,
            )),
            EncryptionAlgorithm::Plain => {
                InMemoryShrine::Clear(LocalShrine::new_closed(uuid, payload, Clear, format))
            }
        })
    }
}

#[derive(Serialize, Deserialize)]
struct HolderV0<T> {
    private: HashMap<String, String>,
    secrets: Node<T>,
}

#[derive(Serialize, Deserialize)]
struct SecretV0 {
    value: bytes::BytesMut,
    mode: Mode,
    created_by: String,
    created_at: DateTime<Utc>,
    updated_by: Option<String>,
    updated_at: Option<DateTime<Utc>>,
}

impl Zeroize for SecretV0 {
    fn zeroize(&mut self) {
        self.value.fill(0);
    }
}

impl ZeroizeOnDrop for SecretV0 {}

#[derive(BorshSerialize, BorshDeserialize)]
struct ShrineV0 {
    encryption_algorithm: EncryptionAlgorithm,
    serialization_format: SerializationFormat,
    payload: Vec<u8>,
}
