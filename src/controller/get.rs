use crate::agent;
use crate::shrine::{Mode, Secret, Shrine, ShrinePassword};
use crate::utils::read_password;
use crate::Error;
use atty::Stream;
use base64::Engine;
use std::io::{stdout, Write};
use std::path::Path;

pub fn get<P>(
    path: P,
    password: Option<ShrinePassword>,
    key: &String,
    encoding: Encoding,
) -> Result<(), Error>
where
    P: AsRef<Path>,
{
    if agent::client::is_running() {
        let secret = agent::client::get_key(path.as_ref().to_str().unwrap(), key)?;
        let _ = stdout().write_all(encoding.encode(&secret).as_slice());
    } else {
        let shrine = Shrine::from_path(&path)?;
        let password = password.unwrap_or_else(|| read_password(&shrine));
        let shrine = shrine.open(&password)?;
        let secret = shrine.get(key.as_ref())?;
        let _ = stdout().write_all(encoding.encode(secret).as_slice());
    };

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
