use crate::agent::{ErrorResponse, GetSecretsRequest, SetPasswordRequest, SetSecretRequest};
use crate::bytes::SecretBytes;
use crate::shrine::{Key, Mode, Secret};
use crate::utils::read_password_from_tty;
use crate::Error;
use hyper::body::HttpBody;
use hyper::{Body, Method, Request};
use hyperlocal::{UnixClientExt, Uri};
use serde::{Deserialize, Serialize};
use std::env;
use std::path::PathBuf;
use tokio::runtime::Runtime;
use uuid::Uuid;

pub fn is_running() -> bool {
    rt().block_on(get::<u32>("/pid")).is_ok()
}

pub fn pid() -> Option<u32> {
    rt().block_on(get::<u32>("/pid")).ok()
}

pub fn stop() -> Result<(), Error> {
    rt().block_on(delete::<Empty>("/")).map(|_| ())
}

pub fn get_key(path: &str, key: &str) -> Result<Secret, Error> {
    rt().block_on(get::<Secret>(&format!(
        "/keys/{}/{}",
        urlencoding::encode(path),
        urlencoding::encode(key)
    )))
}

pub fn set_key(path: &str, key: &str, value: Vec<u8>, mode: Mode) -> Result<(), Error> {
    rt().block_on(put::<_, Empty>(
        &format!(
            "/keys/{}/{}",
            urlencoding::encode(path),
            urlencoding::encode(key)
        ),
        &SetSecretRequest {
            secret: SecretBytes::from(value.as_slice()),
            mode,
        },
    ))
    .map(|_| ())
}

pub fn delete_key(path: &str, key: &str) -> Result<Vec<Secret>, Error> {
    rt().block_on(delete::<Vec<Secret>>(&format!(
        "/keys/{}/{}",
        urlencoding::encode(path),
        urlencoding::encode(key)
    )))
}

pub fn ls(path: &str, regexp: Option<&str>) -> Result<Vec<Key>, Error> {
    rt().block_on(get::<Vec<Key>>(&format!(
        "/keys/{}?{}",
        urlencoding::encode(path),
        serde_qs::to_string(&GetSecretsRequest {
            regexp: regexp.map(|s| s.to_string())
        })
        .unwrap()
    )))
}

pub fn clear_passwords() -> Result<(), Error> {
    rt().block_on(delete::<Empty>("/passwords")).map(|_| ())
}

fn rt() -> Runtime {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
}

enum Response<T> {
    Payload(T),
    Uuid(Uuid),
}

struct Empty {}

trait DoDeserialize {
    fn do_deserialize(data: &[u8]) -> serde_json::error::Result<Self>
    where
        Self: Sized;
}

impl DoDeserialize for Empty {
    fn do_deserialize(_: &[u8]) -> serde_json::error::Result<Empty> {
        Ok(Self {})
    }
}

impl<T> DoDeserialize for T
where
    T: for<'d> Deserialize<'d> + Sized,
{
    fn do_deserialize(data: &[u8]) -> serde_json::error::Result<T> {
        serde_json::from_slice(data)
    }
}

async fn get<T>(uri: &str) -> Result<T, Error>
where
    T: DoDeserialize,
{
    let socket = match env::var("XDG_RUNTIME_DIR") {
        Ok(dir) => format!("{}/shrine.socket", dir),
        Err(_) => return Err(Error::Agent("XDG_RUNTIME_DIR not set".to_string())),
    };

    loop {
        let request = Request::builder()
            .method(Method::GET)
            .uri(Uri::new(&socket, uri))
            .body(Default::default())
            .unwrap();

        match execute::<T>(request).await? {
            Response::Payload(payload) => return Ok(payload),
            Response::Uuid(uuid) => {
                let password = read_password_from_tty();
                let pwd_request = Request::builder()
                    .method(Method::PUT)
                    .header("content-type", "application/json")
                    .uri(Uri::new(&socket, "/passwords"))
                    .body(Body::from(
                        serde_json::to_string(&SetPasswordRequest { uuid, password })
                            .expect("could not serialize body"),
                    ))
                    .expect("could not prepare request");
                execute::<Empty>(pwd_request).await?;
            }
        }
    }
}

async fn put<P, T>(uri: &str, payload: &P) -> Result<T, Error>
where
    P: Serialize,
    T: DoDeserialize,
{
    let socket = match env::var("XDG_RUNTIME_DIR") {
        Ok(dir) => format!("{}/shrine.socket", dir),
        Err(_) => return Err(Error::Agent("XDG_RUNTIME_DIR not set".to_string())),
    };

    loop {
        let request = Request::builder()
            .method(Method::PUT)
            .header("content-type", "application/json")
            .uri(Uri::new(&socket, uri))
            .body(Body::from(
                serde_json::to_string(payload).expect("could not serialize body"),
            ))
            .unwrap();

        match execute::<T>(request).await? {
            Response::Payload(payload) => return Ok(payload),
            Response::Uuid(uuid) => {
                let password = read_password_from_tty();
                let pwd_request = Request::builder()
                    .method(Method::PUT)
                    .header("content-type", "application/json")
                    .uri(Uri::new(&socket, "/passwords"))
                    .body(Body::from(
                        serde_json::to_string(&SetPasswordRequest { uuid, password })
                            .expect("could not serialize body"),
                    ))
                    .expect("could not prepare request");
                execute::<Empty>(pwd_request).await?;
            }
        }
    }
}

async fn delete<T>(uri: &str) -> Result<T, Error>
where
    T: DoDeserialize,
{
    let socket = match env::var("XDG_RUNTIME_DIR") {
        Ok(dir) => format!("{}/shrine.socket", dir),
        Err(_) => return Err(Error::Agent("XDG_RUNTIME_DIR not set".to_string())),
    };

    loop {
        let request = Request::builder()
            .method(Method::DELETE)
            .uri(Uri::new(&socket, uri))
            .body(Default::default())
            .unwrap();

        match execute::<T>(request).await? {
            Response::Payload(payload) => return Ok(payload),
            Response::Uuid(uuid) => {
                let password = read_password_from_tty();
                let pwd_request = Request::builder()
                    .method(Method::PUT)
                    .header("content-type", "application/json")
                    .uri(Uri::new(&socket, "/passwords"))
                    .body(Body::from(
                        serde_json::to_string(&SetPasswordRequest { uuid, password })
                            .expect("could not serialize body"),
                    ))
                    .expect("could not prepare request");
                execute::<Empty>(pwd_request).await?;
            }
        }
    }
}

async fn execute<T>(request: Request<Body>) -> Result<Response<T>, Error>
where
    T: DoDeserialize,
{
    let client = hyper::Client::unix();
    let mut response = client
        .request(request)
        .await
        .map_err(|_| Error::Agent("communication problem".to_string()))?;
    let mut payload = Vec::<u8>::new();
    while let Some(data) = response.data().await {
        let data = data.map_err(|_| Error::Agent("could not get response data".to_string()))?;
        payload.extend(data);
    }

    if response.status().is_success() {
        return T::do_deserialize(&payload)
            .map_err(|_| Error::Agent("invalid response data".to_string()))
            .map(|s| Response::Payload(s));
    }

    match serde_json::from_slice::<ErrorResponse>(&payload).map_err(|_| {
        Error::Agent(format!(
            "invalid error data: {:?}",
            String::from_utf8(payload)
        ))
    })? {
        ErrorResponse::FileNotFound(file) => Err(Error::FileNotFound(PathBuf::from(file))),
        ErrorResponse::Unauthorized(uuid) => Ok(Response::Uuid(uuid)),
        ErrorResponse::Forbidden(uuid) => Ok(Response::Uuid(uuid)),
        ErrorResponse::KeyNotFound { key, .. } => Err(Error::KeyNotFound(key)),
        ErrorResponse::Regex(e) => Err(Error::InvalidPattern(regex::Error::Syntax(e))),
        _ => Err(Error::Agent("unknown error".to_string())),
    }
}
