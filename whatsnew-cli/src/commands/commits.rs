use anyhow::{Result, anyhow};
use jiff::Timestamp;
use whatsnew_core::UserData;

use super::GlobalOpts;
use crate::output::print_new_commits_to_stdout;

pub fn add(userdata: &mut UserData, input: &str) -> Result<()> {
    userdata.add_repo(input)?;
    println!("Added {input}");
    Ok(())
}

pub fn remove(userdata: &mut UserData, input: &str) -> Result<()> {
    userdata.remove_repo(input)?;
    println!("Removed {input}");
    Ok(())
}

pub fn list(userdata: &UserData) -> Result<()> {
    if userdata.commits.is_empty() {
        println!(
            "No repos tracked, you can start tracking a repo with `whatsnew repos add <OWNER/REPO>`"
        );
    } else {
        for repo in userdata.commits.keys() {
            println!("{repo}");
        }
    }
    Ok(())
}

pub async fn check(userdata: &mut UserData, global_opts: &GlobalOpts) -> Result<()> {
    if userdata.commits.is_empty() {
        println!(
            "No repos tracked, you can start tracking a repo with `whatsnew repos add <OWNER/REPO>`"
        );
    } else {
        let all_new_commits = userdata.get_all_new_commits().await?;

        for (reponame, commits) in all_new_commits {
            // print function returns last_sha if new commits exists
            let last_sha = print_new_commits_to_stdout(&reponame, &commits)?;

            if !global_opts.skip_update {
                if let Some(repo) = userdata.get_repo_mut(&reponame) {
                    repo.set_last_viewed_time(Timestamp::now());
                    if let Some(last_sha) = last_sha {
                        repo.set_last_viewed_sha(&last_sha);
                    }
                } else {
                    return Err(anyhow!("Repository {reponame} not found"));
                }
            }
        }
    }
    Ok(())
}
