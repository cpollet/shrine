use crate::git::Repository;
use crate::shrine::{EncryptionAlgorithm, ShrineBuilder};
use crate::utils::read_new_password;
use crate::{git, Error, SHRINE_FILENAME};
use secrecy::Secret;
use std::path::Path;
use std::path::PathBuf;
use std::string::ToString;

pub fn init(
    path: PathBuf,
    password: Option<Secret<String>>,
    force: bool,
    encryption: Option<EncryptionAlgorithm>,
    git: bool,
) -> Result<(), Error> {
    let mut file = PathBuf::from(&path);
    file.push(SHRINE_FILENAME);

    if !force && Path::new(&file).exists() {
        return Err(Error::FileAlreadyExists(file.display().to_string()));
    }

    let mut shrine_builder = ShrineBuilder::new();

    if let Some(encryption) = encryption {
        shrine_builder = shrine_builder.with_encryption_algorithm(encryption);
    }

    let mut shrine = shrine_builder.build();

    let password = password
        .map(Ok)
        .unwrap_or_else(|| read_new_password(&shrine))?;

    if git {
        git::write_configuration(&mut shrine);
    }

    let repository = Repository::new(path.clone(), &shrine);

    let shrine = shrine.close(&password)?;

    shrine.to_path(&path)?;

    print!("Initialized new shrine in `{}`", file.display());

    if let Some(repository) = repository {
        let commit = repository
            .open()
            .and_then(|r| r.create_commit("Initialize shrine"))?;
        print!("; git commit {}", commit);
    }

    println!();
    Ok(())
}
