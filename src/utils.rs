use crate::shrine_file::ShrineFile;
use secrecy::Secret;
use std::str::FromStr;

pub fn read_password(shrine_file: &ShrineFile) -> Secret<String> {
    if shrine_file.requires_password() {
        Secret::new(rpassword::prompt_password("Enter shrine password: ").unwrap())
    } else {
        Secret::from_str("").unwrap()
    }
}
