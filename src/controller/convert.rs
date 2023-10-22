use crate::shrine::encryption::EncryptionAlgorithm;
use crate::shrine::local::LocalShrine;
use crate::shrine::{ClosedShrine, OpenShrine};
use crate::utils::read_password;
use crate::values::password::ShrinePassword;
use crate::{format, Error};
use std::path::Path;

pub fn convert<P, L>(
    shrine: OpenShrine<L>,
    change_password: bool,
    new_password: Option<ShrinePassword>,
    encryption: Option<EncryptionAlgorithm>,
    path: P,
) -> Result<(), Error>
where
    P: AsRef<Path>,
{
    let default_version = format::default().lock().unwrap().version();

    let change_password = change_password || new_password.is_some();

    let latest_version = match &shrine {
        OpenShrine::LocalClear(s) => s.version() == default_version,
        OpenShrine::LocalAes(s) => s.version() == default_version,
        OpenShrine::Remote(_) => true,
    };

    if !change_password && encryption.is_none() && latest_version {
        return Ok(());
    }

    let new_shrine = LocalShrine::default().with_path(path.as_ref().to_path_buf());
    let mut new_shrine = match encryption {
        Some(EncryptionAlgorithm::Plain) => OpenShrine::LocalClear(new_shrine.into_clear()),
        _ => {
            let uuid = new_shrine.uuid();

            let password = match new_password {
                None => read_password(uuid),
                Some(password) => password,
            };

            OpenShrine::LocalAes(new_shrine.set_password(password))
        }
    };

    shrine.mv(&mut new_shrine);

    match new_shrine.close()? {
        ClosedShrine::LocalClear(s) => s.write_file()?,
        ClosedShrine::LocalAes(s) => s.write_file()?,
        ClosedShrine::Remote(_) => {}
    }

    // todo git

    Ok(())
}
