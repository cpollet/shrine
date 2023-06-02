use crate::shrine_file::{FileFormatError, ShrineFile};
use crate::SHRINE_FILENAME;
use std::fs::File;

use std::io::{Error, ErrorKind, Read, Write};
use std::path::Path;

pub fn load_shrine_file() -> Result<ShrineFile, Error> {
    if !Path::new(SHRINE_FILENAME).exists() {
        return Err(Error::new(ErrorKind::NotFound, SHRINE_FILENAME.to_string()));
    }

    let bytes = {
        let mut file = File::open(SHRINE_FILENAME)?;
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes)?;
        bytes
    };

    ShrineFile::from_bytes(&bytes).map_err(|e| Error::new(ErrorKind::InvalidData, e.to_string()))
}

pub fn save_shrine_file(shrine_file: &ShrineFile) -> Result<(), Error> {
    let bytes = match shrine_file.as_bytes() {
        Ok(bytes) => Ok(bytes),
        Err(FileFormatError::Serialization(e)) => Err(e),
        Err(e) => Err(Error::new(ErrorKind::Other, e.to_string())),
    }?;

    File::create(SHRINE_FILENAME)?.write_all(&bytes)
}
