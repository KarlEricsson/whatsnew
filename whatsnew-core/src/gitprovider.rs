use std::sync::LazyLock;

use anyhow::{Context, Result, anyhow};
use jiff::Timestamp;
use octocrab::Octocrab;

use crate::repos::CommitInfo;

pub(crate) trait GitClient {
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

static GITHUB_CLIENT: LazyLock<Result<GitHubClient>> = LazyLock::new(|| {
    let token = get_github_token()?;
    let client = Octocrab::builder().personal_token(token).build()?;
    Ok(GitHubClient { client })
});

impl GitHubClient {
    pub fn instance() -> Result<&'static Self> {
        GITHUB_CLIENT.as_ref().map_err(|e| anyhow::anyhow!("{}", e))
    }
}

fn get_github_token() -> Result<String> {
    std::env::var("GH_TOKEN")
        .or_else(|_| std::env::var("GITHUB_TOKEN"))
        .or_else(|_| std::env::var("WHATSNEW_TOKEN"))
        .or_else(|_| -> std::result::Result<String, anyhow::Error> {
            let output = std::process::Command::new("gh")
                .args(["auth", "token"])
                .output()
                .context("Failed to execute 'gh auth token' command")?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(anyhow::anyhow!(
                    "'gh auth token' command failed: {}",
                    stderr
                ));
            }

            let token = String::from_utf8(output.stdout)
                .context("Invalid UTF-8 output from 'gh auth token'")?
                .trim()
                .to_string();

            if token.is_empty() {
                return Err(anyhow::anyhow!("'gh auth token' returned empty token"));
            }

            Ok(token)
        })
        .context(
            "Found no GitHub token\nPossible variables: GH_TOKEN, GITHUB_TOKEN and WHATSNEW_TOKEN",
        )
}

impl GitClient for GitHubClient {
    async fn get_repo_commits(&self, owner: &str, name: &str) -> Result<Vec<CommitInfo>> {
        let page = self
            .client
            .repos(owner, name)
            .list_commits()
            .per_page(30)
            .send()
            .await
            .map_err(|err| list_commits_error(err, owner, name))?;

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
            .await
            .map_err(|err| list_commits_error(err, owner, name))?;

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

/// Wrap a commit-listing error with the repo name, and add a hint when GitHub
/// reports the repo as not found (HTTP 404) — e.g. the repo was deleted,
/// renamed, or made private, or the token lacks access to it.
fn list_commits_error(err: octocrab::Error, owner: &str, name: &str) -> anyhow::Error {
    let message = match &err {
        octocrab::Error::GitHub { source, .. } if source.status_code.as_u16() == 404 => format!(
            "repo {owner}/{name} was not found on GitHub: it may have been deleted, renamed, or \
             made private, or the token may lack access to it"
        ),
        _ => format!("failed to fetch commits for {owner}/{name}"),
    };
    anyhow!(err).context(message)
}
