mod holder;

use crate::bytes::SecretBytes;
use crate::encrypt::aes::Aes;
use crate::encrypt::plain::Plain;
use crate::encrypt::EncDec;
use crate::serialize::bson::BsonSerDe;
use crate::serialize::json::JsonSerDe;
use crate::serialize::message_pack::MessagePackSerDe;
use crate::serialize::SerDe;
use crate::shrine::holder::Holder;
use crate::{Error, SHRINE_FILENAME};
use borsh::{BorshDeserialize, BorshSerialize};
use std::fmt::{Display, Formatter};
use std::fs::File;

use chrono::{DateTime, Utc};

use secrecy::{CloneableSecret, ExposeSecret, SerializableSecret, Zeroize};
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use uuid::Uuid;

/// Max supported file version
const VERSION: u8 = 0;

#[derive(Serialize, Deserialize, Clone)]
pub struct ShrinePassword(secrecy::Secret<SerializableSecretString>);

impl ShrinePassword {
    pub fn expose_secret(&self) -> &str {
        self.0.expose_secret().0.as_str()
    }
    pub fn expose_secret_as_bytes(&self) -> &[u8] {
        self.0.expose_secret().0.as_bytes()
    }
}

impl<S> From<S> for ShrinePassword
where
    S: Into<String>,
{
    fn from(value: S) -> Self {
        Self(secrecy::Secret::new(SerializableSecretString(value.into())))
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SerializableSecretString(String);
impl SerializableSecret for SerializableSecretString {}
impl CloneableSecret for SerializableSecretString {}
impl Zeroize for SerializableSecretString {
    fn zeroize(&mut self) {
        self.0.zeroize()
    }
}

pub type Secrets = Holder<Secret>;

#[derive(Debug, BorshSerialize, BorshDeserialize)]
pub struct Closed(Vec<u8>);

#[derive(Debug)]
pub struct Open(Secrets);

#[derive(Debug, BorshSerialize, BorshDeserialize)]
pub struct Shrine<Payload = Open> {
    /// Always "shrine".
    magic_number: [u8; 6],
    metadata: Metadata,
    /// The serialized then encrypted payload.
    payload: Payload,
}

impl Shrine {
    fn new(metadata: Metadata) -> Self {
        Self {
            metadata,
            ..Default::default()
        }
    }
}

impl<Payload> Shrine<Payload> {
    pub fn version(&self) -> u8 {
        self.metadata.version()
    }

    pub fn uuid(&self) -> Uuid {
        self.metadata.uuid()
    }

    pub fn encryption_algorithm(&self) -> EncryptionAlgorithm {
        self.metadata.encryption_algorithm()
    }

    pub fn serialization_format(&self) -> SerializationFormat {
        self.metadata.serialization_format()
    }

    pub fn requires_password(&self) -> bool {
        self.encryption_algorithm().requires_password()
    }
}

impl Shrine<Closed> {
    /// Write the shrine to a path.
    pub fn to_path<P>(&self, path: P) -> Result<(), Error>
    where
        P: AsRef<Path>,
    {
        let mut file = PathBuf::from(path.as_ref().as_os_str());
        file.push(SHRINE_FILENAME);

        let bytes = self.as_bytes()?;

        File::create(&file)
            .map_err(Error::IoWrite)?
            .write_all(&bytes)
            .map_err(Error::IoWrite)?;

        Ok(())
    }

    /// Serializes the `Shrine`.
    pub fn as_bytes(&self) -> Result<Vec<u8>, Error> {
        let mut buffer = Vec::new();
        self.serialize(&mut buffer).map_err(Error::IoWrite)?;
        Ok(buffer)
    }

    /// Read a shrine from a path.
    pub fn from_path<P>(path: P) -> Result<Self, Error>
    where
        P: AsRef<Path>,
    {
        let mut file = PathBuf::from(path.as_ref().as_os_str());
        file.push(SHRINE_FILENAME);

        if !Path::new(&file).exists() {
            return Err(Error::FileNotFound(path.as_ref().to_path_buf()));
        }

        let bytes = {
            let mut file = File::open(&file).map_err(Error::IoRead)?;
            let mut bytes = Vec::new();
            file.read_to_end(&mut bytes).map_err(Error::IoRead)?;
            bytes
        };

        Shrine::from_bytes(&bytes)
    }

    /// Deserializes a slice of bytes into a `Shrine`.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        if bytes.len() < 6 || &bytes[0..6] != "shrine".as_bytes() {
            return Err(Error::Read());
        }

        if bytes[6] > VERSION {
            return Err(Error::UnsupportedVersion(bytes[6]));
        }

        Self::try_from_slice(bytes).map_err(Error::IoRead)
    }

    /// Decrypt and deserialize the `Shrine`.
    ///
    /// ```
    /// # use secrecy::Secret;
    /// # use shrine::bytes::SecretBytes;
    /// # use shrine::shrine::{Mode, Shrine, ShrineBuilder, ShrinePassword};
    /// # let password = ShrinePassword::from("password");
    /// let mut shrine = ShrineBuilder::new().build();
    /// shrine.set("key", "val", Mode::Text).unwrap();
    ///
    /// let shrine = shrine.close(&password).unwrap();
    /// let shrine = shrine.open(&password).unwrap();
    ///
    /// assert_eq!(shrine.get("key").unwrap().value().expose_secret_as_bytes(), "val".as_bytes());
    pub fn open(self, password: &ShrinePassword) -> Result<Shrine<Open>, Error> {
        let bytes = self
            .metadata
            .encryption_algorithm()
            .encryptor(password, None)
            .decrypt(&self.payload.0)?;

        let holder = self
            .metadata
            .serialization_format()
            .serializer()
            .deserialize(&bytes)?;

        Ok(Shrine {
            magic_number: self.magic_number,
            metadata: self.metadata,
            payload: Open(holder),
        })
    }
}

impl Shrine<Open> {
    /// Move the content of the current shrine into `shrine`.
    ///
    /// ```
    /// # use secrecy::Secret;
    /// # use shrine::bytes::SecretBytes;
    /// # use shrine::shrine::{Mode, Shrine, ShrineBuilder};
    /// let mut src = ShrineBuilder::new().build();
    /// let mut dst = ShrineBuilder::new().build();
    ///
    /// src.set("key", "val", Mode::Text).unwrap();
    /// src.move_to(&mut dst);
    ///
    /// assert_eq!(dst.get("key").unwrap().value().expose_secret_as_bytes(), "val".as_bytes());
    pub fn move_to(self, shrine: &mut Shrine<Open>) {
        shrine.payload.0 = self.payload.0
    }

    /// Serialise and encrypt the shrine's content.
    ///
    /// ```
    /// # use secrecy::Secret;
    /// # use shrine::bytes::SecretBytes;
    /// # use shrine::shrine::{Mode, Shrine, ShrineBuilder, ShrinePassword};
    /// # let password = ShrinePassword::from("password");
    /// let mut shrine = ShrineBuilder::new().build();
    /// shrine.set("key", "val", Mode::Text).unwrap();
    ///
    /// let shrine = shrine.close(&password).unwrap();
    /// let shrine = shrine.open(&password).unwrap();
    ///
    /// assert_eq!(shrine.get("key").unwrap().value().expose_secret_as_bytes(), "val".as_bytes());
    pub fn close(self, password: &ShrinePassword) -> Result<Shrine<Closed>, Error> {
        let bytes = self
            .metadata
            .serialization_format()
            .serializer()
            .serialize(&self.payload.0)?;

        let bytes = self
            .metadata
            .encryption_algorithm()
            .encryptor(password, None)
            .encrypt(&bytes)?;

        Ok(Shrine {
            magic_number: self.magic_number,
            metadata: self.metadata,
            payload: Closed(bytes),
        })
    }

    /// Set a key/value pair.
    ///
    /// ```
    /// # use secrecy::Secret;
    /// # use shrine::bytes::SecretBytes;
    /// # use shrine::shrine::{Mode, Shrine, ShrineBuilder};
    /// let mut shrine = ShrineBuilder::new().build();
    ///
    /// shrine.set("key", "value", Mode::Text).unwrap();
    ///
    /// assert_eq!(shrine.get("key").unwrap().value().expose_secret_as_bytes(), "value".as_bytes());
    /// ```
    pub fn set<V>(&mut self, key: &str, value: V, mode: Mode) -> Result<(), Error>
    where
        V: Into<SecretBytes>,
    {
        match self.payload.0.get_mut(key) {
            Ok(secret) => {
                secret.with_data(value.into());
                Ok(())
            }
            Err(Error::KeyNotFound(_)) => self.payload.0.set(key, Secret::new(value.into(), mode)),
            Err(e) => Err(e),
        }
    }

    /// Get a previously set value by its key.
    /// ```
    /// # use secrecy::Secret;
    /// # use shrine::bytes::SecretBytes;
    /// # use shrine::shrine::{Mode, Shrine, ShrineBuilder};
    /// let mut shrine = ShrineBuilder::new().build();
    ///
    /// shrine.set("key", "value", Mode::Text).unwrap();
    ///
    /// assert_eq!(shrine.get("key").unwrap().value().expose_secret_as_bytes(), "value".as_bytes());
    /// assert!(shrine.get("unknown").is_err());
    /// ```
    pub fn get(&self, key: &str) -> Result<&Secret, Error> {
        self.payload.0.get(key)
    }

    /// Get the sorted list of all keys.
    /// ```
    /// # use secrecy::Secret;
    /// # use shrine::bytes::SecretBytes;
    /// # use shrine::shrine::{Mode, Shrine, ShrineBuilder};
    /// let mut shrine = ShrineBuilder::new().build();
    ///
    /// shrine.set("def", "val", Mode::Text).unwrap();
    /// shrine.set("abc", "val", Mode::Text).unwrap();
    ///
    /// assert_eq!(shrine.keys().len(), 2);
    /// assert_eq!(shrine.keys().get(0).unwrap(), "abc");
    /// assert_eq!(shrine.keys().get(1).unwrap(), "def");
    /// ```
    pub fn keys(&self) -> Vec<String> {
        self.payload.0.keys()
    }

    /// Remove a key from the shrine.
    ///
    /// ```
    /// # use secrecy::Secret;
    /// # use shrine::bytes::SecretBytes;
    /// # use shrine::shrine::{Mode, Shrine, ShrineBuilder};
    /// let mut shrine = ShrineBuilder::new().build();
    ///
    /// shrine.set("key", "value", Mode::Text).unwrap();
    /// shrine.remove("key");
    ///
    /// assert!(shrine.get("key").is_err());
    /// ```
    pub fn remove(&mut self, key: &str) -> bool {
        self.payload.0.remove(key)
    }

    /// Return the keys count.
    ///
    /// ```
    /// # use secrecy::Secret;
    /// # use shrine::bytes::SecretBytes;
    /// # use shrine::shrine::{Mode, Shrine, ShrineBuilder};
    /// let mut shrine = ShrineBuilder::new().build();
    ///
    /// assert_eq!(shrine.len(), 0);
    ///
    /// shrine.set("key", "val", Mode::Text).unwrap();
    ///
    /// assert_eq!(shrine.len(), 1);
    /// ```
    pub fn len(&self) -> u64 {
        self.payload.0.len()
    }

    /// Return whether the shrine is empty (i.e. contains key/value pairs) or not.
    ///
    /// ```
    /// # use secrecy::Secret;
    /// # use shrine::bytes::SecretBytes;
    /// # use shrine::shrine::{Mode, Shrine, ShrineBuilder};
    /// let mut shrine = ShrineBuilder::new().build();
    ///
    /// assert!(shrine.is_empty());
    ///
    /// shrine.set("key", "val", Mode::Text).unwrap();
    ///
    /// assert!(!shrine.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.payload.0.is_empty()
    }

    pub fn json(&self) -> String {
        serde_json::to_string(&self.payload.0).unwrap_or_default()
    }

    pub fn set_private<K, V>(&mut self, key: K, value: V)
    where
        K: Into<String>,
        V: Into<String>,
    {
        self.payload.0.set_private(key.into(), value.into());
    }

    pub fn get_private<'k, K>(&self, key: K) -> Option<&String>
    where
        K: Into<&'k str>,
    {
        self.payload.0.get_private(key.into())
    }

    pub fn remove_private<'k, K>(&mut self, key: K)
    where
        K: Into<&'k str>,
    {
        self.payload.0.remove_private(key.into());
    }

    pub fn keys_private(&self) -> Vec<String> {
        self.payload.0.keys_private()
    }
}

/// Builds a default `ShrineFile`.
///
/// ```
/// # use crate::shrine::shrine::{EncryptionAlgorithm, SerializationFormat, Shrine};
/// let file = Shrine::default();
/// assert_eq!(file.version(), 0);
/// assert_eq!(file.encryption_algorithm(), EncryptionAlgorithm::Aes);
/// assert_eq!(file.serialization_format(), SerializationFormat::Bson);
///```
impl Default for Shrine {
    fn default() -> Self {
        Self {
            magic_number: [b's', b'h', b'r', b'i', b'n', b'e'],
            metadata: Metadata::default(),
            payload: Open(Secrets::new()),
        }
    }
}

#[derive(Debug, BorshSerialize, BorshDeserialize)]
enum Metadata {
    V0 {
        uuid: u128,
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

    fn uuid(&self) -> Uuid {
        match self {
            Metadata::V0 { uuid, .. } => Uuid::from_u128(*uuid),
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
            uuid: Uuid::new_v4().as_u128(),
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
    fn requires_password(&self) -> bool {
        match self {
            EncryptionAlgorithm::Aes => true,
            EncryptionAlgorithm::Plain => false,
        }
    }

    fn encryptor<'pwd>(
        &self,
        password: &'pwd ShrinePassword,
        aad: Option<String>,
    ) -> Box<dyn EncDec + 'pwd> {
        match self {
            EncryptionAlgorithm::Aes => {
                // FIXME (#2): use the previous commit hash and repo remote as the AAD
                //  something similar to https://github.com/cpollet/shrine.git#ae9ef36cc813d90a47c13315158f8dc3f87ee81e
                Box::new(Aes::new(password, aad))
            }
            EncryptionAlgorithm::Plain => Box::new(Plain::new()),
        }
    }
}

impl Display for EncryptionAlgorithm {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            EncryptionAlgorithm::Aes => write!(f, "AES-GCM-SIV with 256-bits key"),
            EncryptionAlgorithm::Plain => write!(f, "Not encrypted"),
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
    MessagePack,
}

impl SerializationFormat {
    fn serializer(&self) -> Box<dyn SerDe<Secrets>> {
        match self {
            SerializationFormat::Bson => Box::new(BsonSerDe::new()),
            SerializationFormat::Json => Box::new(JsonSerDe::new()),
            SerializationFormat::MessagePack => Box::new(MessagePackSerDe::new()),
        }
    }
}

impl Display for SerializationFormat {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            SerializationFormat::Bson => write!(f, "BSON"),
            SerializationFormat::Json => write!(f, "JSON"),
            SerializationFormat::MessagePack => write!(f, "MessagePack"),
        }
    }
}

#[derive(Default)]
pub struct ShrineBuilder {
    encryption_algorithm: EncryptionAlgorithm,
    serialization_format: SerializationFormat,
}

impl ShrineBuilder {
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

    pub fn build(self) -> Shrine {
        Shrine::new(Metadata::V0 {
            uuid: Uuid::new_v4().as_u128(),
            encryption_algorithm: self.encryption_algorithm,
            serialization_format: self.serialization_format,
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Secret {
    value: SecretBytes,
    mode: Mode,
    created_by: String,
    created_at: DateTime<Utc>,
    updated_by: Option<String>,
    updated_at: Option<DateTime<Utc>>,
}

impl Secret {
    fn new(value: SecretBytes, mode: Mode) -> Self {
        Self {
            value,
            mode,
            created_by: format!("{}@{}", whoami::username(), whoami::hostname()),
            created_at: Utc::now(),
            updated_by: None,
            updated_at: None,
        }
    }

    pub fn mode(&self) -> Mode {
        self.mode
    }

    pub fn value(&self) -> &SecretBytes {
        &self.value
    }

    pub fn created_by(&self) -> &str {
        &self.created_by
    }

    pub fn created_at(&self) -> &DateTime<Utc> {
        &self.created_at
    }

    pub fn updated_by(&self) -> Option<&str> {
        match &self.updated_by {
            None => None,
            Some(s) => Some(s.as_ref()),
        }
    }

    pub fn updated_at(&self) -> Option<&DateTime<Utc>> {
        self.updated_at.as_ref()
    }
}

impl Secret {
    fn with_data(&mut self, data: SecretBytes) -> &mut Self {
        self.value = data;
        self.updated_by = Some(format!("{}@{}", whoami::username(), whoami::hostname()));
        self.updated_at = Some(Utc::now());
        self
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Mode {
    Binary,
    Text,
}

impl Display for Mode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Mode::Binary => write!(f, "bin"),
            Mode::Text => write!(f, "txt"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Key {
    pub key: String,
    pub mode: Mode,
    pub created_by: String,
    pub created_at: DateTime<Utc>,
    pub updated_by: Option<String>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl From<(String, &Secret)> for Key {
    fn from((key, secret): (String, &Secret)) -> Self {
        Self {
            key,
            mode: secret.mode(),
            created_by: secret.created_by().to_string(),
            created_at: *secret.created_at(),
            updated_by: secret.updated_by().map(|s| s.to_string()),
            updated_at: secret.updated_at().copied(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn invalid_magic_number() {
        let password = ShrinePassword::from("p");
        let mut bytes = ShrineBuilder::new()
            .build()
            .close(&password)
            .expect("could not close the shrine")
            .as_bytes()
            .expect("could not serialize the shrine");
        bytes[0] += 1;

        let shrine = Shrine::from_bytes(bytes.as_slice());

        assert!(shrine.is_err());
        assert_eq!(shrine.unwrap_err().to_string(), "Could not read shrine");
    }

    #[test]
    fn unsupported_version() {
        let password = ShrinePassword::from("p");
        let mut bytes = ShrineBuilder::new()
            .build()
            .close(&password)
            .expect("could not close the shrine")
            .as_bytes()
            .expect("could not serialize the shrine");
        bytes[6] = VERSION + 1;

        let shrine = Shrine::from_bytes(bytes.as_slice());

        assert!(shrine.is_err());
        assert_eq!(
            shrine.unwrap_err().to_string(),
            format!("Unsupported shrine version: {}", VERSION + 1)
        );
    }

    #[test]
    fn close_open() {
        let password = ShrinePassword::from("password");

        let mut shrine = ShrineBuilder::new().build();
        shrine.set("key", "val", Mode::Text).unwrap();

        let shrine = shrine.close(&password).expect("could not close shrine");
        let bytes = shrine.as_bytes().expect("could not serialize shrine file");

        let shrine = Shrine::from_bytes(&bytes).expect("could not deserialize shrine file");
        let shrine = shrine.open(&password).expect("could not open shrine");

        assert_eq!(
            "val".as_bytes(),
            shrine
                .get("key")
                .expect("key not found")
                .value()
                .expose_secret_as_bytes()
        )
    }

    #[test]
    fn to_path_from_path() {
        let temp_dir = tempdir().unwrap();
        let password = ShrinePassword::from("password");

        let mut shrine = Shrine::default();
        shrine.set("key", "value", Mode::Text).unwrap();
        let shrine = shrine.close(&password).unwrap();
        shrine.to_path(temp_dir.path()).unwrap();

        let shrine = Shrine::from_path(temp_dir.path()).unwrap();
        let shrine = shrine.open(&password).unwrap();

        assert_eq!(
            shrine.get("key").unwrap().value.expose_secret_as_bytes(),
            "value".as_bytes()
        );
    }

    #[test]
    fn move_content() {
        let mut shrine = Shrine::default();
        shrine.set("key", "value", Mode::Text).unwrap();

        let mut new_shrine = Shrine::default();
        shrine.move_to(&mut new_shrine);

        assert_eq!(
            new_shrine
                .get("key")
                .unwrap()
                .value
                .expose_secret_as_bytes(),
            "value".as_bytes()
        );
    }
}
