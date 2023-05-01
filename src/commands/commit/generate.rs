use dialoguer::theme::ColorfulTheme;
use dialoguer::Confirm;

use crate::commands::config::AutocommitConfig;
use crate::git::GitRepository;
use crate::utils::{generate_message, outro, spinner, Message, MessageRole};

fn get_chat_context(config: &AutocommitConfig, diff: &str) -> String {
    let language = format!("{:?}", config.config_data.language).to_lowercase();
    format!("Write a git commit message in present tense for the following diff without prefacing it with anything. \
    Do not be needlessly verbose and make sure the answer is concise and to the point. \
    The response must be in the language {}: \n{}", language, diff)
}

pub async fn generate_autocommit_message(
    config: &AutocommitConfig,
    staged_diff: &str,
) -> anyhow::Result<String> {
    let mut commit_spinner = spinner();
    commit_spinner.start("Generating the commit message");

    let prompt = Message {
        role: MessageRole::User,
        content: get_chat_context(config, &staged_diff),
    };

    let commit_message = generate_message(&[prompt]).await?;
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
        commit_spinner.stop("✔ Changes committed successfully");
        Ok(commit_message)
    } else {
        outro("Commit cancelled, exiting...");
        Ok("".to_string())
    }
}
