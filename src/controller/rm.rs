use crate::git::Repository;
use crate::shrine::{Closed, Shrine, ShrinePassword};
use crate::utils::read_password;
use crate::Error;

use std::path::PathBuf;

pub fn rm(
    shrine: Shrine<Closed>,
    path: PathBuf,
    password: Option<ShrinePassword>,
    key: &String,
) -> Result<(), Error> {
    let password = password.unwrap_or_else(|| read_password(&shrine));

    let mut shrine = shrine.open(&password)?;
    let repository = Repository::new(path.clone(), &shrine);

    if !shrine.remove(key) {
        return Err(Error::KeyNotFound(key.to_string()));
    }
    shrine.close(&password)?.to_path(&path)?;

    if let Some(repository) = repository {
        if repository.commit_auto() {
            repository
                .open()
                .and_then(|r| r.create_commit("Update shrine"))?;
        }
    }

    Ok(())
}
