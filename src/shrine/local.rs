use crate::shrine::encryption::EncryptionAlgorithm;
use crate::shrine::holder::Holder;
use crate::shrine::metadata::Metadata;
use crate::shrine::serialization::SerializationFormat;
use crate::shrine::{OpenShrine, VERSION};
use crate::values::bytes::SecretBytes;
use crate::values::password::ShrinePassword;
use crate::values::secret::{Mode, Secret};
use crate::Error;
use borsh::{BorshDeserialize, BorshSerialize};
use std::fmt::{Debug, Formatter};
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use uuid::Uuid;

pub type Secrets = Holder<Secret>;

pub struct Open {
    secrets: Secrets,
}

#[derive(Clone, BorshSerialize, BorshDeserialize)]
pub struct Closed(Vec<u8>);

impl Debug for Closed {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Closed(..)")
    }
}

#[derive(Clone, Debug)]
pub struct NoPassword;

#[derive(Debug)]
pub struct Aes<P = ShrinePassword> {
    password: P,
}

impl<P> Clone for Aes<P>
where
    P: Clone,
{
    fn clone(&self) -> Self {
        Self {
            password: self.password.clone(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Clear;

#[derive(Default)]
pub struct Unknown;

#[derive(Debug, Clone)]
pub struct Memory;

#[derive(Debug, BorshSerialize, BorshDeserialize)]
pub struct LocalShrine<S = Open, E = Aes<ShrinePassword>, L = PathBuf> {
    /// Always "shrine".
    magic_number: [u8; 6],
    metadata: Metadata,
    payload: S,
    #[borsh(skip)]
    encryption: E,
    #[borsh(skip)]
    location: L,
}

impl LocalShrine<Open, Aes<NoPassword>, Memory> {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Default for LocalShrine<Open, Aes<NoPassword>, Memory> {
    fn default() -> Self {
        Self {
            magic_number: [b's', b'h', b'r', b'i', b'n', b'e'],
            metadata: Metadata::V0 {
                uuid: Uuid::new_v4().as_u128(),
                encryption_algorithm: EncryptionAlgorithm::Aes,
                serialization_format: Default::default(),
            },
            payload: Open {
                secrets: Holder::new(),
            },
            encryption: Aes {
                password: NoPassword,
            },
            location: Memory,
        }
    }
}

impl<S, E> LocalShrine<S, E, Memory> {
    pub fn with_path(self, path: PathBuf) -> LocalShrine<S, E, PathBuf> {
        LocalShrine {
            magic_number: self.magic_number,
            metadata: self.metadata,
            payload: self.payload,
            encryption: self.encryption,
            location: path,
        }
    }
}

impl<S, E, L> LocalShrine<S, E, L> {
    pub fn uuid(&self) -> Uuid {
        self.metadata.uuid()
    }

    pub fn version(&self) -> u8 {
        self.metadata.version()
    }

    pub fn serialization_format(&self) -> SerializationFormat {
        self.metadata.serialization_format()
    }

    pub fn encryption_algorithm(&self) -> EncryptionAlgorithm {
        self.metadata.encryption_algorithm()
    }
}

impl<E, L> LocalShrine<Closed, E, L> {
    fn try_to_bytes(&self) -> Result<Vec<u8>, Error> {
        let mut buffer = Vec::new();
        self.write(&mut buffer)?;
        Ok(buffer)
    }

    fn write<W>(&self, writer: &mut W) -> Result<(), Error>
    where
        W: Write,
    {
        self.serialize(writer).map_err(Error::IoWrite)
    }

    pub fn write_to<P>(&self, path: P) -> Result<(), Error>
    where
        P: AsRef<Path>,
    {
        let file = PathBuf::from(path.as_ref().as_os_str());

        let bytes = self.try_to_bytes()?;

        File::create(file)
            .map_err(Error::IoWrite)?
            .write_all(&bytes)
            .map_err(Error::IoWrite)?;

        Ok(())
    }
}

impl<E, L> Clone for LocalShrine<Closed, E, L>
    where
        E: Clone,
        L: Clone,
{
    fn clone(&self) -> Self {
        Self {
            magic_number: self.magic_number,
            metadata: self.metadata.clone(),
            payload: self.payload.clone(),
            encryption: self.encryption.clone(),
            location: self.location.clone(),
        }
    }
}

impl<E> LocalShrine<Closed, E, PathBuf> {
    pub fn write_file(&self) -> Result<(), Error> {
        self.write_to(&self.location)
    }
}

impl<L> LocalShrine<Closed, Clear, L> {
    pub fn open(self) -> Result<LocalShrine<Open, Clear, L>, Error> {
        let secrets = self
            .metadata
            .serialization_format()
            .serializer()
            .deserialize(&self.payload.0)?;

        Ok(LocalShrine {
            magic_number: self.magic_number,
            metadata: self.metadata,
            payload: Open { secrets },
            encryption: Clear,
            location: self.location,
        })
    }
}

impl<L> LocalShrine<Closed, Aes<NoPassword>, L> {
    pub fn open(
        self,
        password: ShrinePassword,
    ) -> Result<LocalShrine<Open, Aes<ShrinePassword>, L>, Error> {
        let clear_bytes = self
            .metadata
            .encryption_algorithm()
            .encryptor(&password, None)
            .decrypt(&self.payload.0)?;

        let secrets = self
            .metadata
            .serialization_format()
            .serializer()
            .deserialize(&clear_bytes)?;

        Ok(LocalShrine {
            magic_number: self.magic_number,
            metadata: self.metadata,
            payload: Open { secrets },
            encryption: Aes { password },
            location: self.location,
        })
    }
}

impl<E, L> LocalShrine<Open, E, L> {
    pub fn with_serialization_format(&mut self, format: SerializationFormat) {
        self.metadata = match self.metadata {
            Metadata::V0 {
                uuid,
                encryption_algorithm,
                ..
            } => Metadata::V0 {
                uuid,
                encryption_algorithm,
                serialization_format: format,
            },
        };
    }

    pub fn set(&mut self, key: &str, value: SecretBytes, mode: Mode) -> Result<(), Error> {
        if let Some(key) = key.strip_prefix('.') {
            return self
                .payload
                .secrets
                .set_private(key, Secret::new(value, mode));
        }

        match self.payload.secrets.get_mut(key) {
            Ok(secret) => {
                secret.update_with(value, mode);
                Ok(())
            }
            Err(Error::KeyNotFound(_)) => self.payload.secrets.set(key, Secret::new(value, mode)),
            Err(e) => Err(e),
        }
    }

    pub fn get(&self, key: &str) -> Result<&Secret, Error> {
        if let Some(key) = key.strip_prefix('.') {
            return self.payload.secrets.get_private(key);
        }
        self.payload.secrets.get(key)
    }

    pub fn rm(&mut self, key: &str) -> bool {
        self.payload.secrets.remove(key)
    }

    pub fn mv<T>(self, other: &mut OpenShrine<T>) {
        match other {
            OpenShrine::LocalClear(s) => s.payload = self.payload,
            OpenShrine::LocalAes(s) => s.payload = self.payload,
            OpenShrine::Remote(_) => {
                unimplemented!("Moving a local shrine to remote one is not supported")
            }
        }
    }

    pub fn keys(&self) -> Vec<String> {
        self.payload.secrets.keys()
    }

    pub fn keys_private(&self) -> Vec<String> {
        self.payload.secrets.keys_private()
    }
}

impl<T, L> LocalShrine<Open, Aes<T>, L> {
    pub fn into_clear(self) -> LocalShrine<Open, Clear, L> {
        LocalShrine {
            magic_number: self.magic_number,
            metadata: match self.metadata {
                Metadata::V0 {
                    uuid,
                    serialization_format,
                    ..
                } => Metadata::V0 {
                    uuid,
                    encryption_algorithm: EncryptionAlgorithm::Plain,
                    serialization_format,
                },
            },
            payload: self.payload,
            encryption: Clear,
            location: self.location,
        }
    }

    pub fn set_password(
        self,
        password: ShrinePassword,
    ) -> LocalShrine<Open, Aes<ShrinePassword>, L> {
        LocalShrine {
            magic_number: self.magic_number,
            metadata: self.metadata,
            payload: self.payload,
            encryption: Aes { password },
            location: self.location,
        }
    }
}

impl<L> LocalShrine<Open, Aes<NoPassword>, L> {
    pub fn close(
        self,
        password: ShrinePassword,
    ) -> Result<LocalShrine<Closed, Aes<NoPassword>, L>, Error> {
        self.set_password(password).close()
    }
}

impl<L> LocalShrine<Open, Aes<ShrinePassword>, L> {
    pub fn close(self) -> Result<LocalShrine<Closed, Aes<NoPassword>, L>, Error> {
        let clear_bytes = self
            .metadata
            .serialization_format()
            .serializer()
            .serialize(&self.payload.secrets)?;

        let cipher_bytes = self
            .metadata
            .encryption_algorithm()
            .encryptor(&self.encryption.password, None)
            .encrypt(&clear_bytes)?;

        Ok(LocalShrine {
            magic_number: self.magic_number,
            metadata: self.metadata,
            payload: Closed(cipher_bytes),
            encryption: Aes {
                password: NoPassword,
            },
            location: self.location,
        })
    }
}

impl<L> LocalShrine<Open, Clear, L> {
    pub fn into_aes(self) -> LocalShrine<Open, Aes<NoPassword>, L> {
        LocalShrine {
            magic_number: self.magic_number,
            metadata: match self.metadata {
                Metadata::V0 {
                    uuid,
                    serialization_format,
                    ..
                } => Metadata::V0 {
                    uuid,
                    encryption_algorithm: EncryptionAlgorithm::Aes,
                    serialization_format,
                },
            },
            payload: self.payload,
            encryption: Aes {
                password: NoPassword,
            },
            location: self.location,
        }
    }

    pub fn into_aes_with_password(
        self,
        password: ShrinePassword,
    ) -> LocalShrine<Open, Aes<ShrinePassword>, L> {
        let shrine = self.into_aes();
        LocalShrine {
            magic_number: shrine.magic_number,
            metadata: shrine.metadata,
            payload: shrine.payload,
            encryption: Aes { password },
            location: shrine.location,
        }
    }

    pub fn close(self) -> Result<LocalShrine<Closed, Clear, L>, Error> {
        let bytes = self
            .metadata
            .serialization_format()
            .serializer()
            .serialize(&self.payload.secrets)?;

        Ok(LocalShrine {
            magic_number: self.magic_number,
            metadata: self.metadata,
            payload: Closed(bytes),
            encryption: Clear,
            location: self.location,
        })
    }
}

#[derive(Debug)]
pub enum LoadedShrine {
    Clear(LocalShrine<Closed, Clear, PathBuf>),
    Aes(LocalShrine<Closed, Aes<NoPassword>, PathBuf>),
}

impl LoadedShrine {
    /// Read a shrine from a path.
    pub fn try_from_path<P>(path: P) -> Result<Self, Error>
    where
        P: AsRef<Path>,
    {
        if !path.as_ref().exists() {
            return Err(Error::FileNotFound(path.as_ref().to_path_buf()));
        }

        let bytes = {
            let mut file = File::open(&path).map_err(Error::IoRead)?;
            let mut bytes = Vec::new();
            file.read_to_end(&mut bytes).map_err(Error::IoRead)?;
            bytes
        };

        let shrine = InMemoryShrine::try_from_bytes(&bytes)?;

        Ok(match shrine {
            InMemoryShrine::Clear(s) => LoadedShrine::Clear(LocalShrine {
                magic_number: s.magic_number,
                metadata: s.metadata,
                payload: s.payload,
                encryption: s.encryption,
                location: path.as_ref().to_path_buf(),
            }),
            InMemoryShrine::Aes(s) => LoadedShrine::Aes(LocalShrine {
                magic_number: s.magic_number,
                metadata: s.metadata,
                payload: s.payload,
                encryption: s.encryption,
                location: path.as_ref().to_path_buf(),
            }),
        })
    }
}

#[derive(Debug)]
enum InMemoryShrine {
    Clear(LocalShrine<Closed, Clear, Memory>),
    Aes(LocalShrine<Closed, Aes<NoPassword>, Memory>),
}

impl InMemoryShrine {
    /// Read a shrine from a byte slice.
    fn try_from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        if bytes.len() < 6 || &bytes[0..6] != "shrine".as_bytes() {
            return Err(Error::Read());
        }

        if bytes[6] > VERSION {
            return Err(Error::UnsupportedVersion(bytes[6]));
        }

        let shrine =
            LocalShrine::<Closed, Unknown>::try_from_slice(bytes).map_err(Error::IoRead)?;

        Ok(match shrine.metadata {
            Metadata::V0 {
                encryption_algorithm,
                ..
            } => match encryption_algorithm {
                EncryptionAlgorithm::Aes => InMemoryShrine::Aes(LocalShrine {
                    magic_number: shrine.magic_number,
                    metadata: shrine.metadata,
                    payload: shrine.payload,
                    encryption: Aes {
                        password: NoPassword,
                    },
                    location: Memory,
                }),
                EncryptionAlgorithm::Plain => InMemoryShrine::Clear(LocalShrine {
                    magic_number: shrine.magic_number,
                    metadata: shrine.metadata,
                    payload: shrine.payload,
                    encryption: Clear,
                    location: Memory,
                }),
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shrine::VERSION;
    use tempfile::tempdir;

    #[test]
    fn local_shrine_uuid() {
        let shrine = LocalShrine::new();
        let uuid = (&shrine.metadata).uuid();
        assert_eq!(shrine.uuid().as_u128(), uuid.as_u128());
    }

    #[test]
    fn local_shrine_version() {
        let shrine = LocalShrine::new();
        assert_eq!(shrine.version(), VERSION);
    }

    #[test]
    fn local_shrine_serialization_format() {
        let shrine = LocalShrine::new();
        assert_eq!(shrine.serialization_format(), SerializationFormat::Bson);
    }

    #[test]
    fn local_shrine_encryption_format() {
        let shrine = LocalShrine::new();
        assert_eq!(shrine.encryption_algorithm(), EncryptionAlgorithm::Aes);

        let shrine = LocalShrine::new().into_clear();
        assert_eq!(shrine.encryption_algorithm(), EncryptionAlgorithm::Plain);
    }

    #[test]
    fn loaded_shrine_uuid() {
        let shrine = LocalShrine::new();
        let uuid = (&shrine.metadata).uuid();
        assert_eq!(shrine.uuid().as_u128(), uuid.as_u128());
    }

    #[test]
    fn loaded_shrine_version() {
        let shrine = LocalShrine::new();
        assert_eq!(shrine.version(), VERSION);
    }

    #[test]
    fn loaded_shrine_serialization_format() {
        let shrine = LocalShrine::new();
        assert_eq!(shrine.serialization_format(), SerializationFormat::Bson);
    }

    #[test]
    fn loaded_shrine_encryption_format() {
        let shrine = LocalShrine::new();
        assert_eq!(shrine.encryption_algorithm(), EncryptionAlgorithm::Aes);

        let shrine = LocalShrine::new().into_clear();
        assert_eq!(shrine.encryption_algorithm(), EncryptionAlgorithm::Plain);
    }

    #[test]
    fn set_get() {
        let mut shrine = LocalShrine::new();

        shrine
            .set("key", SecretBytes::from("value".as_bytes()), Mode::Text)
            .unwrap();
        let secret = shrine.get("key").unwrap();
        assert_eq!(secret.value().expose_secret_as_bytes(), "value".as_bytes());
        assert_eq!(secret.mode(), Mode::Text);

        shrine
            .set("key", SecretBytes::from("bin".as_bytes()), Mode::Binary)
            .unwrap();
        let secret = shrine.get("key").unwrap();
        assert_eq!(secret.value().expose_secret_as_bytes(), "bin".as_bytes());
        assert_eq!(secret.mode(), Mode::Binary);
    }

    #[test]
    fn set_get_private() {
        let mut shrine = LocalShrine::new();

        shrine
            .set(".key", SecretBytes::from("value".as_bytes()), Mode::Text)
            .unwrap();
        let secret = shrine.get(".key").unwrap();
        assert_eq!(secret.value().expose_secret_as_bytes(), "value".as_bytes());
        assert_eq!(secret.mode(), Mode::Text);

        shrine
            .set(".key", SecretBytes::from("bin".as_bytes()), Mode::Binary)
            .unwrap();
        let secret = shrine.get(".key").unwrap();
        assert_eq!(secret.value().expose_secret_as_bytes(), "bin".as_bytes());
        assert_eq!(secret.mode(), Mode::Binary);
    }

    #[test]
    fn rm() {
        let mut shrine = LocalShrine::new();

        shrine
            .set("key", SecretBytes::from("value".as_bytes()), Mode::Text)
            .unwrap();
        assert!(shrine.rm("key"));

        let err = shrine.get("key").unwrap_err();
        match err {
            Error::KeyNotFound(k) => {
                assert_eq!(&k, "key")
            }
            e => panic!("Expected Error::KeyNotFound(\"key\"), got {:?}", e),
        }

        assert!(!shrine.rm("key"));
    }

    #[test]
    fn mv() {
        let mut src = LocalShrine::new();
        src.set("key", SecretBytes::from("value".as_bytes()), Mode::Text)
            .unwrap();

        let mut dst = OpenShrine::LocalClear(LocalShrine::new().into_clear());
        src.mv(&mut dst);

        let secret = dst.get("key").unwrap();
        assert_eq!(secret.value().expose_secret_as_bytes(), "value".as_bytes());
        assert_eq!(secret.mode(), Mode::Text);
    }

    #[test]
    fn keys() {
        let mut shrine = LocalShrine::new();

        shrine
            .set("key", SecretBytes::from("value".as_bytes()), Mode::Text)
            .unwrap();

        let keys = shrine.keys();
        assert_eq!(keys.len(), 1);
        assert_eq!(keys.get(0), Some(&"key".to_string()))
    }

    #[test]
    fn keys_private() {
        let mut shrine = LocalShrine::new();

        shrine
            .set(".key", SecretBytes::from("value".as_bytes()), Mode::Text)
            .unwrap();

        let keys = shrine.keys_private();
        assert_eq!(keys.len(), 1);
        assert_eq!(keys.get(0), Some(&"key".to_string()))
    }

    #[test]
    fn clear_close_open() {
        let mut shrine = LocalShrine::new();

        shrine
            .set("key", SecretBytes::from("value".as_bytes()), Mode::Text)
            .unwrap();

        let shrine = shrine.into_clear();

        let shrine = shrine.close().unwrap();

        let shrine = shrine.open().unwrap();

        assert_eq!(
            shrine.get("key").unwrap().value().expose_secret_as_bytes(),
            "value".as_bytes()
        );
    }

    #[test]
    fn aes_close_open() {
        let mut shrine = LocalShrine::new();

        shrine
            .set("key", SecretBytes::from("value".as_bytes()), Mode::Text)
            .unwrap();

        let shrine = shrine.close(ShrinePassword::from("password")).unwrap();

        let shrine = shrine.open(ShrinePassword::from("password")).unwrap();

        assert_eq!(
            shrine.get("key").unwrap().value().expose_secret_as_bytes(),
            "value".as_bytes()
        );
    }

    #[test]
    fn aes_close_open_wrong_password() {
        let mut shrine = LocalShrine::new();

        shrine
            .set("key", SecretBytes::from("value".as_bytes()), Mode::Text)
            .unwrap();

        let shrine = shrine.set_password(ShrinePassword::from("password"));

        let shrine = shrine.close().unwrap();

        match shrine.open(ShrinePassword::from("wrong")) {
            Err(Error::CryptoRead) => (),
            _ => panic!("Expected Err(Error::CryptoRead)"),
        }
    }

    #[test]
    fn clear_try_to_bytes_try_from_bytes() {
        let mut shrine = LocalShrine::new();

        shrine
            .set("key", SecretBytes::from("value".as_bytes()), Mode::Text)
            .unwrap();

        let shrine = shrine.into_clear().close().unwrap();

        let bytes = shrine.try_to_bytes().unwrap();

        let shrine = match InMemoryShrine::try_from_bytes(&bytes).unwrap() {
            InMemoryShrine::Clear(s) => s.open().unwrap(),
            _ => panic!("Expected clear shrine"),
        };

        assert_eq!(
            shrine.get("key").unwrap().value().expose_secret_as_bytes(),
            "value".as_bytes()
        );
    }

    #[test]
    fn aes_try_to_bytes_try_from_bytes() {
        let mut shrine = LocalShrine::new();

        shrine
            .set("key", SecretBytes::from("value".as_bytes()), Mode::Text)
            .unwrap();

        let shrine = shrine
            .into_clear()
            .into_aes_with_password(ShrinePassword::from("password"))
            .close()
            .unwrap();

        let bytes = shrine.try_to_bytes().unwrap();

        let shrine = match InMemoryShrine::try_from_bytes(&bytes).unwrap() {
            InMemoryShrine::Aes(s) => s.open(ShrinePassword::from("password")).unwrap(),
            _ => panic!("Expected aes shrine"),
        };

        assert_eq!(
            shrine.get("key").unwrap().value().expose_secret_as_bytes(),
            "value".as_bytes()
        );
    }

    #[test]
    fn write_file_try_from_path() {
        let folder = tempdir().unwrap();
        let mut path = folder.into_path();
        path.push("shrine");

        let mut shrine = LocalShrine::new();
        shrine
            .set("key", SecretBytes::from("value".as_bytes()), Mode::Text)
            .unwrap();
        let shrine = shrine.close(ShrinePassword::from("password")).unwrap();
        shrine.write_to(&path).unwrap();

        let shrine = LoadedShrine::try_from_path(&path).unwrap();

        let shrine = match shrine {
            LoadedShrine::Clear(_) => panic!("AES shrine expected"),
            LoadedShrine::Aes(s) => s.open(ShrinePassword::from("password")).unwrap(),
        };

        assert_eq!(
            shrine.get("key").unwrap().value().expose_secret_as_bytes(),
            "value".as_bytes()
        );
    }

    #[test]
    fn invalid_magic_number() {
        let mut bytes = LocalShrine::new()
            .into_clear()
            .close()
            .unwrap()
            .try_to_bytes()
            .unwrap();
        bytes[0] += 1;

        match InMemoryShrine::try_from_bytes(&bytes).unwrap_err() {
            Error::Read() => {}
            e => panic!("expected Error::Read, got {:?}", e),
        }
    }

    #[test]
    fn unsupported_version() {
        let mut bytes = LocalShrine::new()
            .into_clear()
            .close()
            .unwrap()
            .try_to_bytes()
            .unwrap();
        bytes[6] += 1;

        match InMemoryShrine::try_from_bytes(&bytes).unwrap_err() {
            Error::UnsupportedVersion(v) => {
                assert_eq!(v, 1)
            }
            e => panic!("expected Error::Read, got {:?}", e),
        }
    }
}
