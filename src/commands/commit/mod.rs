use crate::{
    commands::commit::chat_context::ChatContext,
    git::GitRepository,
    utils::{outro, spinner, MessageRole},
};
use anyhow::anyhow;
use clipboard::{ClipboardContext, ClipboardProvider};
use colored::Colorize;
use dialoguer::{theme::ColorfulTheme, Confirm, Input, MultiSelect};
use log::{debug, info};
use prettytable::{color, format::Alignment, row, Attr, Cell, Row, Table};
use structopt::StructOpt;
use textwrap::fill;

use super::config::AutocommitConfig;

mod chat_context;

#[derive(Debug, StructOpt)]
pub struct CommitCommand {
    #[structopt(short, long)]
    stage_all: bool,

    #[structopt(short, long, default_value = "1")]
    n: usize,
}

impl CommitCommand {
    pub async fn stage_all_changed_files(changed_files: &[String]) -> anyhow::Result<()> {
        if !changed_files.is_empty() {
            GitRepository::git_add_all()?;
        } else {
            return Err(anyhow!(
                "No changes detected, write some code and run again"
            ));
        }
        Ok(())
    }

    pub async fn run(&mut self, config: &AutocommitConfig) -> anyhow::Result<()> {
        info!("Starting autocommit process");
        GitRepository::assert_git_repo().await?;
        loop {
            // Get the list of changed files
            let changed_files = GitRepository::get_changed_files()?;

            if self.stage_all {
                Self::stage_all_changed_files(&changed_files).await?;
            } else {
                // Prompt the user if they want to see the Git status
                let should_show_status = Confirm::with_theme(&ColorfulTheme::default())
                    .with_prompt("Do you want to see the Git status before committing?")
                    .default(false)
                    .interact_opt()?
                    .unwrap_or(false);

                // Show the Git status if the user wants to see it
                if should_show_status {
                    let status_lines = GitRepository::git_status().await?;
                    outro(&format!("{}\n{}", "Git status:".green(), status_lines));
                }
            }

            // Get the list of staged files
            let staged_files = GitRepository::get_staged_files()?;

            // If there are no changes, exit the loop
            if staged_files.is_empty() && changed_files.is_empty() {
                outro(&format!("{}", "No changes detected, exiting...".red()));
                return Ok(());
            }

            // Count the number of staged files and display them to the user
            let mut staged_spinner = spinner();
            staged_spinner.start("Counting staged files...");
            if staged_files.is_empty() {
                staged_spinner.stop("No files are staged");

                // Prompt the user if they want to stage all files and generate a commit message
                let is_stage_all_and_commit_confirmed_by_user =
                    Confirm::with_theme(&ColorfulTheme::default())
                        .with_prompt("Do you want to stage all files and generate commit message?")
                        .default(true)
                        .interact_opt()?
                        .unwrap_or(false);

                // If the user confirms, stage all files and continue the loop
                if is_stage_all_and_commit_confirmed_by_user {
                    self.stage_all = true;
                    continue;
                } else if !changed_files.is_empty() {
                    // Prompt the user to select files to stage
                    let files = Self::prompt_for_selected_files(&changed_files).await?;
                    GitRepository::git_add(&files).await?;
                    self.stage_all = false;
                    continue;
                } else {
                    // If no files are selected for staging, exit the loop
                    outro(&format!(
                        "{}",
                        "No files selected for staging, exiting...".red()
                    ));
                    return Ok(());
                }
            }
            staged_spinner.stop(&format!(
                "{} staged files:\n{}",
                staged_files.len().to_string().green(),
                staged_files
                    .iter()
                    .map(|file| format!("  📄 {}", file))
                    .collect::<Vec<_>>()
                    .join("\n")
            ));

            // Get the diff of the staged files
            let staged_diffs = GitRepository::get_staged_file_diffs(&staged_files)?;

            // Generate a commit message
            let commit_messages: Vec<String> = self
                .generate_autocommit_messages(config, &staged_diffs)
                .await?;

            // Prompt the user to confirm the commit message
            let message = Self::prompt_for_selected_message(&commit_messages).await?;
            self.commit_changes(config, &message).await?;
            // Prompt the user to confirm the push
            if Self::prompt_for_push()? {
                // Prompt the user to select a remote repository
                if let Some(remote) = Self::prompt_for_remote().await? {
                    // Pull changes from the remote repository if necessary
                    if Self::prompt_for_pull(&remote)? {
                        Self::pull_changes(&remote).await?;
                    }
                    // Push changes to the remote repository
                    Self::push_changes(&remote).await?;
                    info!("Autocommit process completed successfully");
                }
            }

            // Prompt the user to continue or exit the loop
            let should_continue = Self::prompt_to_continue().await?;
            if !should_continue {
                outro(&format!("{}", "Exiting...".red()));
                return Ok(());
            }

            self.stage_all = false;
        }
    }

    pub async fn commit_changes(
        &self,
        config: &AutocommitConfig,
        commit_message: &str,
    ) -> anyhow::Result<()> {
        const COMMITTING_CHANGES: &str = "Committing changes...";

        let mut commit_spinner = spinner();
        commit_spinner.start(COMMITTING_CHANGES);

        let name = config.config_data.name.get_value_ref();
        let email = config.config_data.email.get_value_ref();

        let commit_output = GitRepository::git_commit(commit_message, name, email).await?;
        let commit_table = GitRepository::get_commit_summary_table(name, email).await?;

        commit_spinner.stop(&format!("{} Changes committed successfully", "✔".green()));
        if GitRepository::get_commit_count()? == 1 {
            outro(&commit_output);
        } else {
            commit_table.printstd();
        }

        debug!("Changes committed successfully");

        Ok(())
    }

    pub async fn pull_changes(remote: &str) -> anyhow::Result<()> {
        let mut pull_spinner = spinner();
        pull_spinner.start(&format!(
            "Pulling changes from remote repository {}...",
            remote.green().bold()
        ));
        GitRepository::git_pull(remote).await?;
        pull_spinner.stop(&format!(
            "{} Changes pulled successfully from remote repository {}.",
            "✔".green(),
            remote.green().bold()
        ));
        debug!(
            "Changes pulled successfully from remote repository {}",
            remote
        );
        Ok(())
    }

    pub async fn push_changes(remote: &str) -> anyhow::Result<()> {
        let mut push_spinner = spinner();
        push_spinner.start(&format!(
            "Pushing changes to remote repository {}...",
            remote.green().bold()
        ));
        GitRepository::git_push(remote).await?;
        push_spinner.stop(&format!(
            "{} Changes pushed successfully to remote repository {}.",
            "✔".green(),
            remote.green().bold()
        ));
        debug!(
            "Changes pushed successfully to remote repository {}",
            remote
        );
        Ok(())
    }

    pub async fn generate_autocommit_messages(
        &self,
        config: &AutocommitConfig,
        content: &[String],
    ) -> anyhow::Result<Vec<String>> {
        let mut commit_spinner = spinner();

        let mut chat_context = ChatContext::get_initial_context(config);
        let content = content.join("");
        chat_context.add_message(MessageRole::User, content.to_owned());

        commit_spinner.start("Generating the commit messages...");
        let commit_messages = chat_context.generate_messages(config, self.n).await?;
        commit_spinner.stop("📝 Commit messages generated successfully");

        let mut table = Table::new();
        table.set_format(*prettytable::format::consts::FORMAT_BOX_CHARS);
        let title_row = Row::new(vec![Cell::new_align(
            "Autocommit Messages",
            Alignment::CENTER,
        )
        .with_hspan(4)
        .with_style(Attr::ForegroundColor(color::GREEN))]);
        table.add_row(title_row);
        table.add_row(row![bFb->"Index", bFb->"Message", bFb->"Lines", bFb->"Chars"]);

        for (i, commit_message) in commit_messages.iter().enumerate() {
            let wrapped_message = fill(commit_message, 60);

            let num_lines = wrapped_message.lines().count();
            let num_chars = wrapped_message.chars().count();
            table.add_row(row![i, wrapped_message, num_lines, num_chars]);
        }

        table.printstd();

        debug!("Commit messages generated successfully");
        Ok(commit_messages)
    }

    pub async fn prompt_to_continue() -> anyhow::Result<bool> {
        let should_continue = Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("Do you want to continue?")
            .default(false)
            .interact()?;
        Ok(should_continue)
    }

    pub async fn prompt_for_remote() -> anyhow::Result<Option<String>> {
        let remotes = GitRepository::get_git_remotes()?;
        if remotes.is_empty() {
            eprintln!("  {}", "No remote repository found".yellow());
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
            Ok(None)
        }
    }

    pub async fn prompt_for_selected_message(commit_messages: &[String]) -> anyhow::Result<String> {
        let index = Input::<usize>::with_theme(&ColorfulTheme::default())
            .with_prompt(format!(
                "{}",
                "Enter the index of the message you want to commit:".green()
            ))
            .validate_with(|input: &usize| match input {
                index if *index < commit_messages.len() => Ok(()),
                _ => Err("Invalid index".to_string()),
            })
            .interact()?;

        let selected_message = commit_messages[index].clone();
        let copy_to_clipboard = Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt(format!(
                "{}",
                "Do you want to copy the selected message to clipboard?".green()
            ))
            .interact_opt()?
            .unwrap_or(true);

        if copy_to_clipboard {
            let mut clipboard: ClipboardContext =
                ClipboardProvider::new().map_err(|err| anyhow!(err.to_string()))?;
            clipboard
                .set_contents(selected_message.clone())
                .map_err(|err| anyhow!(err.to_string()))?;
            outro("Selected message copied to clipboard!");
        }

        Ok(selected_message)
    }

    pub async fn prompt_for_selected_files(
        changed_files: &[String],
    ) -> anyhow::Result<Vec<String>> {
        let selected_items = MultiSelect::with_theme(&ColorfulTheme::default())
            .with_prompt(format!(
                "Select the files you want to add to the commit ({} files changed):",
                changed_files.len().to_string().green()
            ))
            .items(changed_files)
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

    pub fn prompt_for_push() -> anyhow::Result<bool> {
        let push_confirmed_by_user = Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt(format!(
                "Do you want to push these changes to remote repository?",
            ))
            .default(true)
            .interact_opt()?
            .unwrap_or(true);

        if push_confirmed_by_user {
            Ok(true)
        } else {
            outro("Push cancelled, exiting...");
            Ok(false)
        }
    }

    pub fn prompt_for_pull(remote: &str) -> anyhow::Result<bool> {
        let pull_confirmed_by_user = Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt(format!(
                "Do you want to pull changes from the remote repository {} before pushing?",
                remote.green()
            ))
            .default(true)
            .interact_opt()?
            .unwrap_or(true);

        if pull_confirmed_by_user {
            Ok(true)
        } else {
            outro("Push cancelled, exiting...");
            Ok(false)
        }
    }
}
