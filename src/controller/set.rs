use crate::git::Repository;
use crate::shrine::{Closed, Mode, Shrine, ShrinePassword};
use crate::utils::read_password;
use crate::{agent, Error};
use rpassword::prompt_password;
use std::io::Read;
use std::path::Path;

pub fn set<P>(
    shrine: Shrine<Closed>,
    path: P,
    password: Option<ShrinePassword>,
    key: &String,
    read_from_stdin: bool,
    mode: Mode,
    value: Option<&str>,
) -> Result<(), Error>
where
    P: AsRef<Path>,
{
    let value = if read_from_stdin {
        let mut input = Vec::new();
        let stdin = std::io::stdin();
        let mut handle = stdin.lock();
        handle.read_to_end(&mut input).map_err(Error::ReadStdIn)?;
        input
    } else {
        value
            .map(|v| v.to_string())
            .unwrap_or_else(|| prompt_password(format!("Enter `{}` value: ", key)).unwrap())
            .as_bytes()
            .to_vec()
    };

    if agent::client::is_running() {
        agent::client::set_key(path.as_ref().to_str().unwrap(), key, value, mode)?;
    } else {
        let password = password.unwrap_or_else(|| read_password(&shrine));
        let mut shrine = shrine.open(&password)?;
        let repository = Repository::new(&path, &shrine);
        shrine.set(key.as_ref(), value.as_slice(), mode)?;
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
