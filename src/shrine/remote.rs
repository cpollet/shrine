use crate::agent::client::Client;
use crate::shrine::encryption::EncryptionAlgorithm;
use crate::shrine::serialization::SerializationFormat;
use crate::shrine::{OpenShrine, QueryClosed, QueryOpen};
use crate::values::secret::{Mode, Secret};
use crate::Error;
use uuid::Uuid;

pub struct RemoteShrine {
    path: String,
    client: Box<dyn Client>,
}

impl RemoteShrine {
    pub fn new(path: String, client: Box<dyn Client>) -> Self {
        Self { path, client }
    }
}

impl QueryClosed for RemoteShrine {
    fn uuid(&self) -> Uuid {
        todo!()
    }

    fn version(&self) -> u8 {
        todo!()
    }

    fn serialization_format(&self) -> SerializationFormat {
        todo!()
    }

    fn encryption_algorithm(&self) -> EncryptionAlgorithm {
        todo!()
    }
}

impl QueryOpen for RemoteShrine {
    type Error = Error;

    fn set(&mut self, key: &str, value: &[u8], mode: Mode) -> Result<(), Self::Error> {
        self.client.set_key(&self.path, key, value, mode)
    }

    fn get(&self, _key: &str) -> Result<&Secret, Self::Error> {
        todo!()
    }

    fn rm(&mut self, _key: &str) -> bool {
        todo!()
    }

    fn mv<T>(self, _other: &mut OpenShrine<T>) {
        todo!()
    }

    fn keys(&self) -> Vec<String> {
        todo!()
    }

    fn keys_private(&self) -> Vec<String> {
        todo!()
    }
}
