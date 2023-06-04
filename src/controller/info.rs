use crate::io::load_shrine_file;
use crate::{Error, SHRINE_FILENAME};
use std::path::PathBuf;

pub enum Fields {
    Version,
    Uuid,
    Serialization,
    Encryption,
}

pub fn info(path: PathBuf, field: Option<Fields>) -> Result<(), Error> {
    let shrine_file = load_shrine_file(&path).map_err(Error::ReadFile)?;

    match field {
        None => {
            println!("File:          {}/{}", path.display(), SHRINE_FILENAME);
            println!("Version:       {}", shrine_file.version());
            println!("UUID:          {}", shrine_file.uuid());
            println!("Serialization: {}", shrine_file.serialization_format());
            println!("Encryption:    {}", shrine_file.encryption_algorithm());
        }
        Some(Fields::Version) => {
            println!("{}", shrine_file.version());
        }
        Some(Fields::Uuid) => {
            println!("{}", shrine_file.uuid());
        }
        Some(Fields::Serialization) => {
            println!("{}", shrine_file.serialization_format());
        }
        Some(Fields::Encryption) => {
            println!("{}", shrine_file.encryption_algorithm());
        }
    }

    Ok(())
}
