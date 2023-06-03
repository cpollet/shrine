use crate::io::load_shrine_file;
use crate::utils::read_password;
use crate::Error;
use regex::Regex;
use secrecy::Secret;

pub fn dump(password: Option<Secret<String>>, pattern: Option<&String>) -> Result<(), Error> {
    let regex = pattern
        .map(|p| Regex::new(p.as_ref()))
        .transpose()
        .map_err(Error::InvalidPattern)?;

    let shrine_file = load_shrine_file().map_err(Error::ReadFile)?;

    let password = password.unwrap_or_else(|| read_password(&shrine_file));

    let shrine = shrine_file
        .unwrap(&password)
        .map_err(|e| Error::InvalidFile(e.to_string()))?;

    let mut keys = shrine
        .keys()
        .into_iter()
        .filter(|k| regex.as_ref().map(|r| r.is_match(k)).unwrap_or(true))
        .collect::<Vec<String>>();
    keys.sort_unstable();

    for key in keys.iter() {
        println!(
            "{}={}",
            key,
            String::from_utf8_lossy(shrine.get(key.as_ref()).unwrap().expose_secret_as_bytes())
        )
    }

    Ok(())
}
