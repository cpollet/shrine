use clap::{Parser, Subcommand, ValueEnum};
use shrine::controller::get::get;
use shrine::controller::init::init;
use shrine::controller::ls::ls;
use shrine::controller::rm::rm;
use shrine::controller::set::set;

use shrine::Error;

use secrecy::Secret;
use shrine::controller::convert::convert;
use shrine::controller::info::{info, Fields};
use shrine::shrine_file::EncryptionAlgorithm;
use std::process::ExitCode;

#[derive(Clone, Parser)]
#[command(author, version, about, long_about = None, arg_required_else_help = true)]
struct Args {
    /// The password to use; if not provided, will be prompted interactively when needed
    #[arg(short, long)]
    password: Option<String>,
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Clone, Subcommand)]
enum Commands {
    /// Initializes a shrine in the current folder
    Init {
        /// Override any existing shrine
        #[arg(long, short)]
        force: bool,
        /// Encryption algorithm to use
        #[arg(long, short)]
        encryption: Option<EncryptionAlgorithms>,
    },
    /// Convert a shrine to a different format and/or password. This always changes the shrine's
    /// UUID
    Convert {
        /// Change the password; if set and no new password is provided, it will be prompted
        #[arg(long, short, default_value = "false")]
        change_password: bool,
        /// The new password to use; if set, implies password change
        #[arg(long, short)]
        new_password: Option<String>,
        /// New encryption algorithm to use (implies password change)
        #[arg(long, short)]
        encryption: Option<EncryptionAlgorithms>,
    },
    /// Get metadata information about the shrine
    Info {
        /// The field to extract
        #[arg(long, short)]
        field: Option<InfoFields>,
    },
    /// Sets a secret key/value pair
    Set {
        /// The secret's key
        key: String,
        /// The secret's value; if not set, will be prompted
        value: Option<String>,
    },
    /// Get a secret's value
    Get {
        /// The secret's key
        key: String,
    },
    /// Lists all secrets keys
    Ls {
        /// Only lists the key matching the provided pattern
        #[arg(value_name = "REGEX")]
        key: Option<String>,
    },
    /// Removes secrets stored in keys matching the provided pattern
    Rm {
        /// The secret's key to remove
        #[arg(value_name = "REGEX")]
        key: String,
    },
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum EncryptionAlgorithms {
    /// No encryption
    None,
    /// AES-GCM-SIV with 256-bits key
    Aes,
}

impl From<EncryptionAlgorithms> for EncryptionAlgorithm {
    fn from(value: EncryptionAlgorithms) -> Self {
        match value {
            EncryptionAlgorithms::None => EncryptionAlgorithm::Plain,
            EncryptionAlgorithms::Aes => EncryptionAlgorithm::Aes,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum InfoFields {
    Version,
    Uuid,
    EncryptionAlgorithm,
    SerializationFormat,
}

impl From<InfoFields> for Fields {
    fn from(value: InfoFields) -> Self {
        match value {
            InfoFields::Version => Fields::Version,
            InfoFields::Uuid => Fields::Uuid,
            InfoFields::EncryptionAlgorithm => Fields::Encryption,
            InfoFields::SerializationFormat => Fields::Serialization,
        }
    }
}

#[allow(unused)]
fn main() -> ExitCode {
    let cli = Args::parse();

    let password = cli.password.map(Secret::new);
    let result = match &cli.command {
        Some(Commands::Init { force, encryption }) => {
            init(password, *force, encryption.map(|algo| algo.into()))
        }
        Some(Commands::Convert {
            change_password,
            new_password,
            encryption,
        }) => convert(
            password,
            *change_password,
            new_password.clone().map(Secret::new),
            encryption.map(|algo| algo.into()),
        ),
        Some(Commands::Info { field }) => info((*field).map(Fields::from)),
        Some(Commands::Set { key, value }) => set(password, key, value.as_deref()),
        Some(Commands::Get { key }) => get(password, key),
        Some(Commands::Ls { key }) => ls(password, key.as_ref()),
        Some(Commands::Rm { key }) => rm(password, key),
        _ => panic!(),
    };

    match result {
        Ok(_) => ExitCode::from(0),
        Err(Error::FileAlreadyExists(filename)) => {
            eprintln!(
                "Shrine file `{}` already exists; use --force flag to override",
                filename
            );
            ExitCode::from(1)
        }
        Err(Error::WriteFile(e)) => {
            eprintln!("Could not write shrine: {}", e);
            ExitCode::from(1)
        }
        Err(Error::ReadFile(e)) => {
            eprintln!("Could not read shrine: {}", e);
            ExitCode::from(1)
        }
        Err(Error::InvalidFile(e)) => {
            eprintln!("Could not read shrine: {}", e);
            ExitCode::from(1)
        }
        Err(Error::Update(message)) => {
            eprintln!("Could not update shrine: `{}`", message);
            ExitCode::from(1)
        }
        Err(Error::KeyNotFound) => {
            eprintln!("Key does not exist");
            ExitCode::from(1)
        }
        Err(Error::InvalidPattern(e)) => {
            eprintln!("Invalid pattern: {}", e);
            ExitCode::from(1)
        }
        Err(Error::InvalidPassword) => {
            eprintln!("Password invalid");
            ExitCode::from(1)
        }
    }
}
