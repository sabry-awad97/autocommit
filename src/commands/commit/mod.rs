use std::time::Duration;

use crate::{
    git::GitRepository,
    utils::{outro, spinner},
};

use anyhow::{anyhow, Result};
use dialoguer::{theme::ColorfulTheme, Confirm, MultiSelect};
use structopt::StructOpt;

use super::config::AutocommitConfig;

mod generate;
mod prompt;
mod push;
mod stage;

#[derive(Debug, StructOpt)]
pub struct CommitCommand {}

impl CommitCommand {
    async fn prompt_for_selected_files(&self, changed_files: &[String]) -> Result<Vec<String>> {
        let selected_items = MultiSelect::with_theme(&ColorfulTheme::default())
            .with_prompt(format!(
                "Select the files you want to add to the commit ({} files changed):",
                changed_files.len()
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

    pub async fn run(&self, config: &AutocommitConfig, mut is_stage_all_flag: bool) -> Result<()> {
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
                    let files = self.prompt_for_selected_files(&changed_files).await?;
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
                return generate::generate_autocommit_message(config, &staged_diff).await;
            }
        }
    }
}
