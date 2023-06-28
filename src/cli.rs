use clap::{command, Parser, Subcommand, ValueEnum};
use shrine::agent::client::{HttpClient, SocketClient};
use shrine::controller::convert::convert;
use shrine::controller::dump::dump;
use shrine::controller::get::get;
use shrine::controller::import::import;
use shrine::controller::info::{info, Fields};
use shrine::controller::init::init;
use shrine::controller::ls::ls;
use shrine::controller::rm::rm;
use shrine::controller::set;
use shrine::controller::set::set;
#[cfg(unix)]
use shrine::controller::{agent, config, get};
use shrine::shrine::{EncryptionAlgorithm, FilesystemShrineProvider, Mode, ShrinePassword};
use shrine::Error;
use std::io::stdout;
use std::path::PathBuf;
use std::process::ExitCode;
use std::{env, fs};

#[derive(Clone, Parser)]
#[command(author, version, about, long_about = None, arg_required_else_help = true)]
struct Args {
    /// The password to use; if not provided, will be prompted interactively when needed
    #[arg(short = 'P', long)]
    password: Option<String>,
    /// The folder containing the shrine file; default is `SHRINE_PATH` env variable or `.` if not set
    #[arg(short, long)]
    path: Option<PathBuf>,
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Clone, Subcommand)]
enum Commands {
    /// Manage the agent
    #[cfg(unix)]
    Agent {
        #[command(subcommand)]
        command: Option<AgentCommands>,
    },
    /// Initializes a shrine in the current folder
    Init {
        /// Override any existing shrine
        #[arg(long, short)]
        force: bool,
        /// Encryption algorithm to use
        #[arg(long, short)]
        encryption: Option<EncryptionAlgorithms>,
        /// Initialize a git repository to contain the shrine
        #[arg(long, short)]
        git: bool,
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
        /// Read value from stdin
        #[arg(long, short)]
        stdin: bool,
        /// The secret's mode
        #[arg(long, short, default_value = "auto")]
        mode: Modes,
        /// The secret's value; if not set and not read from stdin, will be prompted
        value: Option<String>,
    },
    /// Get a secret's value
    Get {
        /// The secret's key
        key: String,
        /// The output encoding (base64 by defaults for binary secrets)
        #[arg(long, short, default_value = "auto")]
        encoding: Encoding,
    },
    /// Lists all secrets keys
    Ls {
        /// Only lists the key matching the provided pattern
        #[arg(value_name = "REGEX")]
        pattern: Option<String>,
    },
    /// Removes secrets stored in keys matching the provided pattern
    Rm {
        /// The secret's key to remove
        #[arg(value_name = "REGEX")]
        key: String,
    },
    /// Imports secret and their values from environment file
    Import {
        /// The file to import
        file: PathBuf,
        /// Prefix keys with value
        #[arg(long, short)]
        prefix: Option<String>,
    },
    /// Dumps the secrets in a `key=value` format
    Dump {
        /// Only dump the key matching the provided pattern
        #[arg(value_name = "REGEX")]
        pattern: Option<String>,
        /// Include configuration keys
        #[arg(long, short, default_value = "false")]
        config: bool,
    },
    /// Configures the shrine
    Config {
        #[command(subcommand)]
        command: Option<ConfigCommands>,
    },
}

#[derive(Clone, Subcommand)]
#[command(arg_required_else_help = true)]
#[cfg(unix)]
enum AgentCommands {
    /// Starts shrine agent
    Start,
    /// Stops shrine agent
    Stop,
    /// Clear cached passwords
    ClearPasswords,
    /// Returns the status of teh shrine agent
    Status,
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

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Modes {
    /// Consider the secret as binary data when read from stdin; text otherwise
    Auto,
    /// The secret is binary data; in that mode, TTY output is base64 encoded by default
    Binary,
    /// The secret is text data
    Text,
}

impl Modes {
    fn to_mode(self, stdin: bool) -> Mode {
        match self {
            Modes::Auto => {
                if stdin {
                    Mode::Binary
                } else {
                    Mode::Text
                }
            }
            Modes::Binary => Mode::Binary,
            Modes::Text => Mode::Text,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Encoding {
    Auto,
    /// No encoding
    Raw,
    /// Use base64 encoding
    Base64,
}

impl From<&Encoding> for get::Encoding {
    fn from(value: &Encoding) -> Self {
        match value {
            Encoding::Auto => get::Encoding::Auto,
            Encoding::Raw => get::Encoding::Raw,
            Encoding::Base64 => get::Encoding::Base64,
        }
    }
}

#[derive(Clone, Subcommand)]
#[command(arg_required_else_help = true)]
enum ConfigCommands {
    /// Sets a configuration option
    Set {
        /// The configuration key
        key: String,
        /// The configuration value; if not set, will be prompted
        value: Option<String>,
    },
    /// Get a configuration option's value
    Get {
        /// The configuration key
        key: String,
    },
}

#[allow(unused)]
fn main() -> ExitCode {
    reset_signal_pipe_handler();
    match exec(Args::parse()) {
        Ok(_) => ExitCode::from(0),
        Err(e) => {
            eprintln!("{}", e);
            ExitCode::from(1)
        }
    }
}

fn exec(cli: Args) -> Result<(), Error> {
    let password = cli.password.map(ShrinePassword::from);
    let path = cli
        .path
        .unwrap_or_else(|| PathBuf::from(env::var("SHRINE_PATH").unwrap_or(".".to_string())));
    let path = fs::canonicalize(path).unwrap();

    #[cfg(unix)]
    let client = HttpClient::<SocketClient>::new().unwrap();
    #[cfg(not(unix))]
    let client = NoClient::new();

    let shrine_provider = FilesystemShrineProvider::new(path);

    match &cli.command {
        #[cfg(unix)]
        Some(Commands::Agent { command }) => match command {
            Some(AgentCommands::Start) => agent::start(client),
            Some(AgentCommands::Stop) => agent::stop(client),
            Some(AgentCommands::ClearPasswords) => agent::clear_passwords(client),
            Some(AgentCommands::Status) => agent::status(client),
            _ => panic!(),
        },
        Some(Commands::Init {
            force,
            encryption,
            git,
        }) => init(
            shrine_provider,
            password,
            *force,
            encryption.map(|algo| algo.into()),
            *git,
        ),
        Some(Commands::Convert {
            change_password,
            new_password,
            encryption,
        }) => convert(
            shrine_provider,
            password,
            *change_password,
            new_password.as_ref().map(ShrinePassword::from),
            encryption.map(|algo| algo.into()),
        ),
        Some(Commands::Info { field }) => info(shrine_provider, (*field).map(Fields::from)),
        Some(Commands::Set {
            key,
            stdin,
            mode,
            value,
        }) => set(
            client,
            shrine_provider,
            password,
            key,
            set::Input {
                read_from_stdin: *stdin,
                mode: mode.to_mode(*stdin),
                value: value.as_deref(),
            },
        ),
        Some(Commands::Get { key, encoding }) => get(
            client,
            shrine_provider,
            password,
            key,
            encoding.into(),
            &mut stdout(),
        ),
        Some(Commands::Ls { pattern }) => ls(
            client,
            shrine_provider,
            password,
            pattern.as_ref().map(|p| p.as_str()),
            &mut stdout(),
        ),
        Some(Commands::Rm { key }) => rm(client, shrine_provider, password, key),
        Some(Commands::Import { file, prefix }) => {
            import(shrine_provider, password, file, prefix.as_deref())
        }
        Some(Commands::Dump { pattern, config }) => {
            dump(shrine_provider, password, pattern.as_ref(), *config)
        }
        Some(Commands::Config { command }) => match command {
            Some(ConfigCommands::Set { key, value }) => {
                config::set(shrine_provider, password, key, value.as_deref())
            }
            Some(ConfigCommands::Get { key }) => config::get(shrine_provider, password, key),
            _ => panic!(),
        },
        _ => panic!(),
    }
}

pub fn reset_signal_pipe_handler() {
    #[cfg(target_family = "unix")]
    {
        use nix::sys::signal;

        unsafe {
            signal::signal(signal::Signal::SIGPIPE, signal::SigHandler::SigDfl).unwrap();
        }
    }
}
