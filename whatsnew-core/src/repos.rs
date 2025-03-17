use anyhow::{Result, anyhow};
use jiff::Timestamp;
use serde::{Deserialize, Serialize};

use crate::gitprovider::{GitClient, GitHubClient};

#[derive(Debug, Serialize, Deserialize)]
pub struct Repo {
    pub owner: String,
    pub name: String,
    pub provider: Option<GitProvider>,
}

impl Repo {
    pub fn new(repostring: &str) -> Result<Self> {
        let provider = Some(GitProvider::GitHub);
        // TODO: Improve validation
        let (owner, name) = repostring
            .split_once('/')
            .ok_or_else(|| anyhow!("Invalid repository format"))?;
        Ok(Self {
            owner: owner.to_string(),
            name: name.to_string(),
            provider,
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum GitProvider {
    GitHub,
    GitLab,
    Codeberg,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CommitsRepo {
    pub repo: Repo,
    pub last_viewed_sha: Option<String>,
    pub last_viewed_time: Option<Timestamp>,
}

impl CommitsRepo {
    pub fn new(repostring: &str) -> Result<Self> {
        let repo = Repo::new(repostring)?;
        Ok(Self {
            repo,
            last_viewed_sha: None,
            last_viewed_time: None,
        })
    }

    pub async fn get_repo_commits_since(&self) -> Result<Vec<CommitInfo>> {
        let client = GitHubClient::new()?;
        let last_viewed_time = self
            .last_viewed_time
            .as_ref()
            .ok_or_else(|| anyhow!("Tried to get new commits without last_viewed_time"))?;
        client
            .get_repo_commits_since(&self.repo.owner, &self.repo.name, last_viewed_time)
            .await
    }

    pub async fn get_repo_commits_since_sha(&self) -> Result<Vec<CommitInfo>> {
        let client = GitHubClient::new()?;
        let last_viewed_sha = self
            .last_viewed_sha
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Tried to get new commits without last_viewed_sha"))?;
        let commits = client
            .get_repo_commits(&self.repo.owner, &self.repo.name)
            .await?;
        commits
            .iter()
            .position(|commit| &commit.sha == last_viewed_sha)
            .map_or_else(
                || Ok(vec![]),
                |viewed_commits_index| Ok(commits[..viewed_commits_index].to_vec()),
            )
    }

    pub async fn get_repo_commits(&self) -> Result<Vec<CommitInfo>> {
        let client = GitHubClient::new()?;
        client
            .get_repo_commits(&self.repo.owner, &self.repo.name)
            .await
    }

    pub fn set_last_viewed_sha(&mut self, sha: &str) {
        self.last_viewed_sha = Some(sha.to_string());
    }

    pub fn set_last_viewed_time(&mut self, time: Timestamp) {
        self.last_viewed_time = Some(time);
    }
}

#[derive(Clone, Debug)]
pub struct CommitInfo {
    pub author: String,
    pub committer: String,
    pub commit_time: Timestamp,
    pub message: String,
    pub sha: String,
    pub url: String,
}
