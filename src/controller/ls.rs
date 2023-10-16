use crate::shrine::{OpenShrine, QueryOpen};
use crate::values::key::Key;
use crate::Error;
use regex::Regex;
use std::io::Write;

pub fn ls<L, O>(shrine: &OpenShrine<L>, pattern: Option<&str>, out: &mut O) -> Result<(), Error>
where
    O: Write,
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

    let keys = keys
        .into_iter()
        .map(|k| (shrine.get(&k).expect("must be there"), k))
        .map(|(s, k)| Key::from((k, s)))
        .collect::<Vec<Key>>();

    print(out, keys);

    Ok(())
}

fn print<W>(out: &mut W, keys: Vec<Key>)
where
    W: Write,
{
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

    out.write_all(format!("total {}\n", keys.len()).as_bytes())
        .unwrap();

    for key in keys {
        out.write_all(
            format!(
                "{} {:cwidth$} {} {} {:uwidth$} {:10} {:5} {}\n",
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
            .as_bytes(),
        )
        .unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shrine::local::LocalShrine;
    use crate::values::secret::Mode;
    // use crate::agent::client::mock::MockClient;
    // use crate::shrine::mocks::MockShrineProvider;
    // use crate::shrine::{EncryptionAlgorithm, Mode, ShrineBuilder, ShrinePassword};

    #[test]
    fn ls() {
        let mut shrine = OpenShrine::LocalClear(LocalShrine::new().into_clear());
        shrine.set("key", "value".as_bytes(), Mode::Text).unwrap();

        let mut out = Vec::<u8>::new();

        super::ls(&shrine, Some("key"), &mut out).unwrap();

        let out = String::from_utf8(out).unwrap();
        assert!(out.contains(&format!(
            "total 1\ntxt {}@{}",
            whoami::username(),
            whoami::hostname()
        )));
        assert!(out.contains("                   key\n"))
    }
}
