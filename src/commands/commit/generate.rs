use dialoguer::theme::ColorfulTheme;
use dialoguer::{Confirm, MultiSelect};

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
) -> anyhow::Result<()> {
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
        commit_changes(&commit_message).await?;
        let remotes = GitRepository::get_git_remotes().await?;

        let remote = if remotes.len() == 1 {
            &remotes[0]
        } else {
            let remote_items = remotes.iter().map(|r| r.as_str()).collect::<Vec<_>>();
            let selected_remote = MultiSelect::with_theme(&ColorfulTheme::default())
                .with_prompt("Select the remote repository to push changes to:")
                .items(&remote_items)
                .interact_opt()?;
            if let Some(items) = selected_remote {
                if items.is_empty() {
                    outro("No remote repository selected, exiting...");
                    return Ok(());
                }
                remotes[items[0]].as_str()
            } else {
                outro("No remote repository selected, exiting...");
                return Ok(());
            }
        };

        let push_confirmed_by_user = Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt(format!(
                "Do you want to push these changes to the remote repository {}?",
                remote
            ))
            .interact_opt()?;

        if let Some(true) = push_confirmed_by_user {
            push_changes(remote).await?;
        }
    } else {
        outro("Commit cancelled, exiting...");
    }

    Ok(())
}

async fn commit_changes(commit_message: &str) -> anyhow::Result<()> {
    let mut commit_spinner = spinner();
    commit_spinner.start("Committing changes...");
    GitRepository::git_commit(&commit_message).await?;
    commit_spinner.stop("✔ Changes committed successfully");
    Ok(())
}

async fn push_changes(remote: &str) -> anyhow::Result<()> {
    let mut push_spinner = spinner();
    push_spinner.start(format!(
        "Pushing changes to remote repository {}...",
        remote
    ));
    GitRepository::git_push(&remote).await?;
    push_spinner.stop(format!("✔ Changes pushed to remote repository {}", remote));
    Ok(())
}
