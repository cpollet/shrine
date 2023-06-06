use crate::git::Repository;
use crate::io::{load_shrine, save_shrine};
use crate::shrine::{EncryptionAlgorithm, ShrineBuilder};
use crate::utils::{read_new_password, read_password};
use crate::Error;
use secrecy::Secret;
use std::path::PathBuf;

pub fn convert(
    path: PathBuf,
    password: Option<Secret<String>>,
    change_password: bool,
    new_password: Option<Secret<String>>,
    encryption_algorithm: Option<EncryptionAlgorithm>,
) -> Result<(), Error> {
    let change_password = change_password || new_password.is_some();
    if !change_password && encryption_algorithm.is_none() {
        return Ok(());
    }

    let mut change_password = change_password;

    let shrine = load_shrine(&path).map_err(Error::ReadFile)?;
    let password = password.unwrap_or_else(|| read_password(&shrine));
    let shrine = shrine
        .open(&password)
        .map_err(|e| Error::InvalidFile(e.to_string()))?;

    let shrine_builder =
        ShrineBuilder::new().with_encryption_algorithm(shrine.encryption_algorithm());

    let shrine_builder = match encryption_algorithm {
        Some(a) if shrine.encryption_algorithm() != a => {
            change_password = true;
            shrine_builder.with_encryption_algorithm(a)
        }
        _ => shrine_builder,
    };

    let mut new_shrine = shrine_builder.build();

    let password = if change_password {
        new_password
            .map(Ok)
            .unwrap_or_else(|| read_new_password(&new_shrine))?
    } else {
        password
    };

    shrine.move_to(&mut new_shrine);

    let repository = Repository::new(path.clone(), &new_shrine);

    let new_shrine = new_shrine
        .close(&password)
        .map_err(|e| Error::Update(e.to_string()))?;

    save_shrine(&path, &new_shrine).map_err(Error::WriteFile)?;

    if let Some(repository) = repository {
        if repository.commit_auto() {
            repository
                .open()
                .and_then(|r| r.create_commit("Update shrine"))
                .map_err(Error::Git)?;
        }
    }

    Ok(())
}
