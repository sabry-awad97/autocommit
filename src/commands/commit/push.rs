use colored::Colorize;

use crate::{git::GitRepository, utils::spinner};

pub async fn push_changes(_commit_message: &str, remote: &str) -> anyhow::Result<()> {
    let mut push_spinner = spinner();
    push_spinner.start(format!(
        "Pushing changes to remote repository {}...",
        remote.green().bold()
    ));
    GitRepository::git_push(&remote).await?;
    push_spinner.stop(format!(
        "{} Changes pushed successfully to remote repository {}.",
        "✔".green(),
        remote.green().bold()
    ));
    Ok(())
}
