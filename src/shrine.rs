use crate::agent::client::Client;
use crate::shrine::encryption::EncryptionAlgorithm;
use crate::shrine::local::{
    Aes, Clear, Closed, LoadedShrine, LocalShrine, NoPassword, Open, Password,
};
use crate::shrine::remote::RemoteShrine;
use crate::shrine::serialization::SerializationFormat;
use crate::values::secret::{Mode, Secret};
use crate::Error;
use std::fmt::Debug;
use std::path::Path;
use uuid::Uuid;

mod holder;
mod metadata;
// todo convert to private
pub mod encryption;
pub mod local;
mod remote;
pub mod serialization;

/// Max supported file version
pub const VERSION: u8 = 0;

pub fn new<P>(client: Box<dyn Client>, path: P) -> Result<ClosedShrine, Error>
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

    // todo use SecretBytes
    fn set(&mut self, key: &str, value: &[u8], mode: Mode) -> Result<(), Self::Error>;

    fn get(&self, key: &str) -> Result<&Secret, Self::Error>;

    fn rm(&mut self, key: &str) -> bool;

    fn mv(self, other: &mut OpenShrine);

    fn keys(&self) -> Vec<String>;

    fn keys_private(&self) -> Vec<String>;
}

pub enum ClosedShrine {
    LocalClear(LocalShrine<Closed, Clear>),
    LocalAes(LocalShrine<Closed, Aes<NoPassword>>),
    Remote(RemoteShrine),
}

impl ClosedShrine {
    pub fn open<F>(self, password_provider: F) -> Result<OpenShrine, Error>
    where
        F: Fn(Uuid) -> String,
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

    pub fn write_file<P>(&self, path: P) -> Result<(), Error>
    where
        P: AsRef<Path>,
    {
        match self {
            ClosedShrine::LocalClear(s) => s.write_file(path),
            ClosedShrine::LocalAes(s) => s.write_file(path),
            ClosedShrine::Remote(_) => Ok(()),
        }
    }
}

impl QueryClosed for ClosedShrine {
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

impl From<LoadedShrine> for ClosedShrine {
    fn from(value: LoadedShrine) -> Self {
        match value {
            LoadedShrine::Clear(s) => ClosedShrine::LocalClear(s),
            LoadedShrine::Aes(s) => ClosedShrine::LocalAes(s),
        }
    }
}

pub enum OpenShrine {
    LocalClear(LocalShrine<Open, Clear>),
    LocalAes(LocalShrine<Open, Aes<Password>>),
    Remote(RemoteShrine),
}

impl OpenShrine {
    pub fn close(self) -> Result<ClosedShrine, Error> {
        Ok(match self {
            OpenShrine::LocalClear(s) => ClosedShrine::LocalClear(s.close()?),
            OpenShrine::LocalAes(s) => ClosedShrine::LocalAes(s.close()?),
            OpenShrine::Remote(s) => ClosedShrine::Remote(s),
        })
    }
}

impl QueryClosed for OpenShrine {
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

impl QueryOpen for OpenShrine {
    type Error = Error;

    // todo use SecretBytes
    fn set(&mut self, key: &str, value: &[u8], mode: Mode) -> Result<(), Self::Error> {
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

    fn mv(self, other: &mut OpenShrine) {
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
