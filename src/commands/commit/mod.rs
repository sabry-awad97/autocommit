use std::time::Duration;

use crate::{
    git::GitRepository,
    utils::{outro, spinner},
};
use anyhow::anyhow;
use colored::Colorize;
use dialoguer::{theme::ColorfulTheme, Confirm};
use log::info;
use structopt::StructOpt;

use super::config::AutocommitConfig;

mod chat_context;
mod generate;
mod prompt;

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
                    let files = prompt::prompt_for_selected_files(&changed_files).await?;
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
                    generate::generate_autocommit_message(config, &staged_diff).await?;

                if let Ok(Some(new_message)) =
                    prompt::prompt_to_commit_changes(config, &staged_diff, &commit_message).await
                {
                    Self::commit_changes(&new_message).await?;
                    if let Some(remote) = prompt::prompt_for_remote().await? {
                        if let Ok(true) = prompt::prompt_for_push(&remote) {
                            Self::push_changes(&new_message, &remote).await?;
                            info!("Autocommit process completed successfully");
                        }
                    }
                }

                let should_continue = prompt::prompt_to_continue().await?;
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
}
