use crate::agent::client::Client;
use crate::shrine::encryption::EncryptionAlgorithm;
use crate::shrine::serialization::SerializationFormat;
use crate::shrine::OpenShrine;
use crate::values::bytes::SecretBytes;
use crate::values::secret::{Mode, Secret};
use crate::Error;
use std::path::{Path, PathBuf};
use uuid::Uuid;

pub struct RemoteShrine {
    path: PathBuf,
    client: Box<dyn Client>,
}

impl RemoteShrine {
    pub fn new(path: PathBuf, client: Box<dyn Client>) -> Self {
        Self { path, client }
    }

    pub fn path(&self) -> &Path {
        self.path.as_path()
    }

    pub fn uuid(&self) -> Uuid {
        todo!()
    }

    pub fn version(&self) -> u8 {
        todo!()
    }

    pub fn serialization_format(&self) -> SerializationFormat {
        todo!()
    }

    pub fn encryption_algorithm(&self) -> EncryptionAlgorithm {
        todo!()
    }

    pub fn set(&mut self, key: &str, value: SecretBytes, mode: Mode) -> Result<(), Error> {
        self.client
            .set_key(self.path.to_str().unwrap(), key, value, mode)
    }

    pub fn get(&self, _key: &str) -> Result<&Secret, Error> {
        todo!()
    }

    pub fn rm(&mut self, _key: &str) -> bool {
        todo!()
    }

    pub fn mv<T>(self, _other: &mut OpenShrine<T>) {
        todo!()
    }

    pub fn keys(&self) -> Vec<String> {
        todo!()
    }

    pub fn keys_private(&self) -> Vec<String> {
        todo!()
    }
}
