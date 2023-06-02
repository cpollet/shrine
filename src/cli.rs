use clap::{Parser, Subcommand};
use shrine::controller::get::get;
use shrine::controller::init::init;
use shrine::controller::ls::ls;
use shrine::controller::rm::rm;
use shrine::controller::set::set;

use shrine::Error;

use std::process::ExitCode;

#[derive(Clone, Parser)]
#[command(author, version, about, long_about = None, arg_required_else_help = true)]
struct Args {
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

#[allow(unused)]
fn main() -> ExitCode {
    let cli = Args::parse();

    let result = match &cli.command {
        Some(Commands::Init { force }) => init(*force),
        Some(Commands::Set { key, value }) => set(key, value.as_deref()),
        Some(Commands::Get { key }) => get(key),
        Some(Commands::Ls { key }) => ls(key.as_ref()),
        Some(Commands::Rm { key }) => rm(key),
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
