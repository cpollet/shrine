use crate::io::{load_shrine_file, save_shrine_file};
use crate::shrine_file::{EncryptionAlgorithm, ShrineFileBuilder};
use crate::utils::{read_new_password, read_password};
use crate::Error;

pub fn convert(
    change_password: bool,
    encryption_algorithm: Option<EncryptionAlgorithm>,
) -> Result<(), Error> {
    if !change_password && encryption_algorithm.is_none() {
        return Ok(());
    }

    let mut change_password = change_password;

    let shrine_file = load_shrine_file().map_err(Error::ReadFile)?;
    let password = read_password(&shrine_file);
    let shrine = shrine_file
        .unwrap(&password)
        .map_err(|e| Error::InvalidFile(e.to_string()))?;

    let shrine_file_builder =
        ShrineFileBuilder::new().with_encryption_algorithm(shrine_file.encryption_algorithm());

    let shrine_file_builder = match encryption_algorithm {
        Some(a) if shrine_file.encryption_algorithm() != a => {
            change_password = true;
            shrine_file_builder.with_encryption_algorithm(a)
        }
        _ => shrine_file_builder,
    };

    let mut new_shrine_file = shrine_file_builder.build();

    let password = if change_password {
        read_new_password(&new_shrine_file)?
    } else {
        password
    };

    new_shrine_file
        .wrap(shrine, &password)
        .map_err(|e| Error::Update(e.to_string()))?;

    save_shrine_file(&new_shrine_file).map_err(Error::WriteFile)
}
