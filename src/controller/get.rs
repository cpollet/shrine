use crate::shrine::{Closed, Shrine};
use crate::utils::read_password;
use crate::Error;
use secrecy::Secret;
use std::io::{stdout, Write};

pub fn get(
    shrine: Shrine<Closed>,
    password: Option<Secret<String>>,
    key: &String,
) -> Result<(), Error> {
    let password = password.unwrap_or_else(|| read_password(&shrine));

    let shrine = shrine.open(&password)?;

    let secret = shrine.get(key.as_ref())?;

    let _ = stdout().write_all(secret.expose_secret_as_bytes());

    Ok(())
}
