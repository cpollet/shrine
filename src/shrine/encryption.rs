use crate::encrypt::aes::Aes;
use crate::encrypt::plain::Plain;
use crate::encrypt::EncDec;
use crate::values::password::ShrinePassword;
use borsh::{BorshDeserialize, BorshSerialize};
use std::fmt::{Display, Formatter};

/// The list of encryption algorithms used to encrypt the payload.
#[derive(Default, Debug, Clone, Copy, Eq, PartialEq, BorshSerialize, BorshDeserialize)]
pub enum EncryptionAlgorithm {
    /// AES-GCM-SIV encryption
    #[default]
    Aes,
    /// No encryption
    Plain,
}

impl EncryptionAlgorithm {
    // todo revert to private
    pub fn encryptor<'pwd>(
        &self,
        password: &'pwd ShrinePassword,
        aad: Option<String>,
    ) -> Box<dyn EncDec + 'pwd> {
        match self {
            EncryptionAlgorithm::Aes => {
                // FIXME (#2): use the previous commit hash and repo remote as the AAD
                //  something similar to https://github.com/cpollet/shrine.git#ae9ef36cc813d90a47c13315158f8dc3f87ee81e
                Box::new(Aes::new(password, aad))
            }
            EncryptionAlgorithm::Plain => Box::new(Plain::new()),
        }
    }
}

impl Display for EncryptionAlgorithm {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            EncryptionAlgorithm::Aes => write!(f, "AES-GCM-SIV with 256-bits key"),
            EncryptionAlgorithm::Plain => write!(f, "Not encrypted"),
        }
    }
}
