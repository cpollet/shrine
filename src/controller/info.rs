use crate::shrine::{ClosedShrine, QueryClosed};
use crate::Error;
use std::path::Path;

pub enum Fields {
    Version,
    Uuid,
    Serialization,
    Encryption,
}

pub fn info<P>(shrine: &ClosedShrine, field: Option<Fields>, path: P) -> Result<(), Error>
where
    P: AsRef<Path>,
{
    match field {
        None => {
            println!("File:          {}", path.as_ref().display());
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
