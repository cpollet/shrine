use crate::git::Repository;
use crate::shrine::{Closed, Shrine, ShrinePassword};
use crate::utils::read_password;
use crate::{agent, Error};

use std::path::Path;

pub fn rm<P>(
    shrine: Shrine<Closed>,
    path: P,
    password: Option<ShrinePassword>,
    key: &String,
) -> Result<(), Error>
where
    P: AsRef<Path>,
{
    if agent::client::is_running() {
        agent::client::delete_key(path.as_ref().to_str().unwrap(), key)?;
    } else {
        let password = password.unwrap_or_else(|| read_password(&shrine));

        let mut shrine = shrine.open(&password)?;
        let repository = Repository::new(&path, &shrine);

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
    }

    Ok(())
}
