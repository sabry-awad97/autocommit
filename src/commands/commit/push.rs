use colored::Colorize;

use crate::{
    git::GitRepository,
    utils::{outro, spinner},
};

pub async fn push_changes(commit_message: &str, remote: &str) -> anyhow::Result<()> {
    let mut push_spinner = spinner();
    push_spinner.start(format!(
        "Pushing changes to remote repository {}...",
        remote.green()
    ));
    GitRepository::git_push(&remote).await?;
    push_spinner.stop(format!(
        "{} Changes pushed to remote repository {}",
        "✔".green(),
        remote.green()
    ));
    outro(&format!(
        "Changes committed and pushed to remote repository {} with message:\n\
        ——————————————————\n\
        {}\n\
        ——————————————————",
        remote.green(), commit_message
    ));

    Ok(())
}
