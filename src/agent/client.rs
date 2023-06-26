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

pub trait Client {
    fn is_running(&self) -> bool;

    fn pid(&self) -> Option<u32>;

    fn stop(&self) -> Result<(), Error>;

    fn get_key(&self, path: &str, key: &str) -> Result<Secret, Error>;

    fn set_key(&self, path: &str, key: &str, value: Vec<u8>, mode: Mode) -> Result<(), Error>;

    fn delete_key(&self, path: &str, key: &str) -> Result<Vec<Secret>, Error>;

    fn ls(&self, path: &str, regexp: Option<&str>) -> Result<Vec<Key>, Error>;

    fn clear_passwords(&self) -> Result<(), Error>;
}

#[cfg(unix)]
#[derive(Default)]
pub struct SocketClient {}

#[cfg(unix)]
impl SocketClient {
    pub fn new() -> Self {
        Self::default()
    }

    fn rt() -> Runtime {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
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

            match Self::execute::<T>(request).await? {
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
                    Self::execute::<Empty>(pwd_request).await?;
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

            match Self::execute::<T>(request).await? {
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
                    Self::execute::<Empty>(pwd_request).await?;
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

            match Self::execute::<T>(request).await? {
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
                    Self::execute::<Empty>(pwd_request).await?;
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
}

#[cfg(unix)]
impl Client for SocketClient {
    fn is_running(&self) -> bool {
        Self::rt().block_on(Self::get::<u32>("/pid")).is_ok()
    }

    fn pid(&self) -> Option<u32> {
        Self::rt().block_on(Self::get::<u32>("/pid")).ok()
    }

    fn stop(&self) -> Result<(), Error> {
        Self::rt().block_on(Self::delete::<Empty>("/")).map(|_| ())
    }

    fn get_key(&self, path: &str, key: &str) -> Result<Secret, Error> {
        Self::rt().block_on(Self::get::<Secret>(&format!(
            "/keys/{}/{}",
            urlencoding::encode(path),
            urlencoding::encode(key)
        )))
    }

    fn set_key(&self, path: &str, key: &str, value: Vec<u8>, mode: Mode) -> Result<(), Error> {
        Self::rt()
            .block_on(Self::put::<_, Empty>(
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

    fn delete_key(&self, path: &str, key: &str) -> Result<Vec<Secret>, Error> {
        Self::rt().block_on(Self::delete::<Vec<Secret>>(&format!(
            "/keys/{}/{}",
            urlencoding::encode(path),
            urlencoding::encode(key)
        )))
    }

    fn ls(&self, path: &str, regexp: Option<&str>) -> Result<Vec<Key>, Error> {
        Self::rt().block_on(Self::get::<Vec<Key>>(&format!(
            "/keys/{}?{}",
            urlencoding::encode(path),
            serde_qs::to_string(&GetSecretsRequest {
                regexp: regexp.map(|s| s.to_string())
            })
            .unwrap()
        )))
    }

    fn clear_passwords(&self) -> Result<(), Error> {
        Self::rt()
            .block_on(Self::delete::<Empty>("/passwords"))
            .map(|_| ())
    }
}

#[cfg(not(unix))]
pub struct NoClient {}

#[cfg(not(unix))]
impl Client for NoClient {
    fn is_running(&self) -> bool {
        false
    }

    fn pid(&self) -> Option<u32> {
        unimplemented!()
    }

    fn stop(&self) -> Result<(), Error> {
        unimplemented!()
    }

    fn get_key(&self, _path: &str, _key: &str) -> Result<Secret, Error> {
        unimplemented!()
    }

    fn set_key(&self, _path: &str, _key: &str, _value: Vec<u8>, _mode: Mode) -> Result<(), Error> {
        unimplemented!()
    }

    fn delete_key(&self, _path: &str, _key: &str) -> Result<Vec<Secret>, Error> {
        unimplemented!()
    }

    fn ls(&self, _path: &str, _regexp: Option<&str>) -> Result<Vec<Key>, Error> {
        unimplemented!()
    }

    fn clear_passwords(&self) -> Result<(), Error> {
        unimplemented!()
    }
}

#[cfg(unix)]
enum Response<T> {
    Payload(T),
    Uuid(Uuid),
}

#[cfg(unix)]
struct Empty {}

#[cfg(unix)]
trait DoDeserialize {
    fn do_deserialize(data: &[u8]) -> serde_json::error::Result<Self>
    where
        Self: Sized;
}

#[cfg(unix)]
impl DoDeserialize for Empty {
    fn do_deserialize(_: &[u8]) -> serde_json::error::Result<Empty> {
        Ok(Self {})
    }
}

#[cfg(unix)]
impl<T> DoDeserialize for T
where
    T: for<'d> Deserialize<'d> + Sized,
{
    fn do_deserialize(data: &[u8]) -> serde_json::error::Result<T> {
        serde_json::from_slice(data)
    }
}

#[cfg(test)]
pub mod mock {
    use super::*;
    use std::cell::RefCell;
    use std::collections::HashMap;

    #[derive(Default)]
    pub struct MockClient {
        is_running: bool,
        get_keys: RefCell<HashMap<(String, String), Result<Secret, Error>>>,
        set_keys: RefCell<HashMap<(String, String, Vec<u8>, Mode), Result<(), Error>>>,
        delete_key: RefCell<HashMap<(String, String), Result<Vec<Secret>, Error>>>,
        ls: RefCell<HashMap<(String, Option<String>), Result<Vec<Key>, Error>>>,
    }

    impl MockClient {
        pub fn with_is_running(&mut self, is_running: bool) {
            self.is_running = is_running;
        }

        pub fn with_get_key(&self, path: &str, key: &str, result: Result<Secret, Error>) {
            self.get_keys
                .borrow_mut()
                .insert((path.to_string(), key.to_string()), result);
        }

        pub fn with_set_key(
            &self,
            path: &str,
            key: &str,
            value: &[u8],
            mode: &Mode,
            result: Result<(), Error>,
        ) {
            self.set_keys.borrow_mut().insert(
                (
                    path.to_string(),
                    key.to_string(),
                    value.to_vec(),
                    mode.clone(),
                ),
                result,
            );
        }

        pub fn with_delete_key(&self, path: &str, key: &str, result: Result<Vec<Secret>, Error>) {
            self.delete_key
                .borrow_mut()
                .insert((path.to_string(), key.to_string()), result);
        }

        pub fn with_ls(&self, path: &str, regexp: Option<&str>, result: Result<Vec<Key>, Error>) {
            self.ls
                .borrow_mut()
                .insert((path.to_string(), regexp.map(|r| r.to_string())), result);
        }
    }

    impl Client for MockClient {
        fn is_running(&self) -> bool {
            self.is_running
        }

        fn pid(&self) -> Option<u32> {
            todo!()
        }

        fn stop(&self) -> Result<(), Error> {
            unimplemented!()
        }

        fn get_key(&self, path: &str, key: &str) -> Result<Secret, Error> {
            self.get_keys
                .borrow_mut()
                .remove(&(path.to_string(), key.to_string()))
                .expect(&format!("unexpected get_key(\"{}\", \"{}\")", path, key))
        }

        fn set_key(&self, path: &str, key: &str, value: Vec<u8>, mode: Mode) -> Result<(), Error> {
            self.set_keys
                .borrow_mut()
                .remove(&(path.to_string(), key.to_string(), value.clone(), mode))
                .expect(&format!(
                    "unexpected set_key(\"{}\", \"{}\", {:?}, {})",
                    path, key, value, mode
                ))
        }

        fn delete_key(&self, path: &str, key: &str) -> Result<Vec<Secret>, Error> {
            self.delete_key
                .borrow_mut()
                .remove(&(path.to_string(), key.to_string()))
                .expect(&format!("unexpected delete_key(\"{}\", \"{}\")", path, key))
        }

        fn ls(&self, path: &str, regexp: Option<&str>) -> Result<Vec<Key>, Error> {
            self.ls
                .borrow_mut()
                .remove(&(path.to_string(), regexp.map(|r| r.to_string())))
                .expect(&format!("unexpected ls(\"{}\", \"{:?}\")", path, regexp))
        }

        fn clear_passwords(&self) -> Result<(), Error> {
            todo!()
        }
    }
}
