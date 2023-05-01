use crate::{
    git::GitRepository,
    utils::{outro, spinner},
};

pub async fn push_changes(commit_message: &str, remote: &str) -> anyhow::Result<()> {
    let mut push_spinner = spinner();
    push_spinner.start(format!(
        "Pushing changes to remote repository {}...",
        remote
    ));
    GitRepository::git_push(&remote).await?;
    push_spinner.stop(format!("✔ Changes pushed to remote repository {}", remote));
    outro(&format!(
        "Changes committed and pushed to remote repository {} with message:\n\
        ——————————————————\n\
        {}\n\
        ——————————————————",
        remote, commit_message
    ));

    Ok(())
}
