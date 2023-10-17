use crate::shrine::{ClosedShrine, OpenShrine, QueryOpen};
use crate::utils::Input;
use crate::Error;
use std::path::PathBuf;

pub fn set(mut shrine: OpenShrine<PathBuf>, key: &str, input: Input) -> Result<(), Error> {
    if key.starts_with('.') {
        return Err(Error::KeyNotFound(key.to_string()));
    }

    let (value, mode) = input.get(&format!("Enter `{}` value: ", key))?;

    shrine.set(key, value, mode)?;

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
