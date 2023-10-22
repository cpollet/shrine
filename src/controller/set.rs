use crate::shrine::{ClosedShrine, OpenShrine};
use crate::utils::Input;
use crate::Error;
use std::path::PathBuf;

pub fn set(mut shrine: OpenShrine<PathBuf>, key: &str, input: Input) -> Result<(), Error> {
    if key.starts_with('.') {
        return Err(Error::KeyNotFound(key.to_string()));
    }

    let (value, mode) = input.get(&format!("Enter `{}` value: ", key))?;

    shrine.set(key, value, mode)?;

    let repository = shrine.repository();

    let shrine = shrine.close()?;

    match shrine {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shrine::local::{LoadedShrine, LocalShrine};
    use crate::values::bytes::SecretBytes;
    use crate::values::secret::Mode;
    use tempfile::tempdir;

    #[test]
    fn set() {
        let folder = tempdir().unwrap();
        let mut path = folder.into_path();
        path.push("shrine");

        let shrine =
            OpenShrine::LocalClear(LocalShrine::default().into_clear().with_path(path.clone()));

        super::set(
            shrine,
            "key",
            Input {
                read_from_stdin: false,
                mode: Mode::Text,
                value: Some(SecretBytes::from("secret")),
            },
        )
        .unwrap();

        let shrine = match LoadedShrine::try_from_path(path).unwrap() {
            LoadedShrine::Clear(s) => s,
            _ => panic!("Expected Clear shrine, got AES one"),
        }
        .open()
        .unwrap();

        let secret = shrine.get("key").unwrap();
        assert_eq!(secret.value().expose_secret_as_bytes(), "secret".as_bytes());
    }
}
