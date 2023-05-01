use std::{thread, time::Duration};

use crate::utils::{
    generate_message, get_colors, get_unicode_string, spinner, Message, MessageRole,
};

use crate::git::GitRepository;

use anyhow::{anyhow, Result};
use async_recursion::async_recursion;
use dialoguer::{theme::ColorfulTheme, Confirm, MultiSelect};
use structopt::StructOpt;

use super::config::AutocommitConfig;

struct CommitPrompt<'a> {
    config: &'a AutocommitConfig,
    diff: String,
}

impl<'a> CommitPrompt<'a> {
    fn new(config: &'a AutocommitConfig, diff: String) -> Self {
        Self { config, diff }
    }

    fn prompt(&self) -> Result<String> {
        let language = format!("{:?}", self.config.config_data.language).to_lowercase();
        let prompt_text = format!("Write a git commit message in present tense for the following diff without prefacing it with anything. Do not be needlessly verbose and make sure the answer is concise and to the point. The response must be in the language {}: \n{}", language, self.diff);
        let prompt: String = dialoguer::Input::with_theme(&ColorfulTheme::default())
            .with_prompt(prompt_text)
            .interact_text()?;
        Ok(prompt)
    }
}

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
    pub async fn run(&self, config: &AutocommitConfig, is_stage_all_flag: bool) -> Result<()> {
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

            if is_stage_all_and_commit_confirmed_by_user.is_some() {
                self.run(config, true).await?;
                std::process::exit(1);
            }

            if staged_files.is_empty() && changed_files.len() > 0 {
                let selected_items = MultiSelect::with_theme(&ColorfulTheme::default())
                    .with_prompt("Select the files you want to add to the commit:")
                    .items(&changed_files)
                    .interact_opt()?;

                if selected_items.is_some() {
                    let files = &selected_items
                        .unwrap()
                        .iter()
                        .map(|&i| changed_files[i].to_string())
                        .collect::<Vec<_>>();

                        GitRepository::git_add(&files).await?;
                }
            }
            self.run(config, false).await?;
            std::process::exit(1);
        }

        let staged_diff = GitRepository::get_staged_diff(&[]).await?;
        let _prompt = Message {
            role: MessageRole::User,
            content: get_prompt(config, &staged_diff),
        };

        // let mesage: String = generate_message(&[prompt]).await?;
        thread::sleep(Duration::from_secs(2));
        staged_spinner.stop("Done!");
        Ok(())
    }
}
