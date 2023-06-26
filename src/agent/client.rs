use crate::agent::{ErrorResponse, GetSecretsRequest, SetPasswordRequest, SetSecretRequest};
use crate::bytes::SecretBytes;
use crate::shrine::{Key, Mode, Secret};
use crate::utils::read_password_from_tty;
use crate::Error;
use async_recursion::async_recursion;
use hyper::body::HttpBody;
use hyper::{Body, Method, Request};
use hyperlocal::{UnixClientExt, UnixConnector, Uri};
use serde::{Deserialize, Serialize};
use std::env;
use std::path::PathBuf;
use tokio::runtime::Runtime;

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
pub struct HttpClient {
    rt: Runtime,
    socket: String,
    client: hyper::Client<UnixConnector>,
}

#[cfg(unix)]
impl HttpClient {
    pub fn new() -> Result<Self, Error> {
        Ok(Self {
            rt: tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap(),
            socket: env::var("XDG_RUNTIME_DIR")
                .map(|s| format!("{}/shrine.socket", s))
                .map_err(|_| Error::Agent("XDG_RUNTIME_DIR not set".to_string()))?,
            client: hyper::Client::unix(),
        })
    }

    async fn get<T>(&self, uri: &str) -> Result<T, Error>
    where
        T: DoDeserialize,
    {
        self.without_body(uri, Method::GET).await
    }

    async fn delete<T>(&self, uri: &str) -> Result<T, Error>
    where
        T: DoDeserialize,
    {
        self.without_body(uri, Method::DELETE).await
    }

    async fn without_body<T>(&self, uri: &str, method: Method) -> Result<T, Error>
    where
        T: DoDeserialize,
    {
        loop {
            let request = Request::builder()
                .method(method.clone())
                .uri(Uri::new(&self.socket, uri))
                .body(Default::default())
                .unwrap();

            if let Some(payload) = self.execute::<T>(request).await? {
                return Ok(payload);
            }
        }
    }

    async fn put<P, T>(&self, uri: &str, payload: &P) -> Result<T, Error>
    where
        P: Serialize,
        T: DoDeserialize,
    {
        loop {
            let request = Request::builder()
                .method(Method::PUT)
                .header("content-type", "application/json")
                .uri(Uri::new(&self.socket, uri))
                .body(Body::from(
                    serde_json::to_string(payload).expect("could not serialize body"),
                ))
                .unwrap();

            if let Some(payload) = self.execute::<T>(request).await? {
                return Ok(payload);
            }
        }
    }

    #[async_recursion(?Send)]
    async fn execute<T>(&self, request: Request<Body>) -> Result<Option<T>, Error>
    where
        T: DoDeserialize,
    {
        let mut response = self
            .client
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
                .map(|s| Some(s));
        }

        match serde_json::from_slice::<ErrorResponse>(&payload).map_err(|_| {
            Error::Agent(format!(
                "invalid error data: {:?}",
                String::from_utf8(payload)
            ))
        })? {
            ErrorResponse::FileNotFound(file) => Err(Error::FileNotFound(PathBuf::from(file))),
            ErrorResponse::Unauthorized(uuid) | ErrorResponse::Forbidden(uuid) => {
                self.put::<_, Empty>(
                    "/passwords",
                    &SetPasswordRequest {
                        uuid,
                        password: read_password_from_tty(),
                    },
                )
                .await?;
                Ok(None)
            }
            ErrorResponse::KeyNotFound { key, .. } => Err(Error::KeyNotFound(key)),
            ErrorResponse::Regex(e) => Err(Error::InvalidPattern(regex::Error::Syntax(e))),
            _ => Err(Error::Agent("unknown error".to_string())),
        }
    }
}

#[cfg(unix)]
impl Client for HttpClient {
    fn is_running(&self) -> bool {
        self.rt.block_on(self.get::<u32>("/pid")).is_ok()
    }

    fn pid(&self) -> Option<u32> {
        self.rt.block_on(self.get::<u32>("/pid")).ok()
    }

    fn stop(&self) -> Result<(), Error> {
        self.rt.block_on(self.delete::<Empty>("/")).map(|_| ())
    }

    fn get_key(&self, path: &str, key: &str) -> Result<Secret, Error> {
        self.rt.block_on(self.get::<Secret>(&format!(
            "/keys/{}/{}",
            urlencoding::encode(path),
            urlencoding::encode(key)
        )))
    }

    fn set_key(&self, path: &str, key: &str, value: Vec<u8>, mode: Mode) -> Result<(), Error> {
        self.rt
            .block_on(self.put::<_, Empty>(
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
        self.rt.block_on(self.delete::<Vec<Secret>>(&format!(
            "/keys/{}/{}",
            urlencoding::encode(path),
            urlencoding::encode(key)
        )))
    }

    fn ls(&self, path: &str, regexp: Option<&str>) -> Result<Vec<Key>, Error> {
        self.rt.block_on(self.get::<Vec<Key>>(&format!(
            "/keys/{}?{}",
            urlencoding::encode(path),
            serde_qs::to_string(&GetSecretsRequest {
                regexp: regexp.map(|s| s.to_string())
            })
            .unwrap()
        )))
    }

    fn clear_passwords(&self) -> Result<(), Error> {
        self.rt
            .block_on(self.delete::<Empty>("/passwords"))
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
