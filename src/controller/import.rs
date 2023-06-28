use crate::shrine::{Mode, ShrinePassword, ShrineProvider};
use crate::utils::read_password;
use crate::Error;
use dotenv_parser::parse_dotenv;

use std::fs::read_to_string;

use std::path::{Path, PathBuf};

// https://crates.io/crates/dotenv-parser
// todo compliant with https://hexdocs.pm/dotenvy/dotenv-file-format.html

pub fn import<P>(
    shrine_provider: P,
    password: Option<ShrinePassword>,
    file: &PathBuf,
    prefix: Option<&str>,
) -> Result<(), Error>
where
    P: ShrineProvider,
{
    let shrine = shrine_provider.load()?;
    let password = password.unwrap_or_else(|| read_password(&shrine));
    let mut shrine = shrine.open(&password)?;

    let prefix = prefix.unwrap_or_default();

    let file = Path::new(file);
    if !(file.exists() && file.is_file()) {
        return Err(Error::FileNotFound(file.to_path_buf()));
    }

    let content = read_to_string(file).map_err(Error::IoRead)?;

    let secrets =
        parse_dotenv(&content).map_err(|e| Error::InvalidDotEnv(e, file.to_path_buf()))?;

    for (key, value) in secrets {
        shrine.set(&format!("{}{}", prefix, key), value.as_bytes(), Mode::Text)?
    }

    shrine_provider.save(shrine.close(&password)?)
}
