use crate::io::{load_shrine_file, save_shrine_file};
use crate::utils::read_password;
use crate::Error;
use secrecy::Secret;
use std::path::PathBuf;

pub fn rm(folder: PathBuf, password: Option<Secret<String>>, key: &String) -> Result<(), Error> {
    let shrine_file = load_shrine_file(&folder).map_err(Error::ReadFile)?;

    let password = password.unwrap_or_else(|| read_password(&shrine_file));

    let mut shrine = shrine_file
        .unwrap(&password)
        .map_err(|e| Error::InvalidFile(e.to_string()))?;

    shrine.remove(key.as_ref());

    let mut shrine_file = shrine_file;
    shrine_file
        .wrap(shrine, &password)
        .map_err(|e| Error::Update(e.to_string()))?;

    save_shrine_file(&folder, &shrine_file).map_err(Error::WriteFile)
}
