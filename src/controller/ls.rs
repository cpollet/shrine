use crate::io::load_shrine;
use crate::utils::read_password;
use crate::Error;
use regex::Regex;
use secrecy::Secret;
use std::path::PathBuf;

pub fn ls(
    path: PathBuf,
    password: Option<Secret<String>>,
    pattern: Option<&String>,
) -> Result<(), Error> {
    let regex = pattern
        .map(|p| Regex::new(p.as_ref()))
        .transpose()
        .map_err(Error::InvalidPattern)?;

    let shrine = load_shrine(&path).map_err(Error::ReadFile)?;

    let password = password.unwrap_or_else(|| read_password(&shrine));

    let shrine = shrine
        .open(&password)
        .map_err(|e| Error::InvalidFile(e.to_string()))?;

    let mut keys = shrine
        .keys()
        .into_iter()
        .filter(|k| regex.as_ref().map(|r| r.is_match(k)).unwrap_or(true))
        .collect::<Vec<String>>();
    keys.sort_unstable();

    for key in keys.iter() {
        println!("{}", key)
    }

    println!("-> {} keys found", keys.len());

    Ok(())
}
