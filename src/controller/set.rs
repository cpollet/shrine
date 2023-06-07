use crate::git::Repository;
use crate::shrine::{Closed, Shrine};
use crate::utils::read_password;
use crate::Error;
use rpassword::prompt_password;
use secrecy::Secret;
use std::path::PathBuf;

pub fn set(
    shrine: Shrine<Closed>,
    path: PathBuf,
    password: Option<Secret<String>>,
    key: &String,
    value: Option<&str>,
) -> Result<(), Error> {
    let password = password.unwrap_or_else(|| read_password(&shrine));

    let mut shrine = shrine.open(&password)?;
    let repository = Repository::new(path.clone(), &shrine);

    let value = value
        .map(|v| v.to_string())
        .unwrap_or_else(|| prompt_password(format!("Enter `{}` value: ", key)).unwrap());

    shrine.set(key.to_string(), value.as_bytes());
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
