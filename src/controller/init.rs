use crate::git::Repository;
use crate::shrine::encryption::EncryptionAlgorithm;
use crate::shrine::local::LocalShrine;
use crate::shrine::QueryClosed;
use crate::values::password::ShrinePassword;
use crate::{git, Error};
use std::path::{Path, PathBuf};
use std::string::ToString;
use uuid::Uuid;

pub fn init<P, F>(
    path: P,
    force: bool,
    encryption: Option<EncryptionAlgorithm>,
    git: bool,
    password_provider: F,
) -> Result<(), Error>
where
    P: AsRef<Path> + Clone,
    PathBuf: From<P>,
    F: FnOnce(Uuid) -> ShrinePassword,
{
    if !force && path.as_ref().exists() {
        return Err(Error::FileAlreadyExists(
            path.as_ref().display().to_string(),
        ));
    }

    let mut shrine = LocalShrine::new();
    // shrine.with_serialization_format(SerializationFormat::Json);

    if git {
        git::write_configuration(&mut shrine);
    }
    let mut repo_path = PathBuf::from(path.clone());
    repo_path.pop();
    let repository = Repository::new(&repo_path, &shrine);

    match encryption {
        Some(EncryptionAlgorithm::Plain) => {
            shrine.into_clear().close()?.write_to(&path)?;
        }
        _ => {
            let uuid = shrine.uuid();
            shrine
                .set_password(password_provider(uuid))
                .close()?
                .write_to(&path)?;
        }
    };

    print!("Initialized new shrine in `{}`", path.as_ref().display());

    if let Some(repository) = repository {
        let commit = repository
            .open()
            .and_then(|r| r.create_commit("Initialize shrine"))?;
        print!("; git commit {} in {}", commit, repo_path.display());
    }

    println!();

    Ok(())
}
