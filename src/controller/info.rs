use crate::shrine::ShrineProvider;
use crate::{Error, SHRINE_FILENAME};

pub enum Fields {
    Version,
    Uuid,
    Serialization,
    Encryption,
}

pub fn info<P>(shrine_provider: P, field: Option<Fields>) -> Result<(), Error>
where
    P: ShrineProvider,
{
    let shrine = shrine_provider.load()?;
    match field {
        None => {
            println!(
                "File:          {}/{}",
                shrine_provider.path().display(),
                SHRINE_FILENAME
            );
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
