mod error;
mod maven_types;
use log::*;

use error::Result;
use reqwest::{Response, StatusCode, Url};

use crate::error::MavenError;
use lazy_static::lazy_static;
use maven_types::Metadata;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

#[derive(Debug, Eq, PartialEq)]
pub struct Artifact {
    pub group: String,
    pub name: String,
}

#[derive(Debug, Eq, PartialEq)]
pub struct ResolvedArtifact {
    pub group: String,
    pub name: String,
    pub versions: Vec<String>,
}

pub struct Credentials {
    username: String,
    password: String,
}

pub struct Repository {
    credentials: Credentials,
    url: Url,
}

lazy_static! {
    static ref HTTP_CLIENT: reqwest::Client = reqwest::Client::new();
}

impl Artifact {
    fn new<S: AsRef<str>>(group: S, name: S) -> Self {
        Artifact {
            group: group.as_ref().to_string(),
            name: name.as_ref().to_string(),
        }
    }
}

impl FromStr for Artifact {
    type Err = MavenError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        // TODO: Check for slashed?
        let split: Vec<&str> = s.split(":").collect();
        match split.len() {
            2 => Ok(Artifact::new(split[0], split[1])),
            3 => MavenError::invalid_input(format!(
                "Sorry, parsing of artifact specification with version number is not yet supported. Input: '{}'. Only supported: <GROUP>:<VERSION>",
                s
            )),
            _ => MavenError::invalid_input(format!(
                "Invalid maven artifact specified: '{}'. Expected format <GROUP>:<NAME>",
                s
            )),
        }
    }
}

impl Display for Artifact {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.group, self.name)
    }
}

impl Credentials {
    pub fn new<S: AsRef<str>>(username: S, password: S) -> Self {
        Credentials {
            username: username.as_ref().to_string(),
            password: password.as_ref().to_string(),
        }
    }
}

impl Repository {
    pub fn open_remote(cred: Credentials, url: Url) -> Self {
        Repository {
            credentials: cred,
            url,
        }
    }

    pub async fn resolve(&self, artifact: Artifact) -> Result<Option<ResolvedArtifact>> {
        info!("Resolving artifact: {}", artifact);
        let path = format!(
            "{}/{}/maven-metadata.xml",
            artifact.group.replace('.', "/"),
            artifact.name
        );
        let url = self
            .url
            .join(path.as_str())
            .expect("could not parse artifact url");
        debug!("Requesting metadata from url: {}", url);
        let result = HTTP_CLIENT
            .get(url)
            .basic_auth(&self.credentials.username, Some(&self.credentials.password))
            .send()
            .await
            .expect("Error while downloading metadata");

        match result.status() {
            StatusCode::OK => Ok(Some(Repository::parse_metadata(result).await?)),
            StatusCode::NOT_FOUND => Ok(None),
            StatusCode::UNAUTHORIZED => todo!("Implement authorization error"),
            _ => todo!("Implement other status errors"),
        }
    }

    async fn parse_metadata(response: Response) -> Result<ResolvedArtifact> {
        let metadata = Metadata::from_xml(
            response
                .bytes()
                .await
                .expect("Error while reading response")
                .to_vec()
                .as_slice(),
        );
        Ok(ResolvedArtifact {
            group: metadata.group_id,
            name: metadata.artifact_id,
            versions: metadata.versioning.versions.version.to_vec(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn convert_artifact_from_string() {
        let target: Artifact = "de.kinch:my-artifact".parse().expect("");
        let expected = Artifact::new("de.kinch", "my-artifact");
        assert_eq!(target, expected);
    }

    #[test]
    fn handle_missing_artifact_delimiter() {
        let target: Result<Artifact> = "de.kinch".parse();
        assert_eq!(
            target,
            MavenError::invalid_input(
                "Invalid maven artifact specified: 'de.kinch'. Expected format <GROUP>:<NAME>"
            )
        )
    }

    #[test]
    fn handle_unsupported_artifact_version() {
        let target: Result<Artifact> = "de.kinch:my-artifact:1.2.3".parse();
        assert_eq!(
            target,
            MavenError::invalid_input(
                "Sorry, parsing of artifact specification with version number is not yet supported. Input: 'de.kinch:my-artifact:1.2.3'. Only supported: <GROUP>:<VERSION>"
            )
        )
    }
}
