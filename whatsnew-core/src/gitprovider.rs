use anyhow::{Context, Result};
use jiff::Timestamp;
use octocrab::Octocrab;

use crate::repos::CommitInfo;

pub(crate) trait GitClient {
    fn new() -> Result<Self>
    where
        Self: Sized;

    async fn get_repo_commits(&self, owner: &str, name: &str) -> Result<Vec<CommitInfo>>;
    async fn get_repo_commits_since(
        &self,
        owner: &str,
        name: &str,
        since_time: &Timestamp,
    ) -> Result<Vec<CommitInfo>>;
}

pub(crate) struct GitHubClient {
    client: octocrab::Octocrab,
}

impl GitClient for GitHubClient {
    fn new() -> Result<Self> {
        let token = std::env::var("GH_TOKEN")
            .or_else(|_| std::env::var("GITHUB_TOKEN"))
            .context("Neither GH_TOKEN nor GITHUB_TOKEN environment variables found")?;
        let client = Octocrab::builder().personal_token(token).build()?;
        Ok(Self { client })
    }

    async fn get_repo_commits(&self, owner: &str, name: &str) -> Result<Vec<CommitInfo>> {
        let page = self
            .client
            .repos(owner, name)
            .list_commits()
            .per_page(30)
            .send()
            .await?;

        Ok(page
            .items
            .into_iter()
            .map(|commit| CommitInfo {
                author: commit.commit.author.map_or(String::new(), |a| a.name),
                committer: commit
                    .commit
                    .committer
                    .as_ref()
                    .map_or(String::new(), |c| c.name.clone()),
                commit_time: commit
                    .commit
                    .committer
                    .unwrap()
                    .date
                    .unwrap()
                    .to_rfc3339()
                    .parse::<Timestamp>()
                    .unwrap(),
                message: commit.commit.message,
                sha: commit.sha,
                url: commit.html_url,
            })
            .collect())
    }

    async fn get_repo_commits_since(
        &self,
        owner: &str,
        name: &str,
        since_time: &Timestamp,
    ) -> Result<Vec<CommitInfo>> {
        let page = self
            .client
            .repos(owner, name)
            .list_commits()
            .since(since_time.to_string().parse()?)
            .per_page(30)
            .send()
            .await?;

        Ok(page
            .items
            .into_iter()
            .map(|commit| CommitInfo {
                author: commit.commit.author.map_or(String::new(), |a| a.name),
                committer: commit
                    .commit
                    .committer
                    .as_ref()
                    .map_or(String::new(), |c| c.name.clone()),
                commit_time: commit
                    .commit
                    .committer
                    .unwrap()
                    .date
                    .unwrap()
                    .to_rfc3339()
                    .parse::<Timestamp>()
                    .unwrap(),
                message: commit.commit.message,
                sha: commit.sha,
                url: commit.html_url,
            })
            .collect())
    }
}
