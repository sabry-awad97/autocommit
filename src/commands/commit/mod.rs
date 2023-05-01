use std::time::Duration;

use crate::{
    git::GitRepository,
    utils::{outro, spinner},
};

use dialoguer::{theme::ColorfulTheme, Confirm};
use structopt::StructOpt;

use super::config::AutocommitConfig;

mod chat_context;
mod generate;
mod prompt;
mod push;
mod stage;

#[derive(Debug, StructOpt)]
pub struct CommitCommand {}

impl CommitCommand {
    pub async fn run(
        &self,
        config: &AutocommitConfig,
        mut is_stage_all_flag: bool,
    ) -> anyhow::Result<()> {
        GitRepository::assert_git_repo().await?;

        loop {
            if is_stage_all_flag {
                stage::stage_all_changed_files().await?;
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
                        .with_prompt(format!(
                            "{}",
                            "Do you want to stage all files and generate commit message?"
                        ))
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
                    outro("No files selected for staging, exiting...");
                    return Ok(());
                }
            } else {
                staged_spinner.stop(format!(
                    "{} staged files:\n{}",
                    staged_files.len(),
                    staged_files
                        .iter()
                        .map(|file| format!("  {}", file))
                        .collect::<Vec<_>>()
                        .join("\n")
                ));

                let staged_diff = GitRepository::get_staged_diff(&[]).await?;
                if let Some(commit_message) =
                    generate::generate_autocommit_message(config, &staged_diff).await?
                {
                    if let Some(remote) = prompt::prompt_for_remote().await? {
                        if let Ok(true) = prompt::prompt_for_push(&remote) {
                            push::push_changes(&commit_message, &remote).await?;
                        }
                    }
                }

                let should_continue = prompt::prompt_to_continue().await?;
                if !should_continue {
                    outro("Exiting...");
                    return Ok(());
                }

                is_stage_all_flag = false;
            }
        }
    }
}
