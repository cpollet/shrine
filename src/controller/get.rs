use crate::agent::client::Client;
use crate::shrine::{Mode, Secret, ShrineProvider};
use crate::Error;
use atty::Stream;
use base64::Engine;
use std::io::Write;

pub fn get<C, P, O>(
    client: C,
    mut shrine_provider: P,
    key: &str,
    encoding: Encoding,
    out: &mut O,
) -> Result<(), Error>
where
    C: Client,
    P: ShrineProvider,
    O: Write,
{
    let secret = if client.is_running() {
        encoding.encode(&client.get_key(shrine_provider.path().to_str().unwrap(), key)?)
    } else {
        let shrine = shrine_provider.load_open()?;
        let secret = shrine.get(key)?;
        encoding.encode(secret)
    };

    out.write_all(secret.as_slice()).unwrap();
    Ok(())
}

pub enum Encoding {
    Auto,
    Raw,
    Base64,
}

impl Encoding {
    fn encode(&self, secret: &Secret) -> Vec<u8> {
        match self {
            Encoding::Auto => match secret.mode() {
                Mode::Binary => {
                    if atty::is(Stream::Stdout) {
                        base64::engine::general_purpose::STANDARD
                            .encode(secret.value().expose_secret_as_bytes())
                            .into_bytes()
                    } else {
                        secret.value().expose_secret_as_bytes().to_vec()
                    }
                }
                Mode::Text => secret.value().expose_secret_as_bytes().to_vec(),
            },
            Encoding::Raw => secret.value().expose_secret_as_bytes().to_vec(),
            Encoding::Base64 => base64::engine::general_purpose::STANDARD
                .encode(secret.value().expose_secret_as_bytes())
                .into_bytes(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::client::mock::MockClient;
    use crate::shrine::mocks::MockShrineProvider;
    use crate::shrine::{EncryptionAlgorithm, ShrineBuilder, ShrinePassword};

    #[test]
    fn get_direct() {
        let mut client = MockClient::default();
        client.with_is_running(false);

        let mut shrine = ShrineBuilder::new()
            .with_encryption_algorithm(EncryptionAlgorithm::Plain)
            .build();
        shrine.set("key", "secret", Mode::Text).unwrap();
        let shrine = shrine.close(&ShrinePassword::default()).unwrap();

        let shrine_provider = MockShrineProvider::new(shrine);

        let mut out = Vec::<u8>::new();

        get(client, shrine_provider, "key", Encoding::Raw, &mut out).expect("expected Ok(())");

        assert_eq!(out.as_slice(), "secret".as_bytes());
    }

    #[test]
    fn get_through_agent() {
        let mut client = MockClient::default();
        client.with_is_running(true);
        client.with_get_key(
            "/path/to/shrine",
            "key",
            Ok(serde_json::from_str::<Secret>(
                r#"
                {
                    "value": [115,101,99,114,101,116],
                    "mode": "Text",
                    "created_by": "cpollet@localhost",
                    "created_at": "2023-06-20T17:51:11.786655084Z"
                }
            "#,
            )
            .unwrap()),
        );

        let mut out = Vec::<u8>::new();

        get(
            client,
            MockShrineProvider::default(),
            "key",
            Encoding::Raw,
            &mut out,
        )
        .expect("expected Ok(())");

        assert_eq!(out.as_slice(), "secret".as_bytes());
    }
}
