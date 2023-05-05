use crate::{
    commands::commit::chat_context::ChatContext,
    git::GitRepository,
    utils::{outro, spinner, MessageRole},
};
use anyhow::anyhow;
use colored::{Color, Colorize};
use dialoguer::{theme::ColorfulTheme, Confirm, MultiSelect};
use log::{debug, info};
use structopt::StructOpt;

use super::config::AutocommitConfig;

mod chat_context;

#[derive(Debug, StructOpt)]
pub struct CommitCommand {
    #[structopt(short, long)]
    stage_all: bool,
    #[structopt(short, long)]
    branch_name: Option<String>,
    #[structopt(long)]
    skip_chatbot: bool,
    #[structopt(long)]
    skip_push_confirmation: bool,
    #[structopt(long)]
    skip_commit_confirmation: bool,
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
                    GitRepository::git_add(&files)?;
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
            let commit_message = if self.skip_chatbot {
                if let Some(default_message) = config
                    .config_data
                    .default_commit_message
                    .get_value_ref()
                    .get_inner_value()
                {
                    outro(&format!(
                        "Using default commit message:\n{}\n",
                        default_message
                    ));
                    default_message.clone()
                } else {
                    return Err(anyhow!("No default commit message provided"));
                }
            } else {
                self.generate_autocommit_message(config, &staged_diffs)
                    .await?
            };

            // Prompt the user to confirm the commit message
            if let Ok(Some(new_message)) = self
                .prompt_to_commit_changes(config, &staged_diffs, &commit_message)
                .await
            {
                self.commit_changes(config, &new_message).await?;
                // Prompt the user to confirm the push
                if Self::prompt_for_push(config)? || self.skip_push_confirmation {
                    // Prompt the user to select a remote repository
                    if let Some(remote) = Self::prompt_for_remote().await? {
                        // Pull changes from the remote repository if necessary
                        if Self::prompt_for_pull(&remote)? {
                            Self::pull_changes(&remote).await?;
                        }
                        // Push changes to the remote repository
                        Self::push_changes(&remote, self.branch_name.clone()).await?;
                        info!("Autocommit process completed successfully");
                    }
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

        if let Some(branch_name) = &self.branch_name {
            GitRepository::git_checkout_new_branch(branch_name).await?;
            GitRepository::git_add_all()?;
        }
        let name = config.config_data.name.get_value_ref();
        let email = config.config_data.email.get_value_ref();

        let commit_table =
            GitRepository::get_commit_summary_table(commit_message, name, email).await?;

        commit_spinner.stop(&format!("{} Changes committed successfully", "✔".green()));
        commit_table.printstd();

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

    pub async fn push_changes(remote: &str, branch_name: Option<String>) -> anyhow::Result<()> {
        let mut push_spinner = spinner();
        push_spinner.start(&format!(
            "Pushing changes to remote repository {}...",
            remote.green().bold()
        ));
        GitRepository::git_push(remote, branch_name).await?;
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

    pub async fn prompt_to_commit_changes(
        &self,
        config: &AutocommitConfig,
        staged_diffs: &[String],
        commit_message: &str,
    ) -> anyhow::Result<Option<String>> {
        let mut message = commit_message.to_string();

        if !self.skip_commit_confirmation {
            loop {
                let is_generate_new_message_confirmed_by_user =
                    Confirm::with_theme(&ColorfulTheme::default())
                        .with_prompt("Do you want to generate a new commit message?")
                        .default(false)
                        .interact()?;
                if is_generate_new_message_confirmed_by_user {
                    let mut new_content = Vec::new();
                    new_content.push(
                        "Suggest a professional git commit message with gitmoji\n".to_string(),
                    );
                    new_content.push("Exclude anything unnecessary such as the original translation — your entire response will be passed directly into git commit.\n".to_string());
                    for staged_diff in staged_diffs {
                        new_content.push(staged_diff.clone());
                    }
                    message = self
                        .generate_autocommit_message(config, &new_content)
                        .await?;
                } else {
                    break;
                }
            }

            let preview_confirmed_by_user = Confirm::with_theme(&ColorfulTheme::default())
                .with_prompt("Do you want to preview the changes before committing?")
                .default(false)
                .interact_opt()?
                .unwrap_or(false);

            if preview_confirmed_by_user {
                outro(&format!(
                    "Staged diff:\n{}\n",
                    staged_diffs.join("").color(Color::TrueColor {
                        r: 128,
                        g: 128,
                        b: 128
                    })
                ));

                let commit_confirmed_by_user = Confirm::with_theme(&ColorfulTheme::default())
                    .with_prompt("Do you want to commit these changes?")
                    .default(true)
                    .interact_opt()?;

                if let Some(false) = commit_confirmed_by_user {
                    outro("Commit cancelled, exiting...");
                    return Ok(None);
                }
            }
        } else if let Some(default_commit_behavior) = &config
            .config_data
            .default_commit_behavior
            .get_value_ref()
            .get_inner_value()
        {
            outro(&format!(
                "Using default commit behavior: {}\n",
                default_commit_behavior
            ));

            if default_commit_behavior.is_no() {
                outro("Commit cancelled, exiting...");
                return Ok(None);
            }
        }

        Ok(Some(message))
    }

    pub async fn generate_autocommit_message(
        &self,
        config: &AutocommitConfig,
        content: &[String],
    ) -> anyhow::Result<String> {
        const GENERATING_MESSAGE: &str = "Generating the commit message...";
        let mut commit_spinner = spinner();
        let commit_message;

        match &config
            .config_data
            .default_commit_message
            .get_value_ref()
            .get_inner_value()
        {
            Some(default_message) => {
                outro(&format!(
                    "Using default commit message:\n{}\n",
                    default_message
                ));
                commit_message = default_message.clone();
            }
            _ => {
                commit_spinner.start(GENERATING_MESSAGE);
                let mut chat_context = ChatContext::get_initial_context(config);
                let content = content.join("");
                chat_context.add_message(MessageRole::User, content.to_owned());
                commit_message = chat_context.generate_message(config).await?;
                commit_spinner.stop("📝 Commit message generated successfully");
            }
        }

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
        debug!("Commit message generated successfully");
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

    pub fn prompt_for_push(config: &AutocommitConfig) -> anyhow::Result<bool> {
        let push_confirmed_by_user = match &config
            .config_data
            .default_push_behavior
            .get_value_ref()
            .get_inner_value()
        {
            Some(default_push_behavior) => {
                outro(&format!(
                    "Using default push behavior: {}\n",
                    default_push_behavior
                ));
                if default_push_behavior.is_no() {
                    false
                } else if default_push_behavior.is_yes() {
                    true
                } else {
                    Confirm::with_theme(&ColorfulTheme::default())
                        .with_prompt(format!(
                            "Do you want to push these changes to remote repository?",
                        ))
                        .default(true)
                        .interact_opt()?
                        .unwrap_or(false)
                }
            }
            _ => Confirm::with_theme(&ColorfulTheme::default())
                .with_prompt(format!(
                    "Do you want to push these changes to remote repository?",
                ))
                .default(true)
                .interact_opt()?
                .unwrap_or(true),
        };

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
                remote
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
