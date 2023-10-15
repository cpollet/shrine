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
use shrine::controller::{config, get};
use shrine::shrine::encryption::EncryptionAlgorithm;
use shrine::utils::read_password;
use shrine::values::password::ShrinePassword;
use shrine::values::secret::Mode;
use shrine::Error;
use std::env;
use std::io::stdout;
use std::path::PathBuf;
use std::process::ExitCode;

static SHRINE_FILENAME: &str = "shrine";

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

impl From<Encoding> for get::Encoding {
    fn from(value: Encoding) -> Self {
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
    #[cfg(unix)]
    let client = HttpClient::<SocketClient>::new().unwrap();
    #[cfg(not(unix))]
    let client = shrine::agent::client::NoClient {};

    #[cfg(unix)]
    if let Some(Commands::Agent { command }) = cli.command {
        return match command {
            Some(AgentCommands::Start) => shrine::controller::agent::start(client),
            Some(AgentCommands::Stop) => shrine::controller::agent::stop(client),
            Some(AgentCommands::ClearPasswords) => {
                shrine::controller::agent::clear_passwords(client)
            }
            Some(AgentCommands::Status) => shrine::controller::agent::status(client),
            None => panic!(),
        };
    }

    let password = cli.password.clone().map(ShrinePassword::from);
    let path = {
        let mut path = cli
            .path
            .unwrap_or_else(|| PathBuf::from(env::var("SHRINE_PATH").unwrap_or(".".to_string())));
        path.push(SHRINE_FILENAME);
        path
        // todo fs::canonicalize(path).unwrap()
    };

    let shrine = match shrine::shrine::new(Box::new(client), &path) {
        Ok(s) => Ok(s),
        Err(Error::FileNotFound(file)) => {
            if let Some(Commands::Init {
                force,
                encryption,
                git,
            }) = cli.command
            {
                init(
                    file,
                    force,
                    encryption.map(|algo| algo.into()),
                    git,
                    move |uuid| match &password {
                        None => read_password(uuid).expose_secret().to_string(),
                        Some(password) => password.expose_secret().to_string(),
                    },
                )?;

                return Ok(());
            } else {
                Err(Error::FileNotFound(file))
            }
        }
        e => e,
    }?;

    if let Some(Commands::Info { field }) = cli.command {
        return info(&shrine, field.map(Fields::from), &path);
    }

    let shrine = shrine.open({
        let password = password.clone();
        move |uuid| match &password {
            None => read_password(uuid).expose_secret().to_string(),
            Some(password) => password.expose_secret().to_string(),
        }
    })?;

    match cli.command {
        Some(Commands::Init {
            force,
            encryption,
            git,
        }) => init(
            path,
            force,
            encryption.map(|algo| algo.into()),
            git,
            move |uuid| match &password {
                None => read_password(uuid).expose_secret().to_string(),
                Some(password) => password.expose_secret().to_string(),
            },
        ),
        Some(Commands::Convert {
            change_password,
            new_password,
            encryption,
        }) => convert(
            shrine,
            change_password,
            new_password.as_ref().map(ShrinePassword::from),
            encryption.map(|algo| algo.into()),
            &path,
        ),

        Some(Commands::Set {
            key,
            stdin,
            mode,
            value,
        }) => set(
            shrine,
            &key,
            set::Input {
                read_from_stdin: stdin,
                mode: mode.to_mode(stdin),
                value: value.as_deref(),
            },
            &path,
        ),
        Some(Commands::Get { key, encoding }) => get(&shrine, &key, encoding.into(), &mut stdout()),
        Some(Commands::Ls { pattern }) => ls(&shrine, pattern.as_deref(), &mut stdout()),
        Some(Commands::Rm { key }) => rm(shrine, &key, &path),
        Some(Commands::Import { file, prefix }) => import(shrine, &file, prefix.as_deref(), &path),
        Some(Commands::Dump { pattern, config }) => {
            dump(&shrine, pattern.as_deref(), config, &path)
        }
        // Some(Commands::Dump { pattern, config }) => dump(shrine_provider, pattern.as_ref(), config),
        Some(Commands::Config { command }) => match command {
            Some(ConfigCommands::Set { key, value }) => config::set(shrine, &key, value, path),
            Some(ConfigCommands::Get { key: _key }) => todo!(), //config::get(shrine_provider, &key),
            _ => panic!(),
        },
        Some(Commands::Info { .. }) => {
            unreachable!("this case is treated before getting to this match expression")
        }
        Some(Commands::Agent { .. }) => {
            unreachable!("this case is treated before getting to this match expression")
        }
        None => panic!(),
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
