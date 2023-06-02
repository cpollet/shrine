use crate::io::{load_shrine_file, save_shrine_file};

use crate::Error;
use rpassword::read_password;

use std::io::{stdout, Write};

pub fn set(key: &String, value: Option<&str>) -> Result<(), Error> {
    let shrine_file = load_shrine_file().map_err(Error::ReadFile)?;

    let mut shrine = shrine_file
        .unwrap()
        .map_err(|e| Error::InvalidFile(e.to_string()))?;

    let value = value.map(|v| v.to_string()).unwrap_or_else(|| {
        print!("Secret: ");
        let _ = stdout().flush();
        read_password().unwrap()
    });

    shrine.set(key.to_string(), value.as_bytes());

    let mut shrine_file = shrine_file;
    shrine_file
        .wrap(shrine)
        .map_err(|e| Error::Update(e.to_string()))?;

    save_shrine_file(&shrine_file).map_err(Error::WriteFile)
}
