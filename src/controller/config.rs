use crate::git::Repository;
use crate::shrine::ShrineProvider;

use crate::Error;
use rpassword::prompt_password;

use std::io::{stdout, Write};

pub fn set<P>(mut shrine_provider: P, key: String, value: Option<String>) -> Result<(), Error>
where
    P: ShrineProvider,
{
    let mut shrine = shrine_provider.load_open()?;

    let repository = Repository::new(shrine_provider.path(), &shrine);

    let value = value.unwrap_or_else(|| prompt_password("Value: ").unwrap());

    shrine.set_private(key, value);
    shrine_provider.save_open(shrine)?;

    if let Some(repository) = repository {
        if repository.commit_auto() {
            repository
                .open()
                .and_then(|r| r.create_commit("Update shrine"))?;
        }
    }

    Ok(())
}

pub fn get<P>(mut shrine_provider: P, key: &str) -> Result<(), Error>
where
    P: ShrineProvider,
{
    let shrine = shrine_provider.load_open()?;

    let secret = shrine
        .get_private(key)
        .ok_or(Error::KeyNotFound(key.to_string()))?;

    let _ = stdout().write_all(secret.as_bytes());

    Ok(())
}
