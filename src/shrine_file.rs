use crate::encrypt::aes::Aes;
use crate::encrypt::plain::Plain;
use crate::encrypt::EncDec;
use crate::serialize::bson::BsonSerDe;
use crate::serialize::json::JsonSerDe;
use crate::serialize::message_pack::MessagePackSerDe;
use crate::serialize::SerDe;
use crate::shrine::Shrine;
use std::fmt::{Display, Formatter};

use borsh::{BorshDeserialize, BorshSerialize};

use secrecy::Secret;

use thiserror::Error;

/// Max supported file version
const VERSION: u8 = 0;

#[derive(Debug, BorshSerialize, BorshDeserialize)]
pub struct ShrineFile {
    /// Always "shrine".
    magic_number: [u8; 6],
    metadata: Metadata,
    /// The serialized then encrypted payload.
    payload: Vec<u8>,
}

impl ShrineFile {
    fn new(metadata: Metadata) -> Self {
        Self {
            metadata,
            ..Default::default()
        }
    }

    pub fn version(&self) -> u8 {
        self.metadata.version()
    }

    pub fn encryption_algorithm(&self) -> EncryptionAlgorithm {
        self.metadata.encryption_algorithm()
    }

    pub fn serialization_format(&self) -> SerializationFormat {
        self.metadata.serialization_format()
    }

    /// Serializes the `ShrineFile`.
    ///
    /// ```
    /// # use shrine::shrine_file::ShrineFile;
    /// let file = ShrineFile::default();
    /// assert_eq!(
    ///     file.as_bytes().unwrap().len() as u64,
    ///     13
    /// );
    /// ```
    pub fn as_bytes(&self) -> Result<Vec<u8>, FileFormatError> {
        let mut buffer = Vec::new();
        self.serialize(&mut buffer)
            .map_err(FileFormatError::Serialization)?;
        Ok(buffer)
    }

    /// Deserializes a slice of bytes into a `ShrineFile`.
    ///
    /// ```
    /// # use shrine::shrine_file::ShrineFile;
    /// # let bytes = ShrineFile::default().as_bytes().unwrap();
    /// # let bytes = bytes.as_slice();
    /// let file = ShrineFile::from_bytes(bytes).unwrap();
    /// assert_eq!(
    ///     file.as_bytes().unwrap().as_slice(),
    ///     bytes
    /// );
    /// ```
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, FileFormatError> {
        if &bytes[0..6] != "shrine".as_bytes() || bytes.len() < 7 {
            return Err(FileFormatError::InvalidFile);
        }

        if bytes[6] > VERSION {
            return Err(FileFormatError::UnsupportedVersion(bytes[6]));
        }

        Self::try_from_slice(bytes).map_err(FileFormatError::Deserialization)
    }

    /// Wraps a `Shrine` inside of a `ShrineFile`.
    pub fn wrap(&mut self, shrine: Shrine) -> Result<(), Error> {
        let bytes = match self
            .metadata
            .serialization_format()
            .serializer()
            .serialize(&shrine)
        {
            Ok(bytes) => bytes,
            Err(e) => return Err(Error::Write(e.to_string())),
        };

        let bytes = self
            .metadata
            .encryption_algorithm()
            .encryptor()
            .encrypt(&bytes)
            .map_err(|e| Error::Write(e.to_string()))?;

        self.payload = bytes;

        Ok(())
    }

    /// Unwraps the `Shrine` from the `ShrineFile`.
    pub fn unwrap(&self) -> Result<Shrine, Error> {
        let bytes = self
            .metadata
            .encryption_algorithm()
            .encryptor()
            .decrypt(&self.payload)
            .map_err(|e| Error::Read(e.to_string()))?;

        self.metadata
            .serialization_format()
            .serializer()
            .deserialize(&bytes)
            .map_err(|e| Error::Read(e.to_string()))
    }
}

/// Builds a default `ShrineFile`.
///
/// ```
/// # use crate::shrine::shrine_file::{EncryptionAlgorithm, SerializationFormat, ShrineFile};
/// let file = ShrineFile::default();
/// assert_eq!(file.version(), 0);
/// assert_eq!(file.encryption_algorithm(), EncryptionAlgorithm::Aes);
/// assert_eq!(file.serialization_format(), SerializationFormat::Bson);
///```
impl Default for ShrineFile {
    fn default() -> Self {
        Self {
            magic_number: [b's', b'h', b'r', b'i', b'n', b'e'],
            metadata: Metadata::default(),
            payload: Vec::default(),
        }
    }
}

#[derive(Debug, BorshSerialize, BorshDeserialize)]
enum Metadata {
    V0 {
        /// The algorithm used to encrypt the payload.
        encryption_algorithm: EncryptionAlgorithm,
        /// The serialization format used to serialize the payload.
        serialization_format: SerializationFormat,
    },
}

impl Metadata {
    fn version(&self) -> u8 {
        match self {
            Metadata::V0 { .. } => 0,
        }
    }

    fn encryption_algorithm(&self) -> EncryptionAlgorithm {
        match self {
            Metadata::V0 {
                encryption_algorithm,
                ..
            } => *encryption_algorithm,
        }
    }

    fn serialization_format(&self) -> SerializationFormat {
        match self {
            Metadata::V0 {
                serialization_format,
                ..
            } => *serialization_format,
        }
    }
}

impl Default for Metadata {
    fn default() -> Self {
        Self::V0 {
            encryption_algorithm: EncryptionAlgorithm::default(),
            serialization_format: SerializationFormat::default(),
        }
    }
}

/// The list of encryption algorithms used to encrypt the payload.
#[derive(Default, Debug, Clone, Copy, Eq, PartialEq, BorshSerialize, BorshDeserialize)]
pub enum EncryptionAlgorithm {
    /// AES-GCM-SIV encryption
    #[default]
    Aes,
    /// No encryption
    Plain,
}

impl EncryptionAlgorithm {
    fn encryptor(&self) -> Box<dyn EncDec> {
        match self {
            EncryptionAlgorithm::Aes => {
                // FIXME (#2): use the previous commit hash and repo remote as the AAD
                //  something similar to https://github.com/cpollet/shrine.git#ae9ef36cc813d90a47c13315158f8dc3f87ee81e
                Box::new(Aes::new(Secret::new("password".to_string()), None))
            }
            EncryptionAlgorithm::Plain => Box::new(Plain::new()),
        }
    }
}

/// The serialization format
#[derive(Default, Debug, Clone, Copy, Eq, PartialEq, BorshSerialize, BorshDeserialize)]
pub enum SerializationFormat {
    /// BSON, the data storage and network transfer format used by MongoDB.
    #[default]
    Bson,
    /// JSON, the ubiquitous JavaScript Object Notation used by many HTTP APIs.
    Json,
    /// MessagePack, an efficient binary format that resembles a compact JSON.
    MessagePage,
}

impl SerializationFormat {
    fn serializer(&self) -> Box<dyn SerDe<Shrine>> {
        match self {
            SerializationFormat::Bson => Box::new(BsonSerDe::new()),
            SerializationFormat::Json => Box::new(JsonSerDe::new()),
            SerializationFormat::MessagePage => Box::new(MessagePackSerDe::new()),
        }
    }
}

#[derive(Error, Debug)]
pub enum FileFormatError {
    #[error("could not serialize shrine file content: {0}")]
    Serialization(std::io::Error),
    #[error("could not deserialize shrine file: {0}")]
    Deserialization(std::io::Error),
    #[error("the provided file is not a valid shrine archive")]
    InvalidFile,
    #[error("the provided file version {0} is not supported (max {})", VERSION)]
    UnsupportedVersion(u8),
}

#[derive(Default)]
pub struct ShrineFileBuilder {
    encryption_algorithm: EncryptionAlgorithm,
    serialization_format: SerializationFormat,
}

impl ShrineFileBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_encryption_algorithm(mut self, encryption_algorithm: EncryptionAlgorithm) -> Self {
        self.encryption_algorithm = encryption_algorithm;
        self
    }

    pub fn with_serialization_format(mut self, serialization_format: SerializationFormat) -> Self {
        self.serialization_format = serialization_format;
        self
    }

    pub fn build(self) -> ShrineFile {
        ShrineFile::new(Metadata::V0 {
            encryption_algorithm: self.encryption_algorithm,
            serialization_format: self.serialization_format,
        })
    }
}

#[derive(Debug, Error)]
pub enum Error {
    Read(String),
    Write(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[cfg(test)]
mod tests {
    use crate::shrine::Shrine;
    use crate::shrine_file::{SerializationFormat, ShrineFile, ShrineFileBuilder, VERSION};

    #[test]
    fn invalid_magic_number() {
        let mut bytes = ShrineFile::default().as_bytes().unwrap();
        bytes[0] += 1;

        let file = ShrineFile::from_bytes(bytes.as_slice());

        assert!(file.is_err());
        assert_eq!(
            file.unwrap_err().to_string(),
            "the provided file is not a valid shrine archive"
        );
    }

    #[test]
    fn unsupported_version() {
        let mut bytes = ShrineFile::default().as_bytes().unwrap();
        bytes[6] = VERSION + 1;

        let file = ShrineFile::from_bytes(bytes.as_slice());

        assert!(file.is_err());
        assert_eq!(
            file.unwrap_err().to_string(),
            format!(
                "the provided file version {} is not supported (max {})",
                VERSION + 1,
                VERSION
            )
        );
    }

    #[test]
    fn wrap_unwrap() {
        let mut shrine = Shrine::new();
        shrine.set("key", "val");

        let mut shrine_file = ShrineFileBuilder::new()
            .with_serialization_format(SerializationFormat::Json)
            .build();
        shrine_file.wrap(shrine).expect("could not wrap shrine");

        let bytes = shrine_file
            .as_bytes()
            .expect("could not serialize shrine file");

        let shrine_file =
            ShrineFile::from_bytes(&bytes).expect("could not deserialize shrine file");

        let shrine = shrine_file.unwrap().expect("could not unwrap shrine");

        assert_eq!(
            "val".as_bytes(),
            shrine
                .get("key")
                .expect("key not found")
                .expose_secret_as_bytes()
        )
    }
}
