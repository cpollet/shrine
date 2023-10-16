use crate::shrine::{OpenShrine, QueryOpen};
use crate::values::secret::Mode;
use crate::Error;
use base64::Engine;
use regex::Regex;
use std::path::{Path, PathBuf};

pub fn dump<P>(
    shrine: &OpenShrine<PathBuf>,
    pattern: Option<&str>,
    private: bool,
    path: P,
) -> Result<(), Error>
where
    P: AsRef<Path>,
{
    let regex = pattern
        .map(Regex::new)
        .transpose()
        .map_err(Error::InvalidPattern)?;

    let mut keys = shrine
        .keys()
        .into_iter()
        .filter(|k| regex.as_ref().map(|r| r.is_match(k)).unwrap_or(true))
        .collect::<Vec<String>>();
    keys.sort_unstable();

    println!("Shrine `{}`", path.as_ref().display());

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
                String::from_utf8_lossy(
                    shrine
                        .get(&format!(".{key}"))?
                        .value()
                        .expose_secret_as_bytes()
                )
            )
        }
    }

    Ok(())
}
