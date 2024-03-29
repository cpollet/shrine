use crate::shrine::Shrine;
use crate::{shrine, Error, SHRINE_FILENAME};
use chrono::Local;
use git2::{Commit, ErrorClass, ErrorCode, ObjectType, RepositoryInitOptions, Signature, Time};
use std::path::Path;
use std::str::FromStr;

struct Configuration {
    enabled: bool,
    commit_auto: bool,
    push_auto: bool,
}

impl Configuration {
    fn read(shrine: &Shrine<shrine::Open>) -> Configuration {
        Self {
            enabled: shrine
                .get_private("git.enabled")
                .map(|s| bool::from_str(s).unwrap_or_default())
                .unwrap_or_default(),
            commit_auto: shrine
                .get_private("git.commit.auto")
                .map(|s| bool::from_str(s).unwrap_or_default())
                .unwrap_or_default(),
            push_auto: shrine
                .get_private("git.push.auto")
                .map(|s| bool::from_str(s).unwrap_or_default())
                .unwrap_or_default(),
        }
    }

    fn write(&self, shrine: &mut Shrine<shrine::Open>) {
        shrine.set_private("git.enabled".to_string(), self.enabled.to_string());
        shrine.set_private("git.commit.auto".to_string(), self.commit_auto.to_string());
        shrine.set_private("git.push.auto".to_string(), self.push_auto.to_string());
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

pub struct Repository<P, State = Closed>
where
    P: AsRef<Path>,
{
    path: P,
    configuration: Configuration,
    state: State,
}

pub struct Closed;

pub struct Open {
    repository: git2::Repository,
}

impl<P> Repository<P>
where
    P: AsRef<Path>,
{
    pub fn new(path: P, shrine: &Shrine<shrine::Open>) -> Option<Self>
    where
        P: AsRef<Path>,
    {
        if let Some(enabled) = shrine.get_private("git.enabled") {
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

impl<P, State> Repository<P, State>
where
    P: AsRef<Path>,
{
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

impl<P> Repository<P, Closed>
where
    P: AsRef<Path>,
{
    pub fn open(self) -> Result<Repository<P, Open>, Error>
    where
        P: AsRef<Path>,
    {
        let mut git_folder = self.path.as_ref().to_path_buf();
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

impl<P> Repository<P, Open>
where
    P: AsRef<Path>,
{
    pub fn create_commit(&self, message: &str) -> Result<String, Error> {
        let mut index = self.state.repository.index()?;
        index.add_path(Path::new(SHRINE_FILENAME))?;

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

pub fn write_configuration(shrine: &mut Shrine<shrine::Open>) {
    Configuration::default().write(shrine);
}
