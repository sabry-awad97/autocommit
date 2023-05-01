use std::{thread, time::Duration};

use crate::git::GitRepository;
use crate::utils::{generate_message, outro, spinner, Message, MessageRole};

use anyhow::{anyhow, Result};
use dialoguer::{theme::ColorfulTheme, Confirm, MultiSelect};
use structopt::StructOpt;

use super::config::AutocommitConfig;

#[derive(Debug, StructOpt)]
pub struct CommitCommand {}

fn get_chat_context(config: &AutocommitConfig, diff: &str) -> String {
    let language = format!("{:?}", config.config_data.language).to_lowercase();
    format!("Write a git commit message in present tense for the following diff without prefacing it with anything. \
    Do not be needlessly verbose and make sure the answer is concise and to the point. \
    The response must be in the language {}: \n{}", language, diff)
}

impl CommitCommand {
    async fn stage_all_changed_files(&self) -> Result<()> {
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
                self.stage_all_changed_files().await?;
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
            thread::sleep(Duration::from_millis(500));
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
                    return Err(anyhow!("No files selected for staging"));
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
                return self.generate_autocommit_message(config, &staged_diff).await;
            }
        }
    }

    async fn generate_autocommit_message(
        &self,
        config: &AutocommitConfig,
        staged_diff: &str,
    ) -> Result<()> {
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

        if preview_confirmed_by_user.is_some() {
            self.commit_changes(&commit_message).await?;
            let push_confirmed_by_user = Confirm::with_theme(&ColorfulTheme::default())
                .with_prompt("Do you want to push these changes to the remote repository?")
                .interact_opt()?;

            if let Some(true) = push_confirmed_by_user {
                self.push_changes().await?;
            }
        }

        Ok(())
    }

    async fn commit_changes(&self, commit_message: &str) -> Result<()> {
        let mut commit_spinner = spinner();
        commit_spinner.start("Committing changes...");
        GitRepository::git_commit(&commit_message).await?;
        commit_spinner.stop("✔ Changes committed successfully");
        Ok(())
    }

    async fn push_changes(&self) -> Result<()> {
        let mut push_spinner = spinner();
        push_spinner.start("Pushing changes to remote repository...");
        let remotes = GitRepository::get_git_remotes().await?;
        push_spinner.set_message(format!("Running `git push {}`", remotes[0]));
        GitRepository::git_push(&remotes[0]).await?;
        push_spinner.stop(format!(
            "✔ Changes pushed to remote repository {}",
            remotes[0]
        ));
        Ok(())
    }
}
