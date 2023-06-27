use crate::agent::{ErrorResponse, GetSecretsRequest, SetPasswordRequest, SetSecretRequest};

use crate::git::Repository;
use crate::shrine::{Closed, Key, Secret, Shrine, ShrinePassword};
use crate::{Error, SHRINE_FILENAME};
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{delete, get, put};
use axum::{Json, Router};
use chrono::{DateTime, Utc};
use hyper::Server;
use hyperlocal::UnixServerExt;
use regex::Regex;
use std::collections::HashMap;
use std::fs::remove_file;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::{mem, process};
use tokio::signal::ctrl_c;
use tokio::signal::unix::{signal, SignalKind};
use tokio::sync::oneshot::{channel, Receiver, Sender};
use tokio_cron_scheduler::{Job, JobScheduler};
use tracing::log::{error, info};
use tracing::Level;
use tracing_subscriber::filter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use uuid::Uuid;

pub async fn serve(pidfile: String, socketfile: String) {
    let filter = filter::Targets::new()
        .with_target("tower_http::trace::on_response", Level::DEBUG)
        .with_target("tower_http::trace::on_request", Level::INFO)
        .with_target("tower_http::trace::make_span", Level::TRACE)
        .with_default(Level::INFO);
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(filter)
        .init();

    let (tx, rx) = channel::<()>();
    let state = AgentState::new(DefaultShrineProvider::default(), tx);

    let mut scheduler = JobScheduler::new().await.unwrap();

    {
        let state = state.clone();
        scheduler
            .add(
                Job::new_repeated(Duration::from_secs(1), move |_uuid, _l| {
                    state.clean_expired_passwords();
                })
                .unwrap(),
            )
            .await
            .unwrap();
    }

    scheduler.set_shutdown_handler(Box::new(|| {
        Box::pin(async move {
            info!("Shut down scheduler");
        })
    }));

    scheduler.start().await.unwrap();

    let app = Router::new()
        .route("/", delete(delete_agent))
        .route("/pid", get(get_pid))
        .route("/passwords", put(put_password))
        .route("/passwords", delete(delete_passwords))
        .route("/keys/:file", get(get_secrets))
        .route("/keys/:file/:key", get(get_key))
        .route("/keys/:file/:key", put(put_key))
        .route("/keys/:file/:key", delete(delete_key))
        .with_state(state);

    if let Ok(x) = Server::bind_unix(&socketfile) {
        x.serve(app.into_make_service())
            .with_graceful_shutdown(shutdown(rx))
            .await
            .unwrap();

        remove_file(pidfile).unwrap();
        remove_file(socketfile).unwrap();
    } else {
        error!("Could not open socket.")
    }

    scheduler.shutdown().await.unwrap();
}

async fn shutdown(shutdown_http_signal_rx: Receiver<()>) {
    let ctrl_c = async {
        ctrl_c().await.expect("failed to install Ctrl+C handler");
    };

    let terminate = async {
        signal(SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    let http_shutdown = async {
        shutdown_http_signal_rx.await.ok();
    };

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
        _ = http_shutdown => {}
    }

    info!("Shut down HTTP server");
}

async fn delete_agent<Provider>(State(state): State<AgentState<Provider>>)
where
    Provider: ShrineProvider,
{
    info!("delete_agent");
    let channel = channel::<()>();

    let mut sig = state.http_shutdown_tx.lock().unwrap();

    let _ = mem::replace(&mut *sig, channel.0).send(());
}

async fn get_pid() -> String {
    info!("get_pid");
    serde_json::to_string(&process::id()).unwrap()
}

async fn put_password<Provider>(
    State(state): State<AgentState<Provider>>,
    Json(set_password_request): Json<SetPasswordRequest>,
) where
    Provider: ShrineProvider,
{
    info!("set_password");
    state.set_password(set_password_request.uuid, set_password_request.password);
}

async fn delete_passwords<Provider>(State(state): State<AgentState<Provider>>)
where
    Provider: ShrineProvider,
{
    info!("delete_passwords");
    state.delete_passwords();
}

async fn get_secrets<Provider>(
    State(state): State<AgentState<Provider>>,
    Path(path): Path<String>,
    Query(params): Query<GetSecretsRequest>,
) -> Response
where
    Provider: ShrineProvider,
{
    info!(
        "get_secrets from file `{}/{}` ({:?})",
        path, SHRINE_FILENAME, params
    );

    let regex = match params
        .regexp
        .as_ref()
        .map(|p| Regex::new(p.as_ref()))
        .transpose()
        .map_err(Error::InvalidPattern)
    {
        Err(e) => return ErrorResponse::Regex(e.to_string()).into(),
        Ok(regex) => regex,
    };

    let shrine = match open_shrine::<Provider>(&state, &path) {
        Ok((shrine, _)) => shrine,
        Err(response) => return response,
    };

    let mut keys = shrine
        .keys()
        .into_iter()
        .filter(|k| regex.as_ref().map(|r| r.is_match(k)).unwrap_or(true))
        .collect::<Vec<String>>();
    keys.sort_unstable();

    let secrets = keys
        .into_iter()
        .map(|k| (shrine.get(&k).expect("must be there"), k))
        .collect::<Vec<(&Secret, String)>>();

    let secrets = secrets
        .into_iter()
        .map(|(s, k)| (Key::from((k, s))))
        .collect::<Vec<Key>>();

    Json(secrets).into_response()
}

async fn get_key<Provider>(
    State(state): State<AgentState<Provider>>,
    Path((path, key)): Path<(String, String)>,
) -> Response
where
    Provider: ShrineProvider,
{
    info!("get_key `{}` from file `{}/{}`", key, path, SHRINE_FILENAME);

    let shrine = match open_shrine::<Provider>(&state, &path) {
        Ok((shrine, _)) => shrine,
        Err(response) => return response,
    };

    match shrine.get(&key) {
        Err(_) => ErrorResponse::KeyNotFound { file: path, key }.into(),
        Ok(secret) => Json(secret).into_response(),
    }
}

fn open_shrine<Provider>(
    state: &AgentState<Provider>,
    path: &str,
) -> Result<(Shrine, ShrinePassword), Response>
where
    Provider: ShrineProvider,
{
    let shrine = match state
        .shrine_provider
        .load_from_path(PathBuf::from_str(path).unwrap())
    {
        Err(Error::FileNotFound(_)) => {
            return Err(ErrorResponse::FileNotFound(path.to_string()).into())
        }
        Err(Error::IoRead(_)) => return Err(ErrorResponse::Read(path.to_string()).into()),
        Err(_) => return Err(ErrorResponse::Io(path.to_string()).into()),
        Ok(shrine) => shrine,
    };

    let uuid = shrine.uuid();

    let shrine_password = if shrine.requires_password() {
        match state.get_password(uuid) {
            None => return Err(ErrorResponse::Unauthorized(uuid).into()),
            Some(p) => p,
        }
    } else {
        ShrinePassword::from("")
    };

    let shrine = match shrine.open(&shrine_password) {
        Err(_) => return Err(ErrorResponse::Forbidden(uuid).into()),
        Ok(shrine) => shrine,
    };

    Ok((shrine, shrine_password))
}

async fn put_key<Provider>(
    State(state): State<AgentState<Provider>>,
    Path((path, key)): Path<(String, String)>,
    Json(request): Json<SetSecretRequest>,
) -> Response
where
    Provider: ShrineProvider,
{
    info!("set_key `{}` on file `{}/{}`", key, path, SHRINE_FILENAME);

    let (mut shrine, shrine_password) = match open_shrine::<Provider>(&state, &path) {
        Ok(v) => v,
        Err(response) => return response,
    };

    let repository = Repository::new(PathBuf::from_str(&path).unwrap(), &shrine);

    match shrine.set(&key, request.secret, request.mode) {
        Ok(_) => {}
        Err(Error::KeyNotFound(key)) => {
            return ErrorResponse::KeyNotFound { file: path, key }.into()
        }
        Err(_) => return ErrorResponse::Write(path).into(),
    }

    let shrine = match shrine.close(&shrine_password) {
        Ok(shrine) => shrine,
        Err(_) => return ErrorResponse::Write(path).into(),
    };

    if state.shrine_provider.save_to_path(&path, shrine).is_err() {
        return ErrorResponse::Write(path).into();
    }

    if let Some(repository) = repository {
        if repository.commit_auto()
            && repository
                .open()
                .and_then(|r| r.create_commit("Update shrine"))
                .is_err()
        {
            return ErrorResponse::Write(path).into();
        }
    }

    Response::builder()
        .status(StatusCode::NO_CONTENT)
        .body(Default::default())
        .unwrap()
}

async fn delete_key<Provider>(
    State(state): State<AgentState<Provider>>,
    Path((path, key)): Path<(String, String)>,
) -> Response
where
    Provider: ShrineProvider,
{
    info!(
        "delete_key `{}` on file `{}/{}`",
        key, path, SHRINE_FILENAME
    );

    let (mut shrine, shrine_password) = match open_shrine::<Provider>(&state, &path) {
        Ok(v) => v,
        Err(response) => return response,
    };

    let repository = Repository::new(PathBuf::from_str(&path).unwrap(), &shrine);

    if !shrine.remove(&key) {
        return ErrorResponse::KeyNotFound { file: path, key }.into();
    }

    let shrine = match shrine.close(&shrine_password) {
        Ok(shrine) => shrine,
        Err(_) => return ErrorResponse::Write(path).into(),
    };
    if state.shrine_provider.save_to_path(&path, shrine).is_err() {
        return ErrorResponse::Write(path).into();
    }

    if let Some(repository) = repository {
        if repository.commit_auto()
            && repository
                .open()
                .and_then(|r| r.create_commit("Update shrine"))
                .is_err()
        {
            return ErrorResponse::Write(path).into();
        }
    }

    Response::builder()
        .status(StatusCode::NO_CONTENT)
        .body(Default::default())
        .unwrap()
}

#[derive(Clone)]
struct AgentState<Provider>
where
    Provider: ShrineProvider,
{
    shrine_provider: Provider,
    http_shutdown_tx: Arc<Mutex<Sender<()>>>,
    passwords: Arc<Mutex<HashMap<Uuid, ATimePassword>>>,
}
type ATimePassword = (DateTime<Utc>, ShrinePassword);

impl<Provider> AgentState<Provider>
where
    Provider: ShrineProvider,
{
    fn new(shrine_provider: Provider, http_shutdown_tx: Sender<()>) -> Self {
        Self {
            shrine_provider,
            http_shutdown_tx: Arc::new(Mutex::new(http_shutdown_tx)),
            passwords: Arc::new(Mutex::new(Default::default())),
        }
    }

    fn set_password(&self, uuid: Uuid, password: ShrinePassword) {
        self.passwords
            .lock()
            .unwrap()
            .insert(uuid, (Utc::now(), password));
    }

    fn delete_passwords(&self) {
        self.passwords.lock().unwrap().clear();
    }

    fn get_password(&self, uuid: Uuid) -> Option<ShrinePassword> {
        let mut passwords = self.passwords.lock().unwrap();
        match passwords.remove(&uuid) {
            None => None,
            Some((_, password)) => {
                passwords.insert(uuid, (Utc::now(), password.clone()));
                Some(password)
            }
        }
    }

    fn clean_expired_passwords(&self) {
        let lowest_barrier = Utc::now() - chrono::Duration::minutes(15);
        self.passwords
            .lock()
            .unwrap()
            .retain(|_, (atime, _)| (*atime).gt(&lowest_barrier));
    }
}

trait ShrineProvider: Clone {
    fn load_from_path<P>(&self, path: P) -> Result<Shrine<Closed>, Error>
    where
        P: AsRef<std::path::Path>;

    fn save_to_path<P>(&self, path: P, shrine: Shrine<Closed>) -> Result<(), Error>
    where
        P: AsRef<std::path::Path>;
}

#[derive(Clone, Default)]
struct DefaultShrineProvider {}

impl ShrineProvider for DefaultShrineProvider {
    fn load_from_path<P>(&self, path: P) -> Result<Shrine<Closed>, Error>
    where
        P: AsRef<std::path::Path>,
    {
        Shrine::from_path(path)
    }

    fn save_to_path<P>(&self, path: P, shrine: Shrine<Closed>) -> Result<(), Error>
    where
        P: AsRef<std::path::Path>,
    {
        shrine.to_path(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bytes::SecretBytes;
    use crate::shrine::{Closed, EncryptionAlgorithm, Mode, ShrineBuilder};
    use axum::body::HttpBody;
    use std::cell::RefCell;
    use std::rc::Rc;

    #[derive(Clone)]
    struct MockShrineProvider {
        shrine: Rc<RefCell<Shrine<Closed>>>,
    }

    impl MockShrineProvider {
        fn new(shrine: Shrine<Closed>) -> Self {
            Self {
                shrine: Rc::new(RefCell::new(shrine)),
            }
        }
    }

    impl ShrineProvider for MockShrineProvider {
        fn load_from_path<P>(&self, _path: P) -> Result<Shrine<Closed>, Error>
        where
            P: AsRef<std::path::Path>,
        {
            Ok(self
                .shrine
                .replace(Shrine::default().close(&ShrinePassword::from("")).unwrap()))
        }

        fn save_to_path<P>(&self, _path: P, shrine: Shrine<Closed>) -> Result<(), Error>
        where
            P: AsRef<std::path::Path>,
        {
            self.shrine.replace(shrine);
            Ok(())
        }
    }

    #[tokio::test]
    async fn get_pid() {
        let pid = super::get_pid().await;
        assert!(pid != "");
    }

    #[tokio::test]
    async fn get_key() {
        let (tx, _) = channel::<()>();

        let shrine = {
            let mut shrine = ShrineBuilder::new()
                .with_encryption_algorithm(EncryptionAlgorithm::Plain)
                .build();
            shrine.set("key", "value", Mode::Text).unwrap();
            shrine.close(&ShrinePassword::from("")).unwrap()
        };

        let state = State(AgentState::new(MockShrineProvider::new(shrine), tx));

        let response =
            super::get_key(state, Path(("fake_path".to_string(), "key".to_string()))).await;

        assert_eq!(response.status(), StatusCode::OK);

        let secret: Secret =
            serde_json::from_slice(response.into_body().data().await.unwrap().unwrap().as_ref())
                .unwrap();

        assert_eq!(secret.value().expose_secret_as_bytes(), "value".as_bytes())
    }

    #[tokio::test]
    async fn get_not_found() {
        let (tx, _) = channel::<()>();

        let shrine = {
            let mut shrine = ShrineBuilder::new()
                .with_encryption_algorithm(EncryptionAlgorithm::Plain)
                .build();
            shrine.set("key", "value", Mode::Text).unwrap();
            shrine.close(&ShrinePassword::from("")).unwrap()
        };

        let response = super::get_key(
            State(AgentState::new(MockShrineProvider::new(shrine), tx)),
            Path(("fake_path".to_string(), "unknown key".to_string())),
        )
        .await;

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn get_password_required() {
        let (tx, _) = channel::<()>();

        let shrine = {
            let mut shrine = ShrineBuilder::new()
                .with_encryption_algorithm(EncryptionAlgorithm::Aes)
                .build();
            shrine.set("key", "value", Mode::Text).unwrap();
            shrine.close(&ShrinePassword::from("")).unwrap()
        };

        let state = State(AgentState::new(MockShrineProvider::new(shrine), tx));

        let response =
            super::get_key(state, Path(("fake_path".to_string(), "key".to_string()))).await;

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn get_password_required_and_set() {
        let (tx, _) = channel::<()>();

        let shrine_password = ShrinePassword::from("password");
        let shrine = {
            let mut shrine = ShrineBuilder::new()
                .with_encryption_algorithm(EncryptionAlgorithm::Aes)
                .build();
            shrine.set("key", "value", Mode::Text).unwrap();
            shrine.close(&shrine_password).unwrap()
        };

        let uuid = shrine.uuid();

        let state = State(AgentState::new(MockShrineProvider::new(shrine), tx));

        state.set_password(uuid, shrine_password);

        let response =
            super::get_key(state, Path(("fake_path".to_string(), "key".to_string()))).await;

        assert_eq!(response.status(), StatusCode::OK);

        let secret: Secret =
            serde_json::from_slice(response.into_body().data().await.unwrap().unwrap().as_ref())
                .unwrap();

        assert_eq!(secret.value().expose_secret_as_bytes(), "value".as_bytes())
    }

    #[tokio::test]
    async fn get_secrets() {
        let (tx, _) = channel::<()>();

        let shrine = {
            let mut shrine = ShrineBuilder::new()
                .with_encryption_algorithm(EncryptionAlgorithm::Plain)
                .build();
            shrine.set("key", "text", Mode::Text).unwrap();
            shrine.set("binkey", "bin", Mode::Binary).unwrap();
            shrine.close(&ShrinePassword::from("")).unwrap()
        };

        let state = State(AgentState::new(MockShrineProvider::new(shrine), tx));

        let response = super::get_secrets(
            state,
            Path("fake_path".to_string()),
            Query(GetSecretsRequest {
                regexp: Some("bin.*".to_string()),
            }),
        )
        .await;

        assert_eq!(response.status(), StatusCode::OK);

        let secrets: Vec<Key> =
            serde_json::from_slice(response.into_body().data().await.unwrap().unwrap().as_ref())
                .unwrap();

        assert_eq!(secrets.len(), 1)
    }

    #[tokio::test]
    async fn put_key() {
        let (tx, _) = channel::<()>();

        let shrine = {
            let mut shrine = ShrineBuilder::new()
                .with_encryption_algorithm(EncryptionAlgorithm::Plain)
                .build();
            shrine.set("key", "value", Mode::Text).unwrap();
            shrine.close(&ShrinePassword::from("")).unwrap()
        };

        let state = State(AgentState::new(MockShrineProvider::new(shrine), tx));

        super::put_key(
            state.clone(),
            Path((String::default(), "key".to_string())),
            Json(SetSecretRequest {
                secret: SecretBytes::from("secret"),
                mode: Mode::Text,
            }),
        )
        .await;

        let value = super::get_key(state, Path((String::default(), "key".to_string()))).await;

        let secret: Secret =
            serde_json::from_slice(value.into_body().data().await.unwrap().unwrap().as_ref())
                .unwrap();

        assert_eq!(secret.value().expose_secret_as_bytes(), "secret".as_bytes())
    }

    #[tokio::test]
    async fn delete_key() {
        let (tx, _) = channel::<()>();

        let shrine = {
            let mut shrine = ShrineBuilder::new()
                .with_encryption_algorithm(EncryptionAlgorithm::Plain)
                .build();
            shrine.set("key", "value", Mode::Text).unwrap();
            shrine.close(&ShrinePassword::from("")).unwrap()
        };

        let state = State(AgentState::new(MockShrineProvider::new(shrine), tx));
        super::delete_key(state.clone(), Path((String::default(), "key".to_string()))).await;

        let value = super::get_key(state, Path((String::default(), "key".to_string()))).await;

        assert_eq!(value.status(), StatusCode::NOT_FOUND);
    }
}
