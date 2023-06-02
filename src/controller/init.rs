use crate::io::save_shrine_file;
use crate::shrine::Shrine;
use crate::shrine_file::ShrineFileBuilder;
use crate::{Error, SHRINE_FILENAME};

use std::path::Path;

use std::string::ToString;

pub fn init(force: bool) -> Result<(), Error> {
    if !force && Path::new(SHRINE_FILENAME).exists() {
        return Err(Error::FileAlreadyExists(SHRINE_FILENAME.to_string()));
    }

    let mut shrine_file = ShrineFileBuilder::new().build();

    shrine_file
        .wrap(Shrine::default())
        .map_err(|e| Error::Update(e.to_string()))?;

    save_shrine_file(&shrine_file).map_err(Error::WriteFile)
}
