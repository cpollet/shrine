use crate::agent::client::Client;
use crate::shrine::encryption::EncryptionAlgorithm;
use crate::shrine::local::{Aes, Clear, Closed, LoadedShrine, LocalShrine, NoPassword, Open};
use crate::shrine::remote::RemoteShrine;
use crate::shrine::serialization::SerializationFormat;
use crate::values::bytes::SecretBytes;
use crate::values::password::ShrinePassword;
use crate::values::secret::{Mode, Secret};
use crate::Error;
use std::fmt::Debug;
use std::path::{Path, PathBuf};
use uuid::Uuid;

pub mod encryption;
mod holder;
pub mod local;
mod metadata;
mod remote;
pub mod serialization;

/// Max supported file version
pub const VERSION: u8 = 0;

pub fn new<P>(client: Box<dyn Client>, path: P) -> Result<ClosedShrine<PathBuf>, Error>
where
    P: AsRef<Path>,
{
    if client.is_running() {
        Ok(ClosedShrine::Remote(RemoteShrine::new(
            path.as_ref().display().to_string(),
            client,
        )))
    } else {
        LoadedShrine::try_from_path(path).map(|s| s.into())
    }
}

pub trait QueryClosed {
    fn uuid(&self) -> Uuid;

    fn version(&self) -> u8;

    fn serialization_format(&self) -> SerializationFormat;

    fn encryption_algorithm(&self) -> EncryptionAlgorithm;
}

pub trait QueryOpen: QueryClosed {
    type Error: Debug;

    fn set(&mut self, key: &str, value: SecretBytes, mode: Mode) -> Result<(), Self::Error>;

    fn get(&self, key: &str) -> Result<&Secret, Self::Error>;

    fn rm(&mut self, key: &str) -> bool;

    fn mv<T>(self, other: &mut OpenShrine<T>);

    fn keys(&self) -> Vec<String>;

    fn keys_private(&self) -> Vec<String>;
}

pub enum ClosedShrine<L> {
    LocalClear(LocalShrine<Closed, Clear, L>),
    LocalAes(LocalShrine<Closed, Aes<NoPassword>, L>),
    Remote(RemoteShrine),
}

impl<L> ClosedShrine<L> {
    pub fn open<F>(self, password_provider: F) -> Result<OpenShrine<L>, Error>
    where
        F: FnOnce(Uuid) -> ShrinePassword,
    {
        Ok(match self {
            ClosedShrine::LocalClear(s) => s.open().map(OpenShrine::LocalClear)?,
            ClosedShrine::LocalAes(s) => {
                let uuid = s.uuid();
                s.open(password_provider(uuid)).map(OpenShrine::LocalAes)?
            }
            ClosedShrine::Remote(s) => OpenShrine::Remote(s),
        })
    }
}

impl<L> QueryClosed for ClosedShrine<L> {
    fn uuid(&self) -> Uuid {
        match self {
            ClosedShrine::LocalClear(s) => s.uuid(),
            ClosedShrine::LocalAes(s) => s.uuid(),
            ClosedShrine::Remote(s) => s.uuid(),
        }
    }

    fn version(&self) -> u8 {
        match self {
            ClosedShrine::LocalClear(s) => s.version(),
            ClosedShrine::LocalAes(s) => s.version(),
            ClosedShrine::Remote(s) => s.version(),
        }
    }

    fn serialization_format(&self) -> SerializationFormat {
        match self {
            ClosedShrine::LocalClear(s) => s.serialization_format(),
            ClosedShrine::LocalAes(s) => s.serialization_format(),
            ClosedShrine::Remote(s) => s.serialization_format(),
        }
    }

    fn encryption_algorithm(&self) -> EncryptionAlgorithm {
        match self {
            ClosedShrine::LocalClear(s) => s.encryption_algorithm(),
            ClosedShrine::LocalAes(s) => s.encryption_algorithm(),
            ClosedShrine::Remote(s) => s.encryption_algorithm(),
        }
    }
}

impl From<LoadedShrine> for ClosedShrine<PathBuf> {
    fn from(value: LoadedShrine) -> Self {
        match value {
            LoadedShrine::Clear(s) => ClosedShrine::LocalClear(s),
            LoadedShrine::Aes(s) => ClosedShrine::LocalAes(s),
        }
    }
}

pub enum OpenShrine<L> {
    LocalClear(LocalShrine<Open, Clear, L>),
    LocalAes(LocalShrine<Open, Aes<ShrinePassword>, L>),
    Remote(RemoteShrine),
}

impl<L> OpenShrine<L> {
    pub fn close(self) -> Result<ClosedShrine<L>, Error> {
        Ok(match self {
            OpenShrine::LocalClear(s) => ClosedShrine::LocalClear(s.close()?),
            OpenShrine::LocalAes(s) => ClosedShrine::LocalAes(s.close()?),
            OpenShrine::Remote(s) => ClosedShrine::Remote(s),
        })
    }
}

impl<L> QueryClosed for OpenShrine<L> {
    fn uuid(&self) -> Uuid {
        match self {
            OpenShrine::LocalClear(s) => s.uuid(),
            OpenShrine::LocalAes(s) => s.uuid(),
            OpenShrine::Remote(s) => s.uuid(),
        }
    }

    fn version(&self) -> u8 {
        match self {
            OpenShrine::LocalClear(s) => s.version(),
            OpenShrine::LocalAes(s) => s.version(),
            OpenShrine::Remote(s) => s.version(),
        }
    }

    fn serialization_format(&self) -> SerializationFormat {
        match self {
            OpenShrine::LocalClear(s) => s.serialization_format(),
            OpenShrine::LocalAes(s) => s.serialization_format(),
            OpenShrine::Remote(s) => s.serialization_format(),
        }
    }

    fn encryption_algorithm(&self) -> EncryptionAlgorithm {
        match self {
            OpenShrine::LocalClear(s) => s.encryption_algorithm(),
            OpenShrine::LocalAes(s) => s.encryption_algorithm(),
            OpenShrine::Remote(s) => s.encryption_algorithm(),
        }
    }
}

impl<L> QueryOpen for OpenShrine<L> {
    type Error = Error;

    // todo use SecretBytes
    fn set(&mut self, key: &str, value: SecretBytes, mode: Mode) -> Result<(), Self::Error> {
        match self {
            OpenShrine::LocalClear(s) => s.set(key, value, mode),
            OpenShrine::LocalAes(s) => s.set(key, value, mode),
            OpenShrine::Remote(s) => s.set(key, value, mode),
        }
    }

    fn get(&self, key: &str) -> Result<&Secret, Self::Error> {
        match self {
            OpenShrine::LocalClear(s) => s.get(key),
            OpenShrine::LocalAes(s) => s.get(key),
            OpenShrine::Remote(s) => s.get(key),
        }
    }

    fn rm(&mut self, key: &str) -> bool {
        match self {
            OpenShrine::LocalClear(s) => s.rm(key),
            OpenShrine::LocalAes(s) => s.rm(key),
            OpenShrine::Remote(s) => s.rm(key),
        }
    }

    fn mv<T>(self, other: &mut OpenShrine<T>) {
        match self {
            OpenShrine::LocalClear(s) => s.mv(other),
            OpenShrine::LocalAes(s) => s.mv(other),
            OpenShrine::Remote(s) => s.mv(other),
        }
    }

    fn keys(&self) -> Vec<String> {
        match self {
            OpenShrine::LocalClear(s) => s.keys(),
            OpenShrine::LocalAes(s) => s.keys(),
            OpenShrine::Remote(s) => s.keys(),
        }
    }

    fn keys_private(&self) -> Vec<String> {
        match self {
            OpenShrine::LocalClear(s) => s.keys_private(),
            OpenShrine::LocalAes(s) => s.keys_private(),
            OpenShrine::Remote(s) => s.keys_private(),
        }
    }
}

// todo add tests
