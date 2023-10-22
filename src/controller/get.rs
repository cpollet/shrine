use crate::shrine::OpenShrine;
use crate::values::secret::{Mode, Secret};
use crate::Error;
use atty::Stream;
use base64::Engine;
use std::io;
use std::io::{stdout, Stdout, Write};

pub fn get<L, W>(
    shrine: &OpenShrine<L>,
    key: &str,
    encoding: Encoding,
    out: &mut Output<W>,
) -> Result<(), Error>
where
    W: Write,
{
    if key.starts_with('.') {
        return Err(Error::KeyNotFound(key.to_string()));
    }

    let secret = shrine.get(key)?;
    let secret = encoding.encode(secret, out);
    out.write_all(secret.as_slice()).map_err(Error::IoWrite)
}

pub struct Output<W: Write> {
    tty: bool,
    out: W,
}

impl Output<Stdout> {
    pub fn stdout() -> Self {
        Self {
            tty: atty::is(Stream::Stdout),
            out: stdout(),
        }
    }
}

impl<O: Write> Output<O> {
    fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        self.out.write_all(buf)
    }
}

pub enum Encoding {
    Auto,
    Raw,
    Base64,
}

impl Encoding {
    fn encode<W>(&self, secret: &Secret, out: &Output<W>) -> Vec<u8>
    where
        W: Write,
    {
        match self {
            Encoding::Auto => match secret.mode() {
                Mode::Binary => {
                    if out.tty {
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
    use crate::values::bytes::SecretBytes;

    #[test]
    fn get_auto() {
        let mut shrine = OpenShrine::LocalClear(LocalShrine::default().into_clear());
        shrine
            .set("txt_key", SecretBytes::from("value".as_bytes()), Mode::Text)
            .unwrap();
        shrine
            .set(
                "bin_key",
                SecretBytes::from("value".as_bytes()),
                Mode::Binary,
            )
            .unwrap();

        let mut out = Output {
            tty: true,
            out: Vec::<u8>::new(),
        };
        get(&shrine, "txt_key", Encoding::Auto, &mut out).unwrap();
        assert_eq!(out.out.as_slice(), "value".as_bytes());

        let mut out = Output {
            tty: true,
            out: Vec::<u8>::new(),
        };
        get(&shrine, "bin_key", Encoding::Auto, &mut out).unwrap();
        assert_eq!(out.out.as_slice(), "dmFsdWU=".as_bytes());
    }

    #[test]
    fn get_raw() {
        let mut shrine = OpenShrine::LocalClear(LocalShrine::default().into_clear());
        shrine
            .set("txt_key", SecretBytes::from("value".as_bytes()), Mode::Text)
            .unwrap();
        shrine
            .set(
                "bin_key",
                SecretBytes::from("value".as_bytes()),
                Mode::Binary,
            )
            .unwrap();

        let mut out = Output {
            tty: true,
            out: Vec::<u8>::new(),
        };
        get(&shrine, "txt_key", Encoding::Raw, &mut out).unwrap();
        assert_eq!(out.out.as_slice(), "value".as_bytes());

        let mut out = Output {
            tty: true,
            out: Vec::<u8>::new(),
        };
        get(&shrine, "bin_key", Encoding::Raw, &mut out).unwrap();
        assert_eq!(out.out.as_slice(), "value".as_bytes());
    }

    #[test]
    fn get_base64() {
        let mut shrine = OpenShrine::LocalClear(LocalShrine::default().into_clear());
        shrine
            .set("txt_key", SecretBytes::from("value".as_bytes()), Mode::Text)
            .unwrap();
        shrine
            .set(
                "bin_key",
                SecretBytes::from("value".as_bytes()),
                Mode::Binary,
            )
            .unwrap();

        let mut out = Output {
            tty: true,
            out: Vec::<u8>::new(),
        };
        get(&shrine, "txt_key", Encoding::Base64, &mut out).unwrap();
        assert_eq!(out.out.as_slice(), "dmFsdWU=".as_bytes());

        let mut out = Output {
            tty: true,
            out: Vec::<u8>::new(),
        };
        get(&shrine, "bin_key", Encoding::Base64, &mut out).unwrap();
        assert_eq!(out.out.as_slice(), "dmFsdWU=".as_bytes());
    }
}
