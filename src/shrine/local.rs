use crate::format::Format;
use crate::shrine::encryption::EncryptionAlgorithm;
use crate::shrine::holder::Holder;
use crate::shrine::serialization::SerializationFormat;
use crate::shrine::OpenShrine;
use crate::values::bytes::SecretBytes;
use crate::values::password::ShrinePassword;
use crate::values::secret::{Mode, Secret};
use crate::{format, Error};
use secrecy::zeroize::Zeroizing;
use std::fmt::{Debug, Formatter};
use std::fs::File;
use std::io::{Read, Write};
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use uuid::Uuid;

pub type Secrets = Holder<Secret>;

pub struct Open {
    secrets: Secrets,
}

#[derive(Clone)]
pub struct Closed(Vec<u8>);

impl Closed {
    pub fn new(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }
}

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

impl Aes {
    pub fn no_password() -> Aes<NoPassword> {
        Aes {
            password: NoPassword,
        }
    }
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

#[derive(Debug)]
pub struct LocalShrine<S = Open, E = Aes<ShrinePassword>, L = PathBuf> {
    uuid: Uuid,
    payload: S,
    encryption: E,
    format: Arc<Mutex<dyn Format>>,
    location: L,
}

impl Default for LocalShrine<Open, Aes<NoPassword>, Memory> {
    fn default() -> Self {
        Self {
            uuid: Uuid::new_v4(),
            payload: Open {
                secrets: Holder::new(),
            },
            encryption: Aes {
                password: NoPassword,
            },
            format: format::default(),
            location: Memory,
        }
    }
}

impl<E> LocalShrine<Closed, E, Memory> {
    pub fn new_closed(
        uuid: Uuid,
        payload: Closed,
        encryption: E,
        format: Arc<Mutex<dyn Format>>,
    ) -> Self {
        Self {
            uuid,
            payload,
            encryption,
            format,
            location: Memory,
        }
    }
}

impl<S, E, L> LocalShrine<S, E, L> {
    pub fn with_path(self, path: PathBuf) -> LocalShrine<S, E, PathBuf> {
        LocalShrine {
            uuid: self.uuid,
            payload: self.payload,
            encryption: self.encryption,
            format: self.format,
            location: path,
        }
    }

    pub fn uuid(&self) -> Uuid {
        self.uuid
    }

    pub fn version(&self) -> u8 {
        self.format.lock().unwrap().version()
    }

    pub fn serialization_format(&self) -> SerializationFormat {
        self.format.lock().unwrap().serialization_format()
    }

    fn is_readonly_format(&self) -> bool {
        self.format.lock().unwrap().is_readonly()
    }
}

impl<E, L> LocalShrine<Closed, E, L> {
    // todo: having to provide the EncryptionAlgorithm is not amazing...
    fn to_bytes(&self, encryption_algorithm: EncryptionAlgorithm) -> Vec<u8> {
        self.format
            .lock()
            .unwrap()
            .serialize(self.uuid, encryption_algorithm, &self.payload.0)
    }

    pub fn write_to<P>(
        &self,
        path: P,
        encryption_algorithm: EncryptionAlgorithm,
    ) -> Result<(), Error>
    where
        P: AsRef<Path>,
    {
        let file = PathBuf::from(path.as_ref().as_os_str());

        let bytes = self.to_bytes(encryption_algorithm);

        File::create(file)
            .map_err(Error::IoWrite)?
            .write_all(&bytes)
            .map_err(Error::IoWrite)?;

        Ok(())
    }
}

// #[cfg(test)]
// impl<E, L> Clone for LocalShrine<Closed, E, L>
// where
//     E: Clone,
//     L: Clone,
// {
//     fn clone(&self) -> Self {
//         Self {
//             uuid: self.uuid,
//             payload: self.payload.clone(),
//             encryption: self.encryption.clone(),
//             format: self.format.clone(),
//             location: self.location.clone(),
//         }
//     }
// }

impl LocalShrine<Closed, Clear, PathBuf> {
    pub fn write_file(&self) -> Result<(), Error> {
        self.write_to(&self.location, self.encryption_algorithm())
    }
}

impl<T> LocalShrine<Closed, Aes<T>, PathBuf> {
    pub fn write_file(&self) -> Result<(), Error> {
        self.write_to(&self.location, self.encryption_algorithm())
    }
}

impl<S, E> LocalShrine<S, E, PathBuf> {
    pub fn path(&self) -> &Path {
        &self.location
    }
}

impl<S, T, L> LocalShrine<S, Aes<T>, L> {
    pub fn encryption_algorithm(&self) -> EncryptionAlgorithm {
        EncryptionAlgorithm::Aes
    }
}

impl<S, L> LocalShrine<S, Clear, L> {
    pub fn encryption_algorithm(&self) -> EncryptionAlgorithm {
        EncryptionAlgorithm::Plain
    }
}

impl<L> LocalShrine<Closed, Clear, L> {
    pub fn open(self) -> Result<LocalShrine<Open, Clear, L>, Error> {
        let secrets = self
            .format
            .lock()
            .unwrap()
            .deserialize_secret(Zeroizing::new(self.payload.0))?;

        Ok(LocalShrine {
            uuid: self.uuid,
            payload: Open { secrets },
            encryption: Clear,
            format: self.format,
            location: self.location,
        })
    }
}

impl<L> LocalShrine<Closed, Aes<NoPassword>, L> {
    pub fn open(
        self,
        password: ShrinePassword,
    ) -> Result<LocalShrine<Open, Aes<ShrinePassword>, L>, Error> {
        let clear_bytes = Zeroizing::new(
            self.encryption_algorithm()
                .encryptor(&password, None)
                .decrypt(&self.payload.0)?,
        );

        let secrets = self
            .format
            .lock()
            .unwrap()
            .deserialize_secret(clear_bytes)?;

        Ok(LocalShrine {
            uuid: self.uuid,
            payload: Open { secrets },
            encryption: Aes { password },
            format: self.format,
            location: self.location,
        })
    }
}

impl<E, L> LocalShrine<Open, E, L> {
    pub fn with_serialization_format(&mut self, format: SerializationFormat) {
        self.format.lock().unwrap().set_serialization_format(format);
    }

    pub fn set(&mut self, key: &str, value: SecretBytes, mode: Mode) -> Result<(), Error> {
        if self.is_readonly_format() {
            return Err(Error::UnsupportedOldFormat(self.version()));
        }

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

    pub fn rm(&mut self, key: &str) -> Result<bool, Error> {
        if self.is_readonly_format() {
            return Err(Error::UnsupportedOldFormat(self.version()));
        }

        Ok(self.payload.secrets.remove(key))
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
            uuid: self.uuid,
            payload: self.payload,
            encryption: Clear,
            format: self.format,
            location: self.location,
        }
    }

    pub fn set_password(
        self,
        password: ShrinePassword,
    ) -> LocalShrine<Open, Aes<ShrinePassword>, L> {
        LocalShrine {
            uuid: self.uuid,
            payload: self.payload,
            encryption: Aes { password },
            format: self.format,
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
            .format
            .lock()
            .unwrap()
            .serialize_secrets(&self.payload.secrets)?;

        let cipher_bytes = self
            .encryption_algorithm()
            .encryptor(&self.encryption.password, None)
            .encrypt(clear_bytes.as_slice())?;

        Ok(LocalShrine {
            uuid: self.uuid,
            payload: Closed(cipher_bytes),
            encryption: Aes {
                password: NoPassword,
            },
            format: self.format,
            location: self.location,
        })
    }
}

impl<L> LocalShrine<Open, Clear, L> {
    pub fn into_aes(self) -> LocalShrine<Open, Aes<NoPassword>, L> {
        LocalShrine {
            uuid: self.uuid,
            payload: self.payload,
            encryption: Aes {
                password: NoPassword,
            },
            format: self.format,
            location: self.location,
        }
    }

    pub fn into_aes_with_password(
        self,
        password: ShrinePassword,
    ) -> LocalShrine<Open, Aes<ShrinePassword>, L> {
        let shrine = self.into_aes();
        LocalShrine {
            uuid: shrine.uuid,
            payload: shrine.payload,
            encryption: Aes { password },
            format: shrine.format,
            location: shrine.location,
        }
    }

    pub fn close(self) -> Result<LocalShrine<Closed, Clear, L>, Error> {
        let bytes = self
            .format
            .lock()
            .unwrap()
            .serialize_secrets(&self.payload.secrets)?;

        Ok(LocalShrine {
            uuid: self.uuid,
            payload: Closed(bytes.deref().clone()),
            encryption: Clear,
            format: self.format,
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
            let mut bytes = Vec::default();
            file.read_to_end(&mut bytes).map_err(Error::IoRead)?;
            bytes
        };

        let shrine = InMemoryShrine::try_from_bytes(&bytes)?;
        let path = path.as_ref().to_path_buf();
        match shrine {
            InMemoryShrine::Clear(s) => Ok(LoadedShrine::Clear(s.with_path(path))),
            InMemoryShrine::Aes(s) => Ok(LoadedShrine::Aes(s.with_path(path))),
        }
    }
}

#[derive(Debug)]
pub enum InMemoryShrine {
    Clear(LocalShrine<Closed, Clear, Memory>),
    Aes(LocalShrine<Closed, Aes<NoPassword>, Memory>),
}

impl InMemoryShrine {
    pub fn try_from_bytes(bytes: &[u8]) -> Result<InMemoryShrine, Error> {
        format::read(bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn local_shrine_version() {
        let shrine = LocalShrine::default();
        assert_eq!(
            shrine.version(),
            format::default().lock().unwrap().version()
        );
    }

    #[test]
    fn local_shrine_serialization_format() {
        let shrine = LocalShrine::default();
        assert_eq!(
            shrine.serialization_format(),
            SerializationFormat::MessagePack
        );
    }

    #[test]
    fn local_shrine_encryption_format() {
        let shrine = LocalShrine::default();
        assert_eq!(shrine.encryption_algorithm(), EncryptionAlgorithm::Aes);

        let shrine = LocalShrine::default().into_clear();
        assert_eq!(shrine.encryption_algorithm(), EncryptionAlgorithm::Plain);
    }

    #[test]
    fn loaded_shrine_version() {
        let shrine = LocalShrine::default();
        assert_eq!(
            shrine.version(),
            format::default().lock().unwrap().version()
        );
    }

    #[test]
    fn loaded_shrine_serialization_format() {
        let shrine = LocalShrine::default();
        assert_eq!(
            shrine.serialization_format(),
            SerializationFormat::default()
        );
    }

    #[test]
    fn loaded_shrine_encryption_format() {
        let shrine = LocalShrine::default();
        assert_eq!(shrine.encryption_algorithm(), EncryptionAlgorithm::Aes);

        let shrine = LocalShrine::default().into_clear();
        assert_eq!(shrine.encryption_algorithm(), EncryptionAlgorithm::Plain);
    }

    #[test]
    fn set_get() {
        let mut shrine = LocalShrine::default();

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
        let mut shrine = LocalShrine::default();

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
        let mut shrine = LocalShrine::default();

        shrine
            .set("key", SecretBytes::from("value".as_bytes()), Mode::Text)
            .unwrap();
        assert!(shrine.rm("key").unwrap());

        let err = shrine.get("key").unwrap_err();
        match err {
            Error::KeyNotFound(k) => {
                assert_eq!(&k, "key")
            }
            e => panic!("Expected Error::KeyNotFound(\"key\"), got {:?}", e),
        }

        assert!(!shrine.rm("key").unwrap());
    }

    #[test]
    fn mv() {
        let mut src = LocalShrine::default();
        src.set("key", SecretBytes::from("value".as_bytes()), Mode::Text)
            .unwrap();

        let mut dst = OpenShrine::LocalClear(LocalShrine::default().into_clear());
        src.mv(&mut dst);

        let secret = dst.get("key").unwrap();
        assert_eq!(secret.value().expose_secret_as_bytes(), "value".as_bytes());
        assert_eq!(secret.mode(), Mode::Text);
    }

    #[test]
    fn keys() {
        let mut shrine = LocalShrine::default();

        shrine
            .set("key", SecretBytes::from("value".as_bytes()), Mode::Text)
            .unwrap();

        let keys = shrine.keys();
        assert_eq!(keys.len(), 1);
        assert_eq!(keys.get(0), Some(&"key".to_string()))
    }

    #[test]
    fn keys_private() {
        let mut shrine = LocalShrine::default();

        shrine
            .set(".key", SecretBytes::from("value".as_bytes()), Mode::Text)
            .unwrap();

        let keys = shrine.keys_private();
        assert_eq!(keys.len(), 1);
        assert_eq!(keys.get(0), Some(&"key".to_string()))
    }

    #[test]
    fn clear_close_open() {
        let mut shrine = LocalShrine::default();

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
        let mut shrine = LocalShrine::default();

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
        let mut shrine = LocalShrine::default();

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
        let mut shrine = LocalShrine::default();

        shrine
            .set("key", SecretBytes::from("value".as_bytes()), Mode::Text)
            .unwrap();

        let shrine = shrine.into_clear().close().unwrap();

        let bytes = shrine.to_bytes(EncryptionAlgorithm::Plain);

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
        let mut shrine = LocalShrine::default();

        shrine
            .set("key", SecretBytes::from("value".as_bytes()), Mode::Text)
            .unwrap();

        let shrine = shrine
            .into_clear()
            .into_aes_with_password(ShrinePassword::from("password"))
            .close()
            .unwrap();

        let bytes = shrine.to_bytes(EncryptionAlgorithm::Aes);

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

        let mut shrine = LocalShrine::default();
        shrine
            .set("key", SecretBytes::from("value".as_bytes()), Mode::Text)
            .unwrap();
        let shrine = shrine.close(ShrinePassword::from("password")).unwrap();
        shrine.write_to(&path, EncryptionAlgorithm::Aes).unwrap();

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
}
