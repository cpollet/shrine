use crate::agent::client;
use crate::Error;
use daemonize::Daemonize;
use sig::kill;
use std::fs::File;
use std::io::ErrorKind;
use std::str::FromStr;
use std::{env, fs, io};

pub fn start() -> Result<(), Error> {
    // https://specifications.freedesktop.org/basedir-spec/latest/ar01s03.html
    let runtime_dir = env::var("XDG_RUNTIME_DIR").expect("$XDG_RUNTIME_DIR is not set or is invalid; read https://specifications.freedesktop.org/basedir-spec/latest/ar01s03.html");

    let stdout = File::create(format!("{}/shrine.out", runtime_dir)).unwrap();
    let stderr = File::create(format!("{}/shrine.err", runtime_dir)).unwrap();

    let pidfile = format!("{}/shrine.pid", runtime_dir);
    let socketfile = format!("{}/shrine.socket", runtime_dir);

    let daemonize = Daemonize::new()
        .pid_file(&pidfile)
        .stdout(stdout)
        .stderr(stderr);

    match daemonize.start() {
        Ok(_) => {
            tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .unwrap()
                .block_on(async { crate::agent::server::serve(pidfile, socketfile).await });
        }
        Err(e) => eprintln!("Error, {}", e),
    };

    Ok(())
}

pub fn stop() -> Result<(), Error> {
    let runtime_dir = env::var("XDG_RUNTIME_DIR").expect("$XDG_RUNTIME_DIR is not set or is invalid; read https://specifications.freedesktop.org/basedir-spec/latest/ar01s03.html");
    let pidfile = format!("{}/shrine.pid", runtime_dir);

    let pid = fs::read_to_string(pidfile)
        .map_err(Error::ReadPidFile)
        .and_then(|pid| {
            i32::from_str(pid.trim()).map_err(|_| {
                Error::ReadPidFile(io::Error::new(
                    ErrorKind::InvalidData,
                    "PID in not a number",
                ))
            })
        })?;

    kill!(pid, 2);
    Ok(())
}

pub fn clear_passwords() -> Result<(), Error> {
    client::clear_passwords()
}

pub fn status() -> Result<(), Error> {
    let runtime_dir = env::var("XDG_RUNTIME_DIR").expect("$XDG_RUNTIME_DIR is not set or is invalid; read https://specifications.freedesktop.org/basedir-spec/latest/ar01s03.html");
    let pidfile = format!("{}/shrine.pid", runtime_dir);

    match fs::read_to_string(pidfile) {
        Ok(pid) => println!("PID: {}", pid.trim()),
        Err(_) => println!("No PID file found"),
    };

    println!("Is running: {}", crate::agent::client::is_running());

    Ok(())
}
