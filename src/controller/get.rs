use crate::shrine::{Closed, Mode, Secret, Shrine, ShrinePassword};
use crate::utils::read_password;
use crate::Error;
use atty::Stream;
use base64::Engine;
use std::io::{stdout, Write};

pub fn get(
    shrine: Shrine<Closed>,
    password: Option<ShrinePassword>,
    key: &String,
    encoding: Encoding,
) -> Result<(), Error> {
    let password = password.unwrap_or_else(|| read_password(&shrine));

    let shrine = shrine.open(&password)?;

    let secret = shrine.get(key.as_ref())?;

    let _ = stdout().write_all(encoding.encode(secret).as_slice());

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
