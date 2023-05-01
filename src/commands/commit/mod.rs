use std::{thread, time::Duration};

use crate::utils::{generate_message, spinner, Message, MessageRole};

use crate::git::GitRepository;

use anyhow::{anyhow, Result};
use async_recursion::async_recursion;
use dialoguer::{theme::ColorfulTheme, Confirm, MultiSelect};
use structopt::StructOpt;

use super::config::AutocommitConfig;

#[derive(Debug, StructOpt)]
pub struct CommitCommand {}

fn get_prompt(config: &AutocommitConfig, diff: &str) -> String {
    let language = format!("{:?}", config.config_data.language).to_lowercase();
    format!("Write a git commit message in present tense for the following diff without prefacing it with anything. \
    Do not be needlessly verbose and make sure the answer is concise and to the point. \
    The response must be in the language {}: \n{}", language, diff)
}

impl CommitCommand {
    #[async_recursion]
    pub async fn run(&self, config: &AutocommitConfig, is_stage_all_flag: bool) -> Result<String> {
        GitRepository::assert_git_repo().await?;

        if is_stage_all_flag {
            let changed_files = GitRepository::get_changed_files().await?;

            if !changed_files.is_empty() {
                GitRepository::git_add(&changed_files).await?;
            } else {
                return Err(anyhow!(
                    "No changes detected, write some code and run again"
                ));
            }
        }

        let staged_files = GitRepository::get_staged_files().await?;
        let changed_files = GitRepository::get_changed_files().await?;

        if staged_files.is_empty() && changed_files.is_empty() {
            return Err(anyhow!(
                "No changes detected, write some code and run again"
            ));
        }

        let mut staged_spinner = spinner();
        staged_spinner.start("Counting staged files");
        thread::sleep(Duration::from_secs(2));
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
                self.run(config, true).await
            } else if changed_files.len() > 0 {
                let selected_items = MultiSelect::with_theme(&ColorfulTheme::default())
                    .with_prompt("Select the files you want to add to the commit:")
                    .items(&changed_files)
                    .interact_opt()?;

                if let Some(items) = selected_items {
                    let files = items
                        .iter()
                        .map(|&i| changed_files[i].to_string())
                        .collect::<Vec<_>>();

                    GitRepository::git_add(&files).await?;
                    self.run(config, false).await
                } else {
                    Err(anyhow!("No files selected for staging"))
                }
            } else {
                Err(anyhow!("No files selected for staging"))
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

            self.generate_commit_message_from_git_diff(config, &staged_diff)
                .await
        }
    }

    async fn generate_commit_message_from_git_diff(
        &self,
        config: &AutocommitConfig,
        staged_diff: &str,
    ) -> Result<String> {
        let prompt = Message {
            role: MessageRole::User,
            content: get_prompt(config, &staged_diff),
        };

        generate_message(&[prompt]).await
    }
}
