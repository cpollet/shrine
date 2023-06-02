use crate::io::save_shrine_file;
use crate::shrine::Shrine;
use crate::shrine_file::{EncryptionAlgorithm, ShrineFileBuilder};
use crate::{Error, SHRINE_FILENAME};

use std::path::Path;

use secrecy::Secret;
use std::string::ToString;

pub fn init(force: bool, encryption: Option<EncryptionAlgorithm>) -> Result<(), Error> {
    if !force && Path::new(SHRINE_FILENAME).exists() {
        return Err(Error::FileAlreadyExists(SHRINE_FILENAME.to_string()));
    }

    let mut shrine_file_builder = ShrineFileBuilder::new();

    if let Some(encryption) = encryption {
        shrine_file_builder = shrine_file_builder.with_encryption_algorithm(encryption);
    }

    let mut shrine_file = shrine_file_builder.build();

    let password = if shrine_file.requires_password() {
        let password1 = rpassword::prompt_password("Enter shrine password: ").unwrap();
        let password2 = rpassword::prompt_password("Enter shrine password (again): ").unwrap();
        if password1 != password2 {
            return Err(Error::InvalidPassword);
        }
        Secret::new(password1)
    } else {
        Secret::new("".to_string())
    };

    shrine_file
        .wrap(Shrine::default(), &password)
        .map_err(|e| Error::Update(e.to_string()))?;

    save_shrine_file(&shrine_file).map_err(Error::WriteFile)
}
