use crate::agent::client::Client;
use crate::Error;
use daemonize::Daemonize;
use std::env;
use std::fs::File;

pub fn start<C: Client>(client: &C) -> Result<(), Error> {
    if client.is_running() {
        return Ok(());
    }

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

pub fn stop<C: Client>(client: &C) -> Result<(), Error> {
    if client.is_running() {
        client.stop()
    } else {
        Ok(())
    }
}

pub fn clear_passwords<C: Client>(client: &C) -> Result<(), Error> {
    client.clear_passwords()
}

pub fn status<C: Client>(client: &C) -> Result<(), Error> {
    match client.pid() {
        None => {
            println!("Is running: false");
        }
        Some(pid) => {
            println!("Is running: false");
            println!("PID:        {}", pid);
        }
    }

    Ok(())
}
