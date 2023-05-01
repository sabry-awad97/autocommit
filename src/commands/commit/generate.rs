use colored::Colorize;

use crate::commands::config::AutocommitConfig;
use crate::git::GitRepository;
use crate::utils::{outro, spinner, MessageRole};

use super::chat_context::ChatContext;

pub async fn generate_autocommit_message(
    config: &AutocommitConfig,
    content: &str,
) -> anyhow::Result<String> {
    let mut commit_spinner = spinner();
    commit_spinner.start("Generating the commit message");

    let mut chat_context = ChatContext::get_initial_context(config);
    chat_context.add_message(MessageRole::User, content.to_owned());

    let commit_message = chat_context.generate_message().await?;

    outro(&format!(
        "Commit message:\n\
        ——————————————————\n\
        {}\n\
        ——————————————————",
        commit_message
    ));

    commit_spinner.stop("📝 Commit message generated successfully");
    Ok(commit_message)
}

pub async fn commit_changes(commit_message: &str) -> anyhow::Result<()> {
    let mut commit_spinner = spinner();
    commit_spinner.start("Committing changes...");
    GitRepository::git_commit(&commit_message).await?;
    commit_spinner.stop(format!("{} Changes committed successfully", "✔".green()));
    Ok(())
}
