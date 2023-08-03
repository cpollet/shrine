use crate::agent::client::Client;
use crate::git::Repository;
use crate::shrine::ShrineProvider;

use crate::Error;

pub fn rm<C, P>(client: C, mut shrine_provider: P, key: &str) -> Result<(), Error>
where
    C: Client,
    P: ShrineProvider,
{
    if client.is_running() {
        client.delete_key(shrine_provider.path().to_str().unwrap(), key)?;
    } else {
        let mut shrine = shrine_provider.load_open()?;
        let repository = Repository::new(shrine_provider.path(), &shrine);

        if !shrine.remove(key) {
            return Err(Error::KeyNotFound(key.to_string()));
        }
        shrine_provider.save_open(shrine)?;

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
    use crate::shrine::mocks::MockShrineProvider;
    use crate::shrine::{EncryptionAlgorithm, Mode, Secret, ShrineBuilder, ShrinePassword};

    #[test]
    fn delete_direct() {
        let mut client = MockClient::default();
        client.with_is_running(false);

        let mut shrine = ShrineBuilder::new()
            .with_encryption_algorithm(EncryptionAlgorithm::Plain)
            .build();
        shrine.set("key", "secret", Mode::Text).unwrap();
        let shrine = shrine.close(&ShrinePassword::default()).unwrap();

        let shrine_provider = MockShrineProvider::new(shrine);

        rm(client, shrine_provider.clone(), "key").expect("Expect Ok(())");

        let shrine = shrine_provider
            .load_closed()
            .unwrap()
            .open(&ShrinePassword::default())
            .unwrap();
        let secret = shrine.get("key");

        let err = secret.expect_err("Expected Err(..)");

        assert_eq!(err.to_string(), "Key `key` does not exist".to_string());
    }

    #[test]
    fn delete_key_through_agent() {
        let mut client = MockClient::default();
        client.with_is_running(true);
        client.with_delete_key(
            "/path/to/shrine",
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

        let shrine_provider = MockShrineProvider::default();

        rm(client, shrine_provider, "key").expect("Expect Ok(())")
    }
}
