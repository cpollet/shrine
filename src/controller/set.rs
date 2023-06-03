use crate::io::{load_shrine_file, save_shrine_file};

use crate::Error;

use crate::utils::read_password;
use rpassword::prompt_password;
use secrecy::Secret;

pub fn set(
    password: Option<Secret<String>>,
    key: &String,
    value: Option<&str>,
) -> Result<(), Error> {
    let shrine_file = load_shrine_file().map_err(Error::ReadFile)?;

    let password = password.unwrap_or_else(|| read_password(&shrine_file));

    let mut shrine = shrine_file
        .unwrap(&password)
        .map_err(|e| Error::InvalidFile(e.to_string()))?;

    let value = value
        .map(|v| v.to_string())
        .unwrap_or_else(|| prompt_password("Secret: ").unwrap());

    shrine.set(key.to_string(), value.as_bytes());

    let mut shrine_file = shrine_file;
    shrine_file
        .wrap(shrine, &password)
        .map_err(|e| Error::Update(e.to_string()))?;

    save_shrine_file(&shrine_file).map_err(Error::WriteFile)
}
