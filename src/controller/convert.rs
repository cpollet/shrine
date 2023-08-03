use crate::git::Repository;
use crate::shrine::{EncryptionAlgorithm, ShrineBuilder};
use crate::shrine::{ShrinePassword, ShrineProvider};
use crate::utils::read_new_password;
use crate::Error;

pub fn convert<P>(
    mut shrine_provider: P,
    change_password: bool,
    new_password: Option<ShrinePassword>,
    encryption_algorithm: Option<EncryptionAlgorithm>,
) -> Result<(), Error>
where
    P: ShrineProvider,
{
    let change_password = change_password || new_password.is_some();
    if !change_password && encryption_algorithm.is_none() {
        return Ok(());
    }

    let mut change_password = change_password;

    let shrine = shrine_provider.load_open()?;

    let shrine_builder =
        ShrineBuilder::new().with_encryption_algorithm(shrine.encryption_algorithm());

    let shrine_builder = match encryption_algorithm {
        Some(a) if shrine.encryption_algorithm() != a => {
            change_password = true;
            shrine_builder.with_encryption_algorithm(a)
        }
        _ => shrine_builder,
    };

    let mut new_shrine = shrine_builder.build();

    shrine.move_to(&mut new_shrine);

    let repository = Repository::new(shrine_provider.path(), &new_shrine);

    if change_password {
        let new_password = if new_shrine.requires_password() {
            new_password.map(Ok).unwrap_or_else(read_new_password)?
        } else {
            ShrinePassword::default()
        };
        shrine_provider.save_closed(new_shrine.close(&new_password)?)?;
    } else {
        shrine_provider.save_open(new_shrine)?;
    }

    if let Some(repository) = repository {
        if repository.commit_auto() {
            repository
                .open()
                .and_then(|r| r.create_commit("Update shrine"))?;
        }
    }

    Ok(())
}
