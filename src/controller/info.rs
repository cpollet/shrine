use crate::shrine::{Closed, Shrine};
use crate::{Error, SHRINE_FILENAME};
use std::path::PathBuf;

pub enum Fields {
    Version,
    Uuid,
    Serialization,
    Encryption,
}

pub fn info(shrine: Shrine<Closed>, path: PathBuf, field: Option<Fields>) -> Result<(), Error> {
    match field {
        None => {
            println!("File:          {}/{}", path.display(), SHRINE_FILENAME);
            println!("Version:       {}", shrine.version());
            println!("UUID:          {}", shrine.uuid());
            println!("Serialization: {}", shrine.serialization_format());
            println!("Encryption:    {}", shrine.encryption_algorithm());
        }
        Some(Fields::Version) => {
            println!("{}", shrine.version());
        }
        Some(Fields::Uuid) => {
            println!("{}", shrine.uuid());
        }
        Some(Fields::Serialization) => {
            println!("{}", shrine.serialization_format());
        }
        Some(Fields::Encryption) => {
            println!("{}", shrine.encryption_algorithm());
        }
    }

    Ok(())
}
