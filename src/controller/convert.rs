use crate::shrine::encryption::EncryptionAlgorithm;
use crate::shrine::local::LocalShrine;
use crate::shrine::{OpenShrine, QueryClosed, QueryOpen};
use crate::utils::read_password;
use crate::values::password::ShrinePassword;
use crate::Error;
use std::path::Path;

pub fn convert<P>(
    shrine: OpenShrine,
    change_password: bool,
    new_password: Option<ShrinePassword>,
    encryption: Option<EncryptionAlgorithm>,
    path: P,
) -> Result<(), Error>
where
    P: AsRef<Path>,
{
    let change_password = change_password || new_password.is_some();
    if !change_password && encryption.is_none() {
        return Ok(());
    }

    let new_shrine = LocalShrine::new();
    let mut new_shrine = match encryption {
        Some(EncryptionAlgorithm::Plain) => OpenShrine::LocalClear(new_shrine.into_clear()),
        _ => {
            let uuid = new_shrine.uuid();

            let password = match &new_password {
                None => read_password(uuid).expose_secret().to_string(),
                Some(password) => password.expose_secret().to_string(),
            };

            OpenShrine::LocalAes(new_shrine.set_password(password))
        }
    };

    shrine.mv(&mut new_shrine);

    new_shrine.close()?.write_file(&path)?;

    // todo git

    Ok(())
}
