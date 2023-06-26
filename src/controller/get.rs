use crate::agent::client::Client;
use crate::shrine::{Mode, Secret, Shrine, ShrinePassword};
use crate::utils::read_password;
use crate::Error;
use atty::Stream;
use base64::Engine;
use std::io::Write;
use std::path::Path;

pub fn get<C, P, W>(
    client: &C,
    path: P,
    password: Option<ShrinePassword>,
    key: &str,
    encoding: Encoding,
    out: &mut W,
) -> Result<(), Error>
where
    C: Client,
    P: AsRef<Path>,
    W: Write,
{
    let secret = if client.is_running() {
        encoding.encode(&client.get_key(path.as_ref().to_str().unwrap(), key)?)
    } else {
        let shrine = Shrine::from_path(&path)?;
        let password = password.unwrap_or_else(|| read_password(&shrine));
        let shrine = shrine.open(&password)?;
        let secret = shrine.get(key.as_ref())?;
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

    #[test]
    fn get_through_agent() {
        let mut mock = MockClient::default();
        mock.with_is_running(true);
        mock.with_get_key(
            "path",
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

        get(&mock, "path", None, "key", Encoding::Raw, &mut out).expect("expected Ok(())");

        assert_eq!(out.as_slice(), "secret".as_bytes());
    }
}
