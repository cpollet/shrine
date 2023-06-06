use crate::shrine::{Closed, FileFormatError, Shrine};
use crate::SHRINE_FILENAME;
use std::fs::File;

use std::io::{Error, ErrorKind, Read, Write};
use std::path::{Path, PathBuf};

pub fn load_shrine(path: &PathBuf) -> Result<Shrine<Closed>, Error> {
    let mut file = PathBuf::from(path);
    file.push(SHRINE_FILENAME);

    if !Path::new(&file).exists() {
        return Err(Error::new(ErrorKind::NotFound, file.display().to_string()));
    }

    let bytes = {
        let mut file = File::open(&file)?;
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes)?;
        bytes
    };

    Shrine::from_bytes(&bytes).map_err(|e| Error::new(ErrorKind::InvalidData, e.to_string()))
}

pub fn save_shrine(path: &PathBuf, shrine: &Shrine<Closed>) -> Result<PathBuf, Error> {
    let mut file = PathBuf::from(path);
    file.push(SHRINE_FILENAME);

    let bytes = match shrine.as_bytes() {
        Ok(bytes) => Ok(bytes),
        Err(FileFormatError::Serialization(e)) => Err(e),
        Err(e) => Err(Error::new(ErrorKind::Other, e.to_string())),
    }?;

    File::create(&file)?.write_all(&bytes)?;

    Ok(file)
}
