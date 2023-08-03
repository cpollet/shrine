use crate::shrine::{Mode, ShrineProvider};

use crate::Error;
use dotenv_parser::parse_dotenv;

use std::fs::read_to_string;

use std::path::{Path, PathBuf};

// https://crates.io/crates/dotenv-parser
// todo compliant with https://hexdocs.pm/dotenvy/dotenv-file-format.html

pub fn import<P>(mut shrine_provider: P, file: &PathBuf, prefix: Option<&str>) -> Result<(), Error>
where
    P: ShrineProvider,
{
    let mut shrine = shrine_provider.load_open()?;
    // let password = password.unwrap_or_else(|| {
    //     if shrine.requires_password() {
    //         read_password(shrine.uuid())
    //     } else {
    //         ShrinePassword::default()
    //     }
    // });
    // let mut shrine = shrine.open(&password)?;

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

    shrine_provider.save_open(shrine)
}
