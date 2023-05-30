use bincode::{DefaultOptions, Options};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Max supported file version
const VERSION: u8 = 0;

#[derive(Debug, Serialize, Deserialize)]
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

    /// Serializes the `ShrineFile`.
    ///
    /// ```
    /// # use bincode::Options;
    /// # use shrine::file_format::ShrineFile;
    /// let file = ShrineFile::default();
    /// assert_eq!(
    ///     file.as_bytes().unwrap().len() as u64,
    ///     bincode::DefaultOptions::new()
    ///         .with_big_endian()
    ///         .serialized_size(&file)
    ///         .unwrap()
    /// );
    /// ```
    pub fn as_bytes(&self) -> Result<Vec<u8>, FileFormatError> {
        DefaultOptions::new()
            .serialize(&self)
            .map_err(|e| FileFormatError::Serialization(e))
    }

    /// Deserializes a slice of bytes into a `ShrineFile`.
    ///
    /// ```
    /// # use shrine::file_format::ShrineFile;
    /// # let bytes = ShrineFile::default().as_bytes().unwrap();
    /// # let bytes = bytes.as_slice();
    /// let file = ShrineFile::from_bytes(bytes).unwrap();
    /// assert_eq!(
    ///     file.as_bytes().unwrap().as_slice(),
    ///     bytes
    /// );
    /// ```
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, FileFormatError> {
        let file = DefaultOptions::new()
            .deserialize::<Self>(bytes.into())
            .map_err(|e| FileFormatError::Deserialization(e))?;

        if file.magic_number.as_slice() != "shrine".as_bytes() {
            return Err(FileFormatError::InvalidFile);
        }

        if file.version > VERSION {
            return Err(FileFormatError::UnsupportedVersion(file.version));
        }

        Ok(file)
    }
}

/// Builds a default `ShrineFile`.
///
/// ```
/// # use crate::shrine::file_format::{EncryptionAlgorithm, SerializationFormat, ShrineFile};
/// let file = ShrineFile::default();
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
#[derive(Default, Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
pub enum EncryptionAlgorithm {
    /// No encryption
    #[default]
    Plain,
}

/// The serialization format
#[derive(Default, Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
pub enum SerializationFormat {
    /// BSON, the data storage and network transfer format used by MongoDB.
    #[default]
    Bson,
    /// JSON, the ubiquitous JavaScript Object Notation used by many HTTP APIs.
    Json,
}

#[derive(Error, Debug)]
pub enum FileFormatError {
    #[error("could not serialize shrine file content: {0}")]
    Serialization(bincode::Error),
    #[error("could not deserialize shrine file: {0}")]
    Deserialization(bincode::Error),
    #[error("the provided file is not a valid shrine archive")]
    InvalidFile,
    #[error("the provided file version {0} is not supported (max {})", VERSION)]
    UnsupportedVersion(u8),
}

#[cfg(test)]
mod tests {
    use crate::file_format::{FileFormatError, ShrineFile, VERSION};

    #[test]
    fn invalid_magic_number() {
        let mut bytes = ShrineFile::default().as_bytes().unwrap();
        bytes[0] += 1;

        let file = ShrineFile::from_bytes(bytes.as_slice());

        assert_eq!(file.is_err(), true);
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

        assert_eq!(file.is_err(), true);
        assert_eq!(
            file.unwrap_err().to_string(),
            format!(
                "the provided file version {} is not supported (max {})",
                VERSION + 1,
                VERSION
            )
        );
    }
}