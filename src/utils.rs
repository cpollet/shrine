use crate::shrine::{Closed, Shrine, ShrinePassword};
use crate::Error;
use csv::ReaderBuilder;
use serde::Deserialize;
use std::env;
use std::ffi::OsString;
use std::ops::BitAnd;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};

static FILE_PERMISSIONS_MASK: u32 = 0o777;
static VALID_FILE_PERMISSION: u32 = 0o600;

#[derive(Debug, Deserialize, PartialEq)]
struct Row {
    uuid: String,
    password: String,
}

pub fn read_password(shrine_file: &Shrine<Closed>) -> ShrinePassword {
    if !shrine_file.requires_password() {
        return ShrinePassword::from("");
    }

    // https://specifications.freedesktop.org/basedir-spec/latest/ar01s03.html
    let config = env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| {
            env::var_os("HOME").map(PathBuf::from).map(|mut p| {
                p.push(OsString::from(".config"));
                p
            })
        });

    if let Some(mut config) = config {
        config.push("shrine");
        config.push("passwords");

        let password_file = Path::new(&config);
        if password_file.exists() && password_file.is_file() {
            if let Ok(mode) = password_file.metadata().map(|m| m.mode()) {
                let actual_permission = mode.bitand(FILE_PERMISSIONS_MASK);
                if actual_permission != VALID_FILE_PERMISSION {
                    eprintln!(
                        "Could not read password from `{}`: invalid permissions. Got 0{:o}, expected 0{:o}",
                        password_file.display(),
                        actual_permission,
                        VALID_FILE_PERMISSION
                    );
                    return read_password_from_tty();
                }
            }

            if let Ok(mut csv) = ReaderBuilder::new()
                .has_headers(false)
                .delimiter(b'=')
                .from_path(password_file)
            {
                let csv = csv.deserialize::<Row>();
                for row in csv {
                    if let Ok(row) = row {
                        if row.uuid == shrine_file.uuid().to_string() {
                            return ShrinePassword::from(row.password);
                        }
                    } else {
                        eprintln!(
                            "Could not read password from `{}`: invalid format",
                            password_file.display(),
                        );
                        return read_password_from_tty();
                    }
                }
            } else {
                eprintln!(
                    "Could not read password from `{}`: invalid format",
                    password_file.display(),
                );
                return read_password_from_tty();
            }
        }
    }

    read_password_from_tty()
}

pub fn read_new_password(shrine_file: &Shrine) -> Result<ShrinePassword, Error> {
    if shrine_file.requires_password() {
        let password1 = rpassword::prompt_password("Enter new shrine password: ").unwrap();
        let password2 = rpassword::prompt_password("Enter new shrine password (again): ").unwrap();
        if password1 != password2 {
            return Err(Error::InvalidPassword);
        }
        Ok(ShrinePassword::from(password1))
    } else {
        Ok(ShrinePassword::from(""))
    }
}

pub fn read_password_from_tty() -> ShrinePassword {
    ShrinePassword::from(rpassword::prompt_password("Enter shrine password: ").unwrap())
}
