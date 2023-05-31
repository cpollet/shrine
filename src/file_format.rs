use borsh::{BorshDeserialize, BorshSerialize};
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
    pub fn version(&self) -> u8 {
        self.metadata.version()
    }

    pub fn encryption_algorithm(&self) -> EncryptionAlgorithm {
        self.metadata.encryption_algorithm()
    }

    pub fn serialization_format(&self) -> SerializationFormat {
        self.metadata.serialization_format()
    }

    pub fn payload(&self) -> &[u8] {
        self.payload.as_slice()
    }

    /// Serializes the `ShrineFile`.
    ///
    /// ```
    /// # use shrine::file_format::ShrineFile;
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
        if &bytes[0..6] != "shrine".as_bytes() || bytes.len() < 7 {
            return Err(FileFormatError::InvalidFile);
        }

        if bytes[6] > VERSION {
            return Err(FileFormatError::UnsupportedVersion(bytes[6]));
        }

        Self::try_from_slice(bytes).map_err(FileFormatError::Deserialization)
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
    /// No encryption
    #[default]
    Plain,
}

/// The serialization format
#[derive(Default, Debug, Clone, Copy, Eq, PartialEq, BorshSerialize, BorshDeserialize)]
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
    Serialization(std::io::Error),
    #[error("could not deserialize shrine file: {0}")]
    Deserialization(std::io::Error),
    #[error("the provided file is not a valid shrine archive")]
    InvalidFile,
    #[error("the provided file version {0} is not supported (max {})", VERSION)]
    UnsupportedVersion(u8),
}

#[cfg(test)]
mod tests {
    use crate::file_format::{ShrineFile, VERSION};

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
}
