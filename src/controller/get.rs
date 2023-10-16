use crate::shrine::{OpenShrine, QueryOpen};
use crate::values::secret::{Mode, Secret};
use crate::Error;
use atty::Stream;
use base64::Engine;
use std::io::Write;

pub fn get<L, O>(
    shrine: &OpenShrine<L>,
    key: &str,
    encoding: Encoding,
    out: &mut O,
) -> Result<(), Error>
where
    O: Write,
{
    if key.starts_with('.') {
        return Err(Error::KeyNotFound(key.to_string()));
    }

    let secret = shrine.get(key)?;
    let secret = encoding.encode(secret);
    out.write_all(secret.as_slice()).map_err(Error::IoWrite)
}

pub enum Encoding {
    Auto,
    Raw,
    Base64,
}

impl Encoding {
    fn encode(&self, secret: &Secret) -> Vec<u8> {
        match self {
            Encoding::Auto => match secret.mode() {
                Mode::Binary => {
                    if atty::is(Stream::Stdout) {
                        base64::engine::general_purpose::STANDARD
                            .encode(secret.value().expose_secret_as_bytes())
                            .into_bytes()
                    } else {
                        secret.value().expose_secret_as_bytes().to_vec()
                    }
                }
                Mode::Text => secret.value().expose_secret_as_bytes().to_vec(),
            },
            Encoding::Raw => secret.value().expose_secret_as_bytes().to_vec(),
            Encoding::Base64 => base64::engine::general_purpose::STANDARD
                .encode(secret.value().expose_secret_as_bytes())
                .into_bytes(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shrine::local::LocalShrine;

    #[test]
    fn get_auto() {
        let mut shrine = OpenShrine::LocalClear(LocalShrine::new().into_clear());
        shrine
            .set("txt_key", "value".as_bytes(), Mode::Text)
            .unwrap();
        shrine
            .set("bin_key", "value".as_bytes(), Mode::Binary)
            .unwrap();

        let mut out = Vec::<u8>::new();
        get(&shrine, "txt_key", Encoding::Auto, &mut out).unwrap();
        assert_eq!(out.as_slice(), "value".as_bytes());

        let mut out = Vec::<u8>::new();
        get(&shrine, "bin_key", Encoding::Auto, &mut out).unwrap();
        assert_eq!(out.as_slice(), "dmFsdWU=".as_bytes());
    }

    #[test]
    fn get_raw() {
        let mut shrine = OpenShrine::LocalClear(LocalShrine::new().into_clear());
        shrine
            .set("txt_key", "value".as_bytes(), Mode::Text)
            .unwrap();
        shrine
            .set("bin_key", "value".as_bytes(), Mode::Binary)
            .unwrap();

        let mut out = Vec::<u8>::new();
        get(&shrine, "txt_key", Encoding::Raw, &mut out).unwrap();
        assert_eq!(out.as_slice(), "value".as_bytes());

        let mut out = Vec::<u8>::new();
        get(&shrine, "bin_key", Encoding::Raw, &mut out).unwrap();
        assert_eq!(out.as_slice(), "value".as_bytes());
    }

    #[test]
    fn get_base64() {
        let mut shrine = OpenShrine::LocalClear(LocalShrine::new().into_clear());
        shrine
            .set("txt_key", "value".as_bytes(), Mode::Text)
            .unwrap();
        shrine
            .set("bin_key", "value".as_bytes(), Mode::Binary)
            .unwrap();

        let mut out = Vec::<u8>::new();
        get(&shrine, "txt_key", Encoding::Base64, &mut out).unwrap();
        assert_eq!(out.as_slice(), "dmFsdWU=".as_bytes());

        let mut out = Vec::<u8>::new();
        get(&shrine, "bin_key", Encoding::Base64, &mut out).unwrap();
        assert_eq!(out.as_slice(), "dmFsdWU=".as_bytes());
    }
}
