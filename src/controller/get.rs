use crate::io::load_shrine_file;
use crate::utils::read_password;
use crate::Error;

use std::io::{stdout, Write};

pub fn get(key: &String) -> Result<(), Error> {
    let shrine_file = load_shrine_file().map_err(Error::ReadFile)?;

    let shrine = shrine_file
        .unwrap(&read_password(&shrine_file))
        .map_err(|e| Error::InvalidFile(e.to_string()))?;

    let secret = shrine.get(key.as_ref()).ok_or(Error::KeyNotFound)?;

    let _ = stdout().write_all(secret.expose_secret_as_bytes());

    Ok(())
}
