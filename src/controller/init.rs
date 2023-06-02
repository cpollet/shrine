use crate::io::save_shrine_file;
use crate::shrine::Shrine;
use crate::shrine_file::ShrineFileBuilder;
use crate::{Error, SHRINE_FILENAME};

use std::path::Path;

use secrecy::Secret;
use std::string::ToString;

pub fn init(force: bool) -> Result<(), Error> {
    if !force && Path::new(SHRINE_FILENAME).exists() {
        return Err(Error::FileAlreadyExists(SHRINE_FILENAME.to_string()));
    }

    let mut shrine_file = ShrineFileBuilder::new().build();

    let password = if shrine_file.requires_password() {
        let password1 = rpassword::prompt_password("Enter shrine password: ").unwrap();
        let password2 = rpassword::prompt_password("Enter shrine password (again): ").unwrap();
        if password1 != password2 {
            return Err(Error::InvalidPassword);
        }
        Secret::new(password1)
    } else {
        Secret::new("".to_string())
    };

    shrine_file
        .wrap(Shrine::default(), &password)
        .map_err(|e| Error::Update(e.to_string()))?;

    save_shrine_file(&shrine_file).map_err(Error::WriteFile)
}
