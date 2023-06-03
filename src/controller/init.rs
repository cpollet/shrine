use crate::io::save_shrine_file;
use crate::shrine::Shrine;
use crate::shrine_file::{EncryptionAlgorithm, ShrineFileBuilder};
use crate::{Error, SHRINE_FILENAME};
use std::path::Path;
use std::path::PathBuf;

use crate::utils::read_new_password;

use secrecy::Secret;
use std::string::ToString;

pub fn init(
    folder: PathBuf,
    password: Option<Secret<String>>,
    force: bool,
    encryption: Option<EncryptionAlgorithm>,
) -> Result<(), Error> {
    let mut file = PathBuf::from(&folder);
    file.push(SHRINE_FILENAME);

    if !force && Path::new(&file).exists() {
        return Err(Error::FileAlreadyExists(file.display().to_string()));
    }

    let mut shrine_file_builder = ShrineFileBuilder::new();

    if let Some(encryption) = encryption {
        shrine_file_builder = shrine_file_builder.with_encryption_algorithm(encryption);
    }

    let mut shrine_file = shrine_file_builder.build();

    let password = password
        .map(Ok)
        .unwrap_or_else(|| read_new_password(&shrine_file))?;

    shrine_file
        .wrap(Shrine::default(), &password)
        .map_err(|e| Error::Update(e.to_string()))?;

    save_shrine_file(&folder, &shrine_file).map_err(Error::WriteFile)
}
