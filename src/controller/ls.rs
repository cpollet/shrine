use crate::agent::client::Client;
use crate::shrine::{Key, Shrine, ShrinePassword};
use crate::utils::read_password;
use crate::Error;
use regex::Regex;
use std::io::Write;
use std::path::Path;

pub fn ls<C, P, W>(
    client: &C,
    path: P,
    password: Option<ShrinePassword>,
    pattern: Option<&str>,
    out: &mut W,
) -> Result<(), Error>
where
    C: Client,
    P: AsRef<Path>,
    W: Write,
{
    let keys = if client.is_running() {
        client.ls(path.as_ref().to_str().unwrap(), pattern)?
    } else {
        let regex = pattern
            .map(Regex::new)
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
    use crate::agent::client::mock::MockClient;
    use crate::shrine::Mode;

    #[test]
    fn ls_through_agent() {
        let mut mock = MockClient::default();
        mock.with_is_running(true);
        mock.with_ls(
            "path",
            Some("pattern"),
            Ok(vec![Key {
                key: "pattern".to_string(),
                mode: Mode::Text,
                created_by: "cpollet".to_string(),
                created_at: Default::default(),
                updated_by: None,
                updated_at: None,
            }]),
        );

        let mut out = Vec::<u8>::new();

        ls(&mock, "path", None, Some("pattern"), &mut out).expect("expected Ok(())");

        assert_eq!(
            String::from_utf8(out).unwrap(),
            "total 1\ntxt cpollet 1970-01-01 00:00                   pattern\n".to_string()
        );
    }
}
