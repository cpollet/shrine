use crate::git::Repository;
use crate::shrine::encryption::EncryptionAlgorithm;
use crate::shrine::local::LocalShrine;
use crate::shrine::{ClosedShrine, OpenShrine};
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

    let shrine = LocalShrine::new();
    let shrine = shrine.with_path(path.as_ref().to_path_buf());
    // shrine.with_serialization_format(SerializationFormat::Json);
    let shrine = match encryption {
        Some(EncryptionAlgorithm::Plain) => OpenShrine::LocalClear(shrine.into_clear()),
        _ => {
            let uuid = shrine.uuid();
            OpenShrine::LocalAes(shrine.set_password(password_provider(uuid)))
        }
    };

    let shrine = if git {
        let mut shrine = shrine;
        git::write_configuration(&mut shrine);
        shrine
    } else {
        shrine
    };

    let mut repo_path = PathBuf::from(path.clone());
    repo_path.pop();
    let repository = Repository::new(&repo_path, &shrine);

    match shrine.close()? {
        ClosedShrine::LocalClear(s) => s.write_file()?,
        ClosedShrine::LocalAes(s) => s.write_file()?,
        ClosedShrine::Remote(_) => panic!("local shrine cannot become a remote shrine"),
    }

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
