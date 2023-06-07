use crate::shrine::{Closed, Shrine};
use crate::utils::read_password;
use crate::Error;
use regex::Regex;
use secrecy::Secret;

pub fn ls(
    shrine: Shrine<Closed>,
    password: Option<Secret<String>>,
    pattern: Option<&String>,
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

    for key in keys.iter() {
        println!("{}", key)
    }

    println!("-> {} keys found", keys.len());

    Ok(())
}
