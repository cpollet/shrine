use crate::git::Repository;
use crate::io::{load_shrine_file, save_shrine_file};
use crate::utils::read_password;
use crate::Error;
use rpassword::prompt_password;
use secrecy::Secret;
use std::io::{stdout, Write};
use std::path::PathBuf;

pub fn set(
    folder: PathBuf,
    password: Option<Secret<String>>,
    key: &String,
    value: Option<&str>,
) -> Result<(), Error> {
    let shrine_file = load_shrine_file(&folder).map_err(Error::ReadFile)?;

    let password = password.unwrap_or_else(|| read_password(&shrine_file));

    let mut shrine = shrine_file
        .unwrap(&password)
        .map_err(|e| Error::InvalidFile(e.to_string()))?;

    let value = value
        .map(|v| v.to_string())
        .unwrap_or_else(|| prompt_password("Value: ").unwrap());

    shrine.set_private(key.to_string(), value);

    let repository = Repository::new(folder.clone(), &shrine);

    let mut shrine_file = shrine_file;
    shrine_file
        .wrap(shrine, &password)
        .map_err(|e| Error::Update(e.to_string()))?;

    save_shrine_file(&folder, &shrine_file)
        .map_err(Error::WriteFile)
        .map(|_| ())?;

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

pub fn get(folder: PathBuf, password: Option<Secret<String>>, key: &String) -> Result<(), Error> {
    let shrine_file = load_shrine_file(&folder).map_err(Error::ReadFile)?;

    let password = password.unwrap_or_else(|| read_password(&shrine_file));

    let shrine = shrine_file
        .unwrap(&password)
        .map_err(|e| Error::InvalidFile(e.to_string()))?;

    let secret = shrine
        .get_private(key.as_ref())
        .ok_or(Error::KeyNotFound(key.to_string()))?;

    let _ = stdout().write_all(secret.as_bytes());

    Ok(())
}
