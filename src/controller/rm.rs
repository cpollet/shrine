use crate::git::Repository;
use crate::io::{load_shrine_file, save_shrine_file};
use crate::utils::read_password;
use crate::Error;
use secrecy::Secret;
use std::path::PathBuf;

pub fn rm(path: PathBuf, password: Option<Secret<String>>, key: &String) -> Result<(), Error> {
    let shrine_file = load_shrine_file(&path).map_err(Error::ReadFile)?;

    let password = password.unwrap_or_else(|| read_password(&shrine_file));

    let mut shrine = shrine_file
        .unwrap(&password)
        .map_err(|e| Error::InvalidFile(e.to_string()))?;

    shrine.remove(key.as_ref());

    let repository = Repository::new(path.clone(), &shrine);

    let mut shrine_file = shrine_file;
    shrine_file
        .wrap(shrine, &password)
        .map_err(|e| Error::Update(e.to_string()))?;

    save_shrine_file(&path, &shrine_file).map_err(Error::WriteFile)?;

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
