use std::path::Path;

use crate::shrine::{OpenShrine, QueryOpen};
use crate::Error;

pub fn rm<P>(mut shrine: OpenShrine, key: &str, path: P) -> Result<(), Error>
where
    P: AsRef<Path>,
{
    if key.starts_with('.') || !shrine.rm(key) {
        return Err(Error::KeyNotFound(key.to_string()));
    }

    shrine.close()?.write_file(&path)?;

    // todo git

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shrine::local::{LoadedShrine, LocalShrine};
    use crate::values::secret::Mode;
    use tempfile::tempdir;

    #[test]
    fn rm() {
        let folder = tempdir().unwrap();
        let mut path = folder.into_path();
        path.push("shrine");

        let mut shrine = OpenShrine::LocalClear(LocalShrine::new().into_clear());
        shrine.set("key", "value".as_bytes(), Mode::Text).unwrap();

        super::rm(shrine, "key", &path).unwrap();

        let shrine = LoadedShrine::try_from_path(&path).unwrap();
        let shrine = match shrine {
            LoadedShrine::Clear(shrine) => {
                OpenShrine::LocalClear(shrine.open().unwrap())
            }
            LoadedShrine::Aes(_) => {
                panic!("Expected Clear shrine, got AES one")
            }
        };

        let err = super::rm(shrine, "key", &path).unwrap_err();
        match err {
           Error::KeyNotFound(key) => {
               assert_eq!(&key, "key")
           }
           e => panic!("Expected Error::KeyNotFound(\"key\"), got {:?}", e)
        }
    }
}
