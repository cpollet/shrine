use crate::shrine::{Closed, Secret, Shrine, ShrinePassword};
use crate::utils::read_password;
use crate::Error;
use regex::Regex;

pub fn ls(
    shrine: Shrine<Closed>,
    password: Option<ShrinePassword>,
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

    let secrets = keys
        .into_iter()
        .map(|k| (shrine.get(&k).expect("must be there"), k))
        .collect::<Vec<(&Secret, String)>>();

    let mut created_by_width = 0;
    let mut updated_by_width = 0;
    for (secret, _) in secrets.iter() {
        if secret.created_by().len() > created_by_width {
            created_by_width = secret.created_by().len();
        }
        if let Some(updated_by) = secret.updated_by() {
            if updated_by.len() > updated_by_width {
                updated_by_width = updated_by.len();
            }
        }
    }

    println!("total {}", secrets.len());
    for (secret, key) in secrets {
        println!(
            "{:cwidth$} {} {} {:uwidth$} {:10} {:5} {}",
            secret.created_by(),
            secret.created_at().format("%Y-%m-%d"),
            secret.created_at().format("%H:%M"),
            secret.updated_by().unwrap_or_default(),
            secret
                .updated_at()
                .map(|dt| dt.format("%Y-%m-%d").to_string())
                .unwrap_or_default(),
            secret
                .updated_at()
                .map(|dt| dt.format("%H:%M").to_string())
                .unwrap_or_default(),
            key,
            cwidth = created_by_width,
            uwidth = updated_by_width
        )
    }

    Ok(())
}
