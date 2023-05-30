#[derive(Debug)]
pub struct ShrineFile {
    /// Always "shrine".
    magic_number: [u8; 6],
    /// The shrine file format version.
    version: u8,
    /// The algorithm used to encrypt the payload.
    encryption_algorithm: EncryptionAlgorithm,
    /// The serialization format used to serialize the payload.
    serialization_format: SerializationFormat,
    /// The serialized then encrypted payload.
    payload: Vec<u8>,
}

impl ShrineFile {
    pub fn is_valid(&self) -> bool {
        self.magic_number.as_slice() == "shrine".as_bytes()
    }

    pub fn version(&self) -> u8 {
        self.version
    }

    pub fn encryption_algorithm(&self) -> EncryptionAlgorithm {
        self.encryption_algorithm
    }

    pub fn serialization_format(&self) -> SerializationFormat {
        self.serialization_format
    }

    pub fn payload(&self) -> &[u8] {
        self.payload.as_slice()
    }
}

/// Builds a default `ShrineFile`.
///
/// ```
/// # use crate::grave::file_format::{EncryptionAlgorithm, SerializationFormat, ShrineFile};
/// let file = ShrineFile::default();
/// assert_eq!(file.is_valid(), true);
/// assert_eq!(file.version(), 0);
/// assert_eq!(file.encryption_algorithm(), EncryptionAlgorithm::Plain);
/// assert_eq!(file.serialization_format(), SerializationFormat::Bson);
/// assert_eq!(file.payload().len(), 0);
///```
impl Default for ShrineFile {
    fn default() -> Self {
        Self {
            magic_number: [b's', b'h', b'r', b'i', b'n', b'e'],
            version: 0,
            encryption_algorithm: EncryptionAlgorithm::default(),
            serialization_format: SerializationFormat::default(),
            payload: Vec::default(),
        }
    }
}

/// The list of encryption algorithms used to encrypt the payload.
#[derive(Default, Debug, Clone, Copy, Eq, PartialEq)]
pub enum EncryptionAlgorithm {
    /// No encryption
    #[default]
    Plain,
}

/// The serialization format
#[derive(Default, Debug, Clone, Copy, Eq, PartialEq)]
pub enum SerializationFormat {
    /// BSON, the data storage and network transfer format used by MongoDB.
    #[default]
    Bson,
    /// JSON, the ubiquitous JavaScript Object Notation used by many HTTP APIs.
    Json,
}
