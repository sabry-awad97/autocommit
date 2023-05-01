use std::time::Duration;

use crate::{
    git::GitRepository,
    utils::{outro, spinner},
};

use colored::Colorize;
use dialoguer::{theme::ColorfulTheme, Confirm, Editor};
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
                let mut commit_message =
                    generate::generate_autocommit_message(config, &staged_diff).await?;

                loop {
                    let edit_message = Confirm::with_theme(&ColorfulTheme::default())
                        .with_prompt("Do you want to edit the commit message?")
                        .interact()?;

                    if !edit_message {
                        break;
                    }


                    let editor = Editor::new();

                    if let Some(new_message) = editor.edit(&commit_message)? {
                        commit_message = new_message.trim().to_string();
                        break;
                    }

                    let is_generate_new_message_confirmed_by_user =
                        Confirm::with_theme(&ColorfulTheme::default())
                            .with_prompt(format!(
                                "{}",
                                "Do you want to generate a new commit message?"
                            ))
                            .interact()?;
                    if is_generate_new_message_confirmed_by_user {
                        // let new_content = prompt::prompt_for_new_message().await?;
                        let mut new_content = String::from("Suggest new commit message");
                        new_content.push_str(&staged_diff);
                        commit_message =
                            generate::generate_autocommit_message(config, &new_content).await?;
                    } else {
                        outro(&format!("{}", "Exiting...".red()));
                        return Ok(());
                    }
                }

                if let Ok(true) = prompt::prompt_to_commit_changes() {
                    generate::commit_changes(&commit_message).await?;
                    if let Some(remote) = prompt::prompt_for_remote().await? {
                        if let Ok(true) = prompt::prompt_for_push(&remote) {
                            push::push_changes(&commit_message, &remote).await?;
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
}
