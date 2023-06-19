use crate::shrine::{Key, Shrine, ShrinePassword};
use crate::utils::read_password;
use crate::{agent, Error};
use regex::Regex;
use std::path::Path;

pub fn ls<P>(
    path: P,
    password: Option<ShrinePassword>,
    pattern: Option<&String>,
) -> Result<(), Error>
where
    P: AsRef<Path>,
{
    let keys = if agent::client::is_running() {
        agent::client::ls(path.as_ref().to_str().unwrap(), pattern.map(|p| p.as_ref()))?
    } else {
        let regex = pattern
            .map(|p| Regex::new(p.as_ref()))
            .transpose()
            .map_err(Error::InvalidPattern)?;

        let shrine = Shrine::from_path(path)?;
        let password = password.unwrap_or_else(|| read_password(&shrine));
        let shrine = shrine.open(&password)?;

        let mut keys = shrine
            .keys()
            .into_iter()
            .filter(|k| regex.as_ref().map(|r| r.is_match(k)).unwrap_or(true))
            .collect::<Vec<String>>();
        keys.sort_unstable();

        keys.into_iter()
            .map(|k| (shrine.get(&k).expect("must be there"), k))
            .map(|(s, k)| Key::from((k, s)))
            .collect::<Vec<Key>>()
    };

    print(keys);

    Ok(())
}

fn print(keys: Vec<Key>) {
    let mut created_by_width = 0;
    let mut updated_by_width = 0;
    for key in keys.iter() {
        if key.created_by.len() > created_by_width {
            created_by_width = key.created_by.len();
        }
        if let Some(updated_by) = key.updated_by.as_ref() {
            if updated_by.len() > updated_by_width {
                updated_by_width = updated_by.len();
            }
        }
    }

    println!("total {}", keys.len());
    for key in keys {
        println!(
            "{} {:cwidth$} {} {} {:uwidth$} {:10} {:5} {}",
            key.mode,
            key.created_by,
            key.created_at.format("%Y-%m-%d"),
            key.created_at.format("%H:%M"),
            key.updated_by.unwrap_or_default(),
            key.updated_at
                .map(|dt| dt.format("%Y-%m-%d").to_string())
                .unwrap_or_default(),
            key.updated_at
                .map(|dt| dt.format("%H:%M").to_string())
                .unwrap_or_default(),
            key.key,
            cwidth = created_by_width,
            uwidth = updated_by_width
        )
    }
}
