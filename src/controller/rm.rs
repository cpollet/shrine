use crate::git::Repository;
use crate::io::{load_shrine, save_shrine};
use crate::utils::read_password;
use crate::Error;
use secrecy::Secret;
use std::path::PathBuf;

pub fn rm(path: PathBuf, password: Option<Secret<String>>, key: &String) -> Result<(), Error> {
    let shrine = load_shrine(&path).map_err(Error::ReadFile)?;

    let password = password.unwrap_or_else(|| read_password(&shrine));

    let mut shrine = shrine
        .open(&password)
        .map_err(|e| Error::InvalidFile(e.to_string()))?;

    shrine.remove(key.as_ref());

    let repository = Repository::new(path.clone(), &shrine);

    let shrine = shrine
        .close(&password)
        .map_err(|e| Error::Update(e.to_string()))?;

    save_shrine(&path, &shrine).map_err(Error::WriteFile)?;

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
