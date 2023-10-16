use crate::shrine::{ClosedShrine, OpenShrine, QueryOpen};
use crate::values::secret::Mode;
use crate::Error;
use rpassword::prompt_password;
use std::io::Read;
use std::path::PathBuf;

pub struct Input<'a> {
    pub read_from_stdin: bool,
    pub mode: Mode,
    pub value: Option<&'a str>,
}

pub fn set(mut shrine: OpenShrine<PathBuf>, key: &str, input: Input<'_>) -> Result<(), Error> {
    if key.starts_with('.') {
        return Err(Error::KeyNotFound(key.to_string()));
    }

    let value = if input.read_from_stdin {
        let mut input = Vec::new();
        let stdin = std::io::stdin();
        let mut handle = stdin.lock();
        handle.read_to_end(&mut input).map_err(Error::ReadStdIn)?;
        input
    } else {
        input
            .value
            .map(|v| v.to_string())
            .unwrap_or_else(|| prompt_password(format!("Enter `{}` value: ", key)).unwrap())
            .as_bytes()
            .to_vec()
    };
    let value = value.as_slice();

    shrine.set(key, value, input.mode)?;

    let shrine = shrine.close()?;

    match shrine {
        ClosedShrine::LocalClear(s) => s.write_file()?,
        ClosedShrine::LocalAes(s) => s.write_file()?,
        ClosedShrine::Remote(_) => {}
    }

    // todo git repo

    Ok(())
}

#[cfg(test)]
mod tests {
    // use super::*;
    // use crate::agent::client::mock::MockClient;
    // use crate::shrine::mocks::MockShrineProvider;
    // use crate::shrine::{EncryptionAlgorithm, ShrineBuilder, ShrinePassword};

    // #[test]
    // fn get_direct() {
    //     let mut client = MockClient::default();
    //     client.with_is_running(false);
    //
    //     let mut shrine = ShrineBuilder::new()
    //         .with_encryption_algorithm(EncryptionAlgorithm::Plain)
    //         .build();
    //     shrine.set("key", "secret", Mode::Text).unwrap();
    //     let shrine = shrine.close(&ShrinePassword::default()).unwrap();
    //
    //     let shrine_provider = MockShrineProvider::new(shrine);
    //
    //
    //     set(
    //         client,
    //         shrine_provider.clone(),
    //         "key",
    //         Input {
    //             read_from_stdin: false,
    //             mode: Mode::Text,
    //             value: Some("value"),
    //         },
    //     )
    //     .expect("expected Ok(())");
    //
    //     let shrine = shrine_provider
    //         .load_closed()
    //         .unwrap()
    //         .open(&ShrinePassword::default())
    //         .unwrap();
    //     let secret = shrine.get("key").unwrap();
    //     assert_eq!("value".as_bytes(), secret.value().expose_secret_as_bytes());
    // }

    // #[test]
    // fn set_through_agent() {
    //     let mut client = MockClient::default();
    //     client.with_is_running(true);
    //     client.with_set_key(
    //         "/path/to/shrine",
    //         "key",
    //         "value".as_bytes(),
    //         &Mode::Text,
    //         Ok(()),
    //     );
    //
    //     let shrine_provider = MockShrineProvider::default();
    //
    //     set(
    //         client,
    //         shrine_provider,
    //         "key",
    //         Input {
    //             read_from_stdin: false,
    //             mode: Mode::Text,
    //             value: Some("value"),
    //         },
    //     )
    //     .expect("Expect Ok(())")
    // }
}
