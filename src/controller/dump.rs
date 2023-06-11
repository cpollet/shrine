use crate::shrine::{Closed, Shrine};
use crate::utils::read_password;
use crate::{Error, SHRINE_FILENAME};
use regex::Regex;
use secrecy::Secret;
use std::path::PathBuf;

pub fn dump(
    shrine: Shrine<Closed>,
    path: PathBuf,
    password: Option<Secret<String>>,
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
        println!(
            "  {}={}",
            key,
            String::from_utf8_lossy(shrine.get(key).unwrap().value().expose_secret_as_bytes())
        )
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
