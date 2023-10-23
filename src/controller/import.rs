use crate::Error;
use dotenv_parser::parse_dotenv;

use std::fs::read_to_string;

use crate::shrine::{ClosedShrine, OpenShrine};
use crate::values::bytes::SecretBytes;
use crate::values::secret::Mode;
use std::path::{Path, PathBuf};

// https://crates.io/crates/dotenv-parser
// todo compliant with https://hexdocs.pm/dotenvy/dotenv-file-format.html

pub fn import<P>(
    mut shrine: OpenShrine<PathBuf>,
    file: P,
    prefix: Option<&str>,
) -> Result<(), Error>
where
    P: AsRef<Path>,
{
    let file = Path::new(file.as_ref());
    if !(file.exists() && file.is_file()) {
        return Err(Error::FileNotFound(file.to_path_buf()));
    }

    let content = read_to_string(file).map_err(Error::IoRead)?;

    let secrets = parse_dotenv(&content).map_err(|_| Error::InvalidDotEnv(file.to_path_buf()))?;

    let prefix = prefix.unwrap_or_default();

    for (key, value) in secrets {
        shrine.set(
            &format!("{}{}", prefix, key),
            SecretBytes::from(value.as_bytes()),
            Mode::Text,
        )?
    }

    let repository = shrine.repository();

    match shrine.close()? {
        ClosedShrine::LocalClear(s) => s.write_file()?,
        ClosedShrine::LocalAes(s) => s.write_file()?,
        ClosedShrine::Remote(_) => {}
    }

    if let Some(repository) = repository {
        if repository.commit_auto() {
            repository.open()?.create_commit("Update shrine")?;
        }
    }

    Ok(())
}
