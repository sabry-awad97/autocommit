use colored::Colorize;
use dialoguer::theme::ColorfulTheme;
use dialoguer::Confirm;

use crate::commands::config::AutocommitConfig;
use crate::git::GitRepository;
use crate::utils::{outro, spinner, MessageRole};

use super::chat_context::ChatContext;

pub async fn generate_autocommit_message(
    config: &AutocommitConfig,
    staged_diff: &str,
) -> anyhow::Result<Option<String>> {
    let mut commit_spinner = spinner();
    commit_spinner.start("Generating the commit message");

    let mut chat_context = ChatContext::get_initial_context(config);
    chat_context.add_message(MessageRole::User, staged_diff.to_owned());

    let commit_message = chat_context.generate_message().await?;
    commit_spinner.stop("📝 Commit message generated successfully");

    outro(&format!(
        "Commit message:\n\
         ——————————————————\n\
         {}\n\
         ——————————————————",
        commit_message
    ));

    let preview_confirmed_by_user = Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt(format!("Do you want to commit these changes?"))
        .interact_opt()?;

    if let Some(true) = preview_confirmed_by_user {
        let mut commit_spinner = spinner();
        commit_spinner.start("Committing changes...");
        GitRepository::git_commit(&commit_message).await?;
        commit_spinner.stop(format!("{} Changes committed successfully", "✔".green()));
        Ok(Some(commit_message))
    } else {
        outro("Commit cancelled, exiting...");
        Ok(None)
    }
}
