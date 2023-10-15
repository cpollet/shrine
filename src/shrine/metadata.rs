use crate::shrine::encryption::EncryptionAlgorithm;
use crate::shrine::serialization::SerializationFormat;
use borsh::{BorshDeserialize, BorshSerialize};
use uuid::Uuid;

#[derive(Clone, Debug, BorshSerialize, BorshDeserialize)]
pub enum Metadata {
    V0 {
        uuid: u128,
        /// The algorithm used to encrypt the payload.
        encryption_algorithm: EncryptionAlgorithm,
        /// The serialization format used to serialize the payload.
        serialization_format: SerializationFormat,
    },
}

impl Metadata {
    pub fn version(&self) -> u8 {
        match self {
            Metadata::V0 { .. } => 0,
        }
    }

    pub fn uuid(&self) -> Uuid {
        match self {
            Metadata::V0 { uuid, .. } => Uuid::from_u128(*uuid),
        }
    }

    pub fn encryption_algorithm(&self) -> EncryptionAlgorithm {
        match self {
            Metadata::V0 {
                encryption_algorithm,
                ..
            } => *encryption_algorithm,
        }
    }

    pub fn serialization_format(&self) -> SerializationFormat {
        match self {
            Metadata::V0 {
                serialization_format,
                ..
            } => *serialization_format,
        }
    }
}
