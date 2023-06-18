use crate::shrine::{Closed, Mode, Shrine, ShrinePassword};
use crate::utils::read_password;
use crate::{Error, SHRINE_FILENAME};
use base64::Engine;
use regex::Regex;

use std::path::PathBuf;

pub fn dump(
    shrine: Shrine<Closed>,
    path: PathBuf,
    password: Option<ShrinePassword>,
    pattern: Option<&String>,
    private: bool,
) -> Result<(), Error> {
    let regex = pattern
        .map(|p| Regex::new(p.as_ref()))
        .transpose()
        .map_err(Error::InvalidPattern)?;

    let password = password.unwrap_or_else(|| read_password(&shrine));

    let shrine = shrine.open(&password)?;

    let mut keys = shrine
        .keys()
        .into_iter()
        .filter(|k| regex.as_ref().map(|r| r.is_match(k)).unwrap_or(true))
        .collect::<Vec<String>>();
    keys.sort_unstable();

    println!("Shrine `{}/{}`", &path.display(), SHRINE_FILENAME);
    println!("Secrets:");
    for key in keys.iter() {
        let secret = shrine.get(key)?;
        let value = match secret.mode() {
            Mode::Binary => base64::engine::general_purpose::STANDARD
                .encode(secret.value().expose_secret_as_bytes()),
            Mode::Text => {
                String::from_utf8_lossy(secret.value().expose_secret_as_bytes()).to_string()
            }
        };
        println!("  {}={}", key, value)
    }

    if private {
        let mut keys = shrine
            .keys_private()
            .into_iter()
            .filter(|k| regex.as_ref().map(|r| r.is_match(k)).unwrap_or(true))
            .collect::<Vec<String>>();
        keys.sort_unstable();

        println!("Configuration:");
        for key in keys.iter() {
            println!(
                "  {}={}",
                key,
                String::from_utf8_lossy(shrine.get_private(key.as_ref()).unwrap().as_ref())
            )
        }
    }

    Ok(())
}
