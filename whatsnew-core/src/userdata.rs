use std::fs;
use std::path::Path;

use anyhow::{Ok, Result, anyhow};
use futures_util::future::try_join_all;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use crate::repos::{CommitInfo, CommitsRepo};

#[derive(Default, Serialize, Deserialize)]
pub struct UserData {
    pub commits: IndexMap<String, CommitsRepo>,
}

impl UserData {
    pub fn new() -> Self {
        // Avoid reallocation/rehashing (default capacity is 4)
        Self {
            commits: IndexMap::with_capacity(64),
        }
    }

    pub fn save_to_file(&self, path: impl AsRef<Path>) -> Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        fs::write(path, json)?;
        Ok(())
    }

    pub fn load_from_file(path: impl AsRef<Path>) -> Result<Self> {
        let content = fs::read_to_string(path)?;
        let userdata = serde_json::from_str(&content)?;
        Ok(userdata)
    }

    pub fn add_repo(&mut self, repostring: &str) -> Result<()> {
        let repo = CommitsRepo::new(repostring)?;
        self.commits.insert(repostring.to_owned(), repo);
        Ok(())
    }

    pub fn remove_repo(&mut self, name: &str) -> Result<CommitsRepo> {
        self.commits
            .shift_remove(name)
            .ok_or_else(|| anyhow!("Repo {name} not found"))
    }

    pub fn remove_all_repos(&mut self) {
        self.commits.clear();
    }

    pub fn contains_repo(&self, name: &str) -> bool {
        self.commits.contains_key(name)
    }

    pub fn get_repo(&self, name: &str) -> Option<&CommitsRepo> {
        self.commits.get(name)
    }

    pub fn get_repo_mut(&mut self, name: &str) -> Option<&mut CommitsRepo> {
        self.commits.get_mut(name)
    }

    pub async fn get_all_new_commits(&self) -> Result<Vec<(String, Vec<CommitInfo>)>> {
        let futures: Vec<_> = self
            .commits
            .iter()
            .map(async |(reponame, commits_repo)| {
                let commits = if commits_repo.last_viewed_time.is_some() {
                    commits_repo.get_repo_commits_since().await?
                } else {
                    commits_repo.get_repo_commits().await?
                };
                Ok((reponame.clone(), commits))
            })
            .collect();
        let results = try_join_all(futures).await?;

        Ok(results)
    }
}
