use crate::io::{load_shrine_file, save_shrine_file};
use crate::Error;

pub fn rm(key: &String) -> Result<(), Error> {
    let shrine_file = load_shrine_file().map_err(Error::ReadFile)?;

    let mut shrine = shrine_file
        .unwrap()
        .map_err(|e| Error::InvalidFile(e.to_string()))?;

    shrine.remove(key.as_ref());

    let mut shrine_file = shrine_file;
    shrine_file
        .wrap(shrine)
        .map_err(|e| Error::Update(e.to_string()))?;

    save_shrine_file(&shrine_file).map_err(Error::WriteFile)
}
