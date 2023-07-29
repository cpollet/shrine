use crate::agent::client::Client;
use crate::git::Repository;
use crate::shrine::{Mode, Shrine, ShrinePassword};
use crate::utils::read_password;
use crate::Error;
use rpassword::prompt_password;
use std::io::Read;
use std::path::Path;

pub struct Input<'a> {
    pub read_from_stdin: bool,
    pub mode: Mode,
    pub value: Option<&'a str>,
}

pub fn set<C, P>(
    client: &C,
    path: P,
    password: Option<ShrinePassword>,
    key: &str,
    input: Input<'_>,
) -> Result<(), Error>
where
    C: Client,
    P: AsRef<Path>,
{
    let value = if input.read_from_stdin {
        let mut input = Vec::new();
        let stdin = std::io::stdin();
        let mut handle = stdin.lock();
        handle.read_to_end(&mut input).map_err(Error::ReadStdIn)?;
        input
    } else {
        input
            .value
            .map(|v| v.to_string())
            .unwrap_or_else(|| prompt_password(format!("Enter `{}` value: ", key)).unwrap())
            .as_bytes()
            .to_vec()
    };
    let value = value.as_slice();

    if client.is_running() {
        client.set_key(path.as_ref().to_str().unwrap(), key, value, input.mode)?;
    } else {
        let shrine = Shrine::from_path(&path)?;
        let password = password.unwrap_or_else(|| read_password(&shrine));
        let mut shrine = shrine.open(&password)?;
        let repository = Repository::new(&path, &shrine);
        shrine.set(key.as_ref(), value, input.mode)?;
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::client::mock::MockClient;

    #[test]
    fn set_through_agent() {
        let mut mock = MockClient::default();
        mock.with_is_running(true);
        mock.with_set_key("path", "key", "value".as_bytes(), &Mode::Text, Ok(()));

        set(
            &mock,
            "path",
            None,
            "key",
            Input {
                read_from_stdin: false,
                mode: Mode::Text,
                value: Some("value"),
            },
        )
        .expect("Expect Ok(())")
    }
}
