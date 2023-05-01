use dialoguer::{theme::ColorfulTheme, Confirm, MultiSelect};

use crate::{git::GitRepository, utils::outro};
use anyhow::anyhow;

pub async fn prompt_to_continue() -> anyhow::Result<bool> {
    let should_continue = Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("Do you want to continue?")
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

pub async fn prompt_for_selected_files(changed_files: &[String]) -> anyhow::Result<Vec<String>> {
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

pub fn prompt_for_push(remote: &str) -> anyhow::Result<bool> {
    let push_confirmed_by_user = Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt(format!(
            "Do you want to push these changes to the remote repository {}?",
            remote
        ))
        .interact_opt()?;

    if let Some(true) = push_confirmed_by_user {
        outro(&format!("Changes pushed to remote repository {}", remote));
        Ok(true)
    } else {
        outro("Push cancelled, exiting...");
        Ok(false)
    }
}