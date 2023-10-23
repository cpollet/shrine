use crate::shrine::OpenShrine;
use crate::values::bytes::SecretBytes;
use crate::values::secret::Mode;
use crate::Error;
use chrono::Local;
use git2::{Commit, ErrorClass, ErrorCode, ObjectType, RepositoryInitOptions, Signature, Time};
use std::path::{Path, PathBuf};
use std::str::FromStr;

struct Configuration {
    enabled: bool,
    commit_auto: bool,
    push_auto: bool,
}

impl Configuration {
    fn read<L>(shrine: &OpenShrine<L>) -> Configuration {
        Self {
            enabled: shrine
                .get(".git.enabled")
                .map(|s| s.value().expose_secret_as_bytes())
                .map(|s| String::from_utf8(Vec::from(s)).unwrap())
                .map(|s| bool::from_str(&s).unwrap_or_default())
                .unwrap_or_default(),
            commit_auto: shrine
                .get(".git.commit.auto")
                .map(|s| s.value().expose_secret_as_bytes())
                .map(|s| String::from_utf8(Vec::from(s)).unwrap())
                .map(|s| bool::from_str(&s).unwrap_or_default())
                .unwrap_or_default(),
            push_auto: shrine
                .get(".git.push.auto")
                .map(|s| s.value().expose_secret_as_bytes())
                .map(|s| String::from_utf8(Vec::from(s)).unwrap())
                .map(|s| bool::from_str(&s).unwrap_or_default())
                .unwrap_or_default(),
        }
    }

    fn write<L>(&self, shrine: &mut OpenShrine<L>) {
        shrine
            .set(
                ".git.enabled",
                SecretBytes::from(self.enabled.to_string().as_bytes()),
                Mode::Text,
            )
            .expect("Could not write .git.enabled");
        shrine
            .set(
                ".git.commit.auto",
                SecretBytes::from(self.commit_auto.to_string().as_bytes()),
                Mode::Text,
            )
            .expect("Could not write .git.commit.auto");
        shrine
            .set(
                ".git.push.auto",
                SecretBytes::from(self.push_auto.to_string().as_bytes()),
                Mode::Text,
            )
            .expect("Could not write .git.push.auto");
    }
}

impl Default for Configuration {
    fn default() -> Self {
        Self {
            enabled: true,
            commit_auto: true,
            push_auto: false,
        }
    }
}

pub struct Repository<State = Closed> {
    path: PathBuf,
    configuration: Configuration,
    state: State,
}

pub struct Closed;

pub struct Open {
    repository: git2::Repository,
}

impl Repository {
    pub fn new(shrine: &OpenShrine<PathBuf>) -> Option<Self> {
        if let OpenShrine::Remote(_) = shrine {
            return None;
        }

        let mut path = shrine.path().to_path_buf();
        path.pop();

        if let Ok(enabled) = shrine
            .get(".git.enabled")
            .map(|s| String::from_utf8(Vec::from(s.value().expose_secret_as_bytes())).unwrap())
        {
            if enabled == "true" {
                return Some(Repository {
                    path,
                    configuration: Configuration::read(shrine),
                    state: Closed,
                });
            }
        }

        None
    }
}

impl<State> Repository<State> {
    pub fn path(&self) -> &Path {
        self.path.as_path()
    }

    fn signature<'a>() -> Result<Signature<'a>, Error> {
        let now = Local::now();
        let username = whoami::username();
        let hostname = whoami::hostname();

        Signature::new(
            &username,
            &format!("{}@{}", username, hostname),
            &Time::new(now.timestamp(), now.offset().local_minus_utc() / 60),
        )
        .map_err(Error::Git)
    }

    pub fn commit_auto(&self) -> bool {
        self.configuration.commit_auto
    }
}

impl Repository<Closed> {
    pub fn open(self) -> Result<Repository<Open>, Error> {
        let mut git_folder = self.path.clone();
        git_folder.push(".git");

        let repository = if git_folder.exists() {
            git2::Repository::open(&self.path)?
        } else {
            let mut init_opts = RepositoryInitOptions::new();
            init_opts.no_reinit(true);
            init_opts.mkdir(false);
            init_opts.mkpath(false);
            init_opts.external_template(false);

            git2::Repository::init_opts(&self.path, &init_opts)?
        };

        Ok(Repository {
            path: self.path,
            configuration: self.configuration,
            state: Open { repository },
        })
    }
}

impl Repository<Open> {
    pub fn create_commit(&self, message: &str) -> Result<String, Error> {
        let mut index = self.state.repository.index()?;
        index.add_path(Path::new("shrine"))?; // todo custom filename

        index.write()?;
        let oid = index.write_tree()?;
        let tree = self.state.repository.find_tree(oid)?;

        let signature = Self::signature()?;

        self.state
            .repository
            .commit(
                Some("HEAD"),
                &signature,
                &signature,
                message,
                &tree,
                self.find_last_commit()?
                    .as_ref()
                    .into_iter()
                    .collect::<Vec<&Commit>>()
                    .as_slice(),
            )
            .map_err(Error::Git)
            .map(|c| c.to_string())
    }

    fn find_last_commit(&self) -> Result<Option<Commit>, Error> {
        let head = match self.state.repository.head() {
            Ok(head) => head,
            Err(_) => return Ok(None),
        };

        let obj = head.resolve()?.peel(ObjectType::Commit)?;
        let commit = obj.into_commit().map_err(|_| {
            git2::Error::new(
                ErrorCode::NotFound,
                ErrorClass::Object,
                "Commit does not exist",
            )
        })?;

        Ok(Some(commit))
    }
}

pub fn write_configuration<L>(shrine: &mut OpenShrine<L>) {
    Configuration::default().write(shrine);
}
