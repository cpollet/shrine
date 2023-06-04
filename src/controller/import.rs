use crate::io::{load_shrine_file, save_shrine_file};
use crate::utils::read_password;
use crate::Error;

use dotenv_parser::parse_dotenv;
use secrecy::Secret;

use std::fs::read_to_string;
use std::io;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

// https://crates.io/crates/dotenv-parser
// todo compliant with https://hexdocs.pm/dotenvy/dotenv-file-format.html

pub fn import(
    path: PathBuf,
    password: Option<Secret<String>>,
    file: &PathBuf,
    prefix: Option<&str>,
) -> Result<(), Error> {
    let shrine_file = load_shrine_file(&path).map_err(Error::ReadFile)?;

    let password = password.unwrap_or_else(|| read_password(&shrine_file));

    let mut shrine = shrine_file
        .unwrap(&password)
        .map_err(|e| Error::InvalidFile(e.to_string()))?;

    let prefix = prefix.unwrap_or_default();

    let file = Path::new(file);
    if !(file.exists() && file.is_file()) {
        return Err(Error::ReadFile(io::Error::new(
            ErrorKind::InvalidInput,
            format!("Could not import `{}`: not a file", file.display()),
        )));
    }

    let content = read_to_string(file).map_err(Error::ReadFile)?;

    let secrets = parse_dotenv(&content).map_err(|e| Error::InvalidFile(e.to_string()))?;

    for (key, value) in secrets {
        shrine.set(format!("{}{}", prefix, key), value.as_bytes())
    }

    let mut shrine_file = shrine_file;
    shrine_file
        .wrap(shrine, &password)
        .map_err(|e| Error::Update(e.to_string()))?;

    save_shrine_file(&path, &shrine_file)
        .map_err(Error::WriteFile)
        .map(|_| ())
}
