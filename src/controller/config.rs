use crate::shrine::{ClosedShrine, OpenShrine};
use crate::utils::Input;
use crate::Error;
use std::io::{stdout, Write};
use std::path::PathBuf;

pub fn set(mut shrine: OpenShrine<PathBuf>, key: &str, value: Input) -> Result<(), Error> {
    let (value, mode) = value.get(&format!("Enter `{}` value: ", key))?;

    shrine.set(&format!(".{key}"), value, mode)?;

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

pub fn get<L>(shrine: &OpenShrine<L>, key: &str) -> Result<(), Error> {
    let secret = shrine.get(&format!(".{key}"));
    let _ = stdout().write_all(secret.unwrap().value().expose_secret_as_bytes());
    Ok(())
}
