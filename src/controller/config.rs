use crate::git::Repository;
use crate::shrine::{ShrinePassword, ShrineProvider};
use crate::utils::read_password;
use crate::Error;
use rpassword::prompt_password;

use std::io::{stdout, Write};

pub fn set<P>(
    shrine_provider: P,
    password: Option<ShrinePassword>,
    key: &String,
    value: Option<&str>,
) -> Result<(), Error>
where
    P: ShrineProvider,
{
    let shrine = shrine_provider.load()?;
    let password = password.unwrap_or_else(|| read_password(&shrine));
    let mut shrine = shrine.open(&password)?;

    let repository = Repository::new(shrine_provider.path(), &shrine);

    let value = value
        .map(|v| v.to_string())
        .unwrap_or_else(|| prompt_password("Value: ").unwrap());

    shrine.set_private(key.to_string(), value);
    shrine_provider.save(shrine.close(&password)?)?;

    if let Some(repository) = repository {
        if repository.commit_auto() {
            repository
                .open()
                .and_then(|r| r.create_commit("Update shrine"))?;
        }
    }

    Ok(())
}

pub fn get<P>(
    shrine_provider: P,
    password: Option<ShrinePassword>,
    key: &String,
) -> Result<(), Error>
where
    P: ShrineProvider,
{
    let shrine = shrine_provider.load()?;
    let password = password.unwrap_or_else(|| read_password(&shrine));
    let shrine = shrine.open(&password)?;

    let secret = shrine
        .get_private(key.as_ref())
        .ok_or(Error::KeyNotFound(key.to_string()))?;

    let _ = stdout().write_all(secret.as_bytes());

    Ok(())
}
