use crate::git::Repository;
use crate::shrine::{Closed, Mode, Shrine};
use crate::utils::read_password;
use crate::Error;
use rpassword::prompt_password;
use secrecy::Secret;
use std::io::Read;
use std::path::PathBuf;

pub fn set(
    shrine: Shrine<Closed>,
    path: PathBuf,
    password: Option<Secret<String>>,
    key: &String,
    read_from_stdin: bool,
    mode: Mode,
    value: Option<&str>,
) -> Result<(), Error> {
    let password = password.unwrap_or_else(|| read_password(&shrine));

    let mut shrine = shrine.open(&password)?;
    let repository = Repository::new(path.clone(), &shrine);

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

    shrine.set(key.as_ref(), value.as_slice(), mode)?;
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
