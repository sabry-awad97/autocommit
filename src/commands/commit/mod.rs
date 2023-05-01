use std::time::Duration;

use crate::{
    commands::commit::chat_context::ChatContext,
    git::GitRepository,
    utils::{outro, spinner, MessageRole},
};
use anyhow::anyhow;
use colored::{Color, Colorize};
use dialoguer::{theme::ColorfulTheme, Confirm, MultiSelect};
use log::info;
use structopt::StructOpt;

use super::config::AutocommitConfig;

mod chat_context;

#[derive(Debug, StructOpt)]
pub struct CommitCommand {}

impl CommitCommand {
    pub async fn stage_all_changed_files() -> anyhow::Result<()> {
        let changed_files = GitRepository::get_changed_files().await?;

        if !changed_files.is_empty() {
            GitRepository::git_add(&changed_files).await?;
        } else {
            return Err(anyhow!(
                "No changes detected, write some code and run again"
            ));
        }

        Ok(())
    }

    pub async fn run(
        &self,
        config: &AutocommitConfig,
        mut is_stage_all_flag: bool,
    ) -> anyhow::Result<()> {
        info!("Starting autocommit process");
        GitRepository::assert_git_repo().await?;

        loop {
            if is_stage_all_flag {
                Self::stage_all_changed_files().await?;
            }

            let staged_files = GitRepository::get_staged_files().await?;
            let changed_files = GitRepository::get_changed_files().await?;

            if staged_files.is_empty() && changed_files.is_empty() {
                outro("No changes detected, exiting...");
                return Ok(());
            }

            let mut staged_spinner = spinner();
            staged_spinner.start("Counting staged files");
            tokio::time::sleep(Duration::from_millis(500)).await;
            if staged_files.is_empty() {
                staged_spinner.stop("No files are staged");

                let is_stage_all_and_commit_confirmed_by_user =
                    Confirm::with_theme(&ColorfulTheme::default())
                        .with_prompt("Do you want to stage all files and generate commit message?")
                        .default(true)
                        .interact_opt()?;

                if let Some(true) = is_stage_all_and_commit_confirmed_by_user {
                    is_stage_all_flag = true;
                    continue;
                } else if changed_files.len() > 0 {
                    let files = Self::prompt_for_selected_files(&changed_files).await?;
                    GitRepository::git_add(&files).await?;
                    is_stage_all_flag = false;
                    continue;
                } else {
                    outro(&format!(
                        "{}",
                        "No files selected for staging, exiting...".red()
                    ));
                    return Ok(());
                }
            } else {
                staged_spinner.stop(format!(
                    "{} staged files:\n{}",
                    staged_files.len().to_string().green(),
                    staged_files
                        .iter()
                        .map(|file| format!("  {}", file))
                        .collect::<Vec<_>>()
                        .join("\n")
                ));

                let staged_diff = GitRepository::get_staged_diff(&[]).await?;
                let commit_message =
                    Self::generate_autocommit_message(config, &staged_diff).await?;

                if let Ok(Some(new_message)) =
                    Self::prompt_to_commit_changes(config, &staged_diff, &commit_message).await
                {
                    Self::commit_changes(&new_message).await?;
                    if let Some(remote) = Self::prompt_for_remote().await? {
                        if let Ok(true) = Self::prompt_for_push(&remote) {
                            Self::push_changes(&new_message, &remote).await?;
                            info!("Autocommit process completed successfully");
                        }
                    }
                }

                let should_continue = Self::prompt_to_continue().await?;
                if !should_continue {
                    outro(&format!("{}", "Exiting...".red()));
                    return Ok(());
                }

                is_stage_all_flag = false;
            }
        }
    }

    pub async fn commit_changes(commit_message: &str) -> anyhow::Result<()> {
        const COMMITTING_CHANGES: &str = "Committing changes...";

        let mut commit_spinner = spinner();
        commit_spinner.start(COMMITTING_CHANGES);
        GitRepository::git_commit(&commit_message).await?;
        commit_spinner.stop(format!("{} Changes committed successfully", "✔".green()));
        Ok(())
    }

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

    pub async fn prompt_to_commit_changes(
        config: &AutocommitConfig,
        staged_diff: &str,
        commit_message: &str,
    ) -> anyhow::Result<Option<String>> {
        let mut message = commit_message.to_string();

        loop {
            let is_generate_new_message_confirmed_by_user =
                Confirm::with_theme(&ColorfulTheme::default())
                    .with_prompt(format!(
                        "{}",
                        "Do you want to generate a new commit message?"
                    ))
                    .default(false)
                    .interact()?;
            if is_generate_new_message_confirmed_by_user {
                let mut new_content = String::from("Suggest a professional git commit message\n");
                new_content.push_str(&staged_diff);
                message = Self::generate_autocommit_message(config, &new_content).await?;
            } else {
                break;
            }
        }

        let preview_confirmed_by_user = Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt(format!("Do you want to commit these changes?"))
            .default(true)
            .interact_opt()?;

        if let Some(true) = preview_confirmed_by_user {
            Ok(Some(message))
        } else {
            outro("Commit cancelled, exiting...");
            Ok(None)
        }
    }

    pub async fn generate_autocommit_message(
        config: &AutocommitConfig,
        content: &str,
    ) -> anyhow::Result<String> {
        const GENERATING_MESSAGE: &str = "Generating the commit message...";
        let mut commit_spinner = spinner();
        commit_spinner.start(GENERATING_MESSAGE);

        let mut chat_context = ChatContext::get_initial_context(config);
        chat_context.add_message(MessageRole::User, content.to_owned());

        let commit_message = chat_context.generate_message().await?;
        commit_spinner.stop("📝 Commit message generated successfully");

        let separator_length = 40;
        let separator = "—"
            .repeat(separator_length)
            .color(Color::TrueColor {
                r: 128,
                g: 128,
                b: 128,
            })
            .bold();

        outro(&format!(
            "Commit message:\n{}\n{}\n{}",
            separator, commit_message, separator
        ));
        Ok(commit_message)
    }

    pub async fn prompt_to_continue() -> anyhow::Result<bool> {
        let should_continue = Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("Do you want to continue?")
            .default(false)
            .interact()?;
        Ok(should_continue)
    }

    pub async fn prompt_for_remote() -> anyhow::Result<Option<String>> {
        let remotes = GitRepository::get_git_remotes().await?;
        if remotes.is_empty() {
            outro("No remote repository found, exiting...");
            return Ok(None);
        }

        if remotes.len() == 1 {
            return Ok(Some(remotes[0].clone()));
        }

        let remote_items = remotes.iter().map(|r| r.as_str()).collect::<Vec<_>>();
        let selected_remote = MultiSelect::with_theme(&ColorfulTheme::default())
            .with_prompt("Select the remote repository to push changes to:")
            .items(&remote_items)
            .report(false)
            .interact_opt()?;

        if let Some(items) = selected_remote {
            if items.is_empty() {
                outro("No remote repository selected, exiting...");
                return Ok(None);
            }
            Ok(Some(remotes[items[0]].clone()))
        } else {
            outro("No remote repository selected, exiting...");
            return Ok(None);
        }
    }

    pub async fn prompt_for_selected_files(
        changed_files: &[String],
    ) -> anyhow::Result<Vec<String>> {
        let selected_items = MultiSelect::with_theme(&ColorfulTheme::default())
            .with_prompt(format!(
                "Select the files you want to add to the commit ({} files changed):",
                changed_files.len().to_string().green()
            ))
            .items(&changed_files)
            .interact_opt()?;

        if let Some(items) = selected_items {
            if items.is_empty() {
                return Err(anyhow!("Please select at least one option with space"));
            }

            let files = items
                .iter()
                .map(|&i| changed_files[i].to_string())
                .collect::<Vec<_>>();

            Ok(files)
        } else {
            return Err(anyhow!("No files selected for staging"));
        }
    }

    pub fn prompt_for_push(remote: &str) -> anyhow::Result<bool> {
        let push_confirmed_by_user = Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt(format!(
                "Do you want to push these changes to the remote repository {}?",
                remote
            ))
            .default(true)
            .interact_opt()?;

        if let Some(true) = push_confirmed_by_user {
            outro(&format!("Changes pushed to remote repository {}", remote));
            Ok(true)
        } else {
            outro("Push cancelled, exiting...");
            Ok(false)
        }
    }
}
