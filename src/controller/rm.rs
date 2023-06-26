use crate::git::Repository;
use crate::shrine::{Shrine, ShrinePassword};
use crate::utils::read_password;
use crate::Error;

use crate::agent::client::Client;
use std::path::Path;

pub fn rm<C, P>(
    client: &C,
    path: P,
    password: Option<ShrinePassword>,
    key: &str,
) -> Result<(), Error>
where
    C: Client,
    P: AsRef<Path>,
{
    if client.is_running() {
        client.delete_key(path.as_ref().to_str().unwrap(), key)?;
    } else {
        let shrine = Shrine::from_path(&path)?;
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::client::mock::MockClient;
    use crate::shrine::Secret;

    #[test]
    fn delete_key_through_agent() {
        let mut mock = MockClient::default();
        mock.with_is_running(true);
        mock.with_delete_key(
            "path",
            "key",
            Ok(vec![serde_json::from_str::<Secret>(
                r#"
                {
                    "value": [115,101,99,114,101,116],
                    "mode": "Text",
                    "created_by": "cpollet@localhost",
                    "created_at": "2023-06-20T17:51:11.786655084Z"
                }
            "#,
            )
            .unwrap()]),
        );

        rm(&mock, "path", None, "key").expect("Expect Ok(())")
    }
}
