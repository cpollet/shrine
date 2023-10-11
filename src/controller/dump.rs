use crate::shrine::{Mode, ShrineProvider};
use crate::{Error, SHRINE_FILENAME};
use base64::Engine;
use regex::Regex;

pub fn dump<P>(mut shrine_provider: P, pattern: Option<&String>, private: bool) -> Result<(), Error>
where
    P: ShrineProvider,
{
    let regex = pattern
        .map(|p| Regex::new(p.as_ref()))
        .transpose()
        .map_err(Error::InvalidPattern)?;

    let shrine = shrine_provider.load_open()?;

    let mut keys = shrine
        .keys()
        .into_iter()
        .filter(|k| regex.as_ref().map(|r| r.is_match(k)).unwrap_or(true))
        .collect::<Vec<String>>();
    keys.sort_unstable();

    println!(
        "Shrine `{}/{}`",
        shrine_provider.path().display(),
        SHRINE_FILENAME
    );
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
            .collect::<Vec<&str>>();
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
