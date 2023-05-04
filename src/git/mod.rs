use std::path::Path;

use crate::utils::outro;
use anyhow::anyhow;
use colored::Colorize;
use git2::{Repository, Status, StatusOptions};
use ignore::{
    gitignore::{Gitignore, GitignoreBuilder},
    WalkBuilder,
};
use log::error;
use prettytable::{format::consts, Cell, Row, Table};
use tokio::process::Command;
mod tests;

pub struct GitRepository {}

impl GitRepository {
    pub async fn assert_git_repo() -> anyhow::Result<()> {
        Repository::open_from_env().map_err(|e| {
            anyhow!(
                "The current working directory is not a Git repository: {}",
                e
            )
        })?;
        Ok(())
    }

    pub fn get_changed_files() -> anyhow::Result<Vec<String>> {
        let repo = Repository::open_from_env()?;
        let mut opts = StatusOptions::new();
        opts.include_untracked(true);
        let statuses = repo.statuses(Some(&mut opts))?;

        let mut files = Vec::new();
        for status in statuses.iter() {
            let path = status.path().unwrap().to_string();
            if status.status().contains(git2::Status::WT_MODIFIED)
                || status.status().contains(git2::Status::INDEX_MODIFIED)
                || status.status().contains(git2::Status::WT_NEW)
                || status.status().contains(git2::Status::INDEX_NEW)
            {
                files.push(path);
            }
        }
        files.sort();
        Ok(files)
    }

    pub fn get_ignore_patterns() -> anyhow::Result<Gitignore> {
        let top_level_dir = std::env::current_dir()?;
        let mut ignore_file_paths = Vec::new();

        // Find all .gitignore files in the repository
        for result in WalkBuilder::new(&top_level_dir)
            .hidden(false)
            .ignore(false)
            .git_ignore(false)
            .git_exclude(false)
            .build()
        {
            let entry = result?;
            let pat = ".autoignore";
            if entry.file_type().map_or(false, |t| t.is_file())
                && entry.file_name().to_string_lossy().ends_with(pat)
            {
                ignore_file_paths.push(entry.path().to_owned());
            }
        }

        // Create a Gitignore object from all the .gitignore files
        let mut ig = GitignoreBuilder::new("");
        for path in ignore_file_paths {
            ig.add(path);
        }

        Ok(ig.build()?)
    }

    pub fn get_staged_files() -> anyhow::Result<Vec<String>> {
        let repo = Repository::open_from_env()?;
        let mut opts = StatusOptions::new();
        opts.include_untracked(false);
        let statuses = repo.statuses(Some(&mut opts))?;

        let ignore_patterns = Self::get_ignore_patterns()?;
        let mut files = Vec::new();
        for status in statuses.iter() {
            let path = status.path().unwrap().to_string();
            if status.status().contains(Status::INDEX_MODIFIED)
                || status.status().contains(Status::INDEX_NEW)
            {
                if ignore_patterns
                    .matched_path_or_any_parents(&path, false)
                    .is_none()
                {
                    files.push(path);
                }
            }
        }

        files.sort();
        Ok(files)
    }

    pub async fn get_staged_diff(files: &[String]) -> anyhow::Result<String> {
        let lock_files = files
            .iter()
            .filter(|file| file.contains(".lock") || file.contains("-lock."))
            .map(|s| format!("  {} {}", ":(exclude)".red(), s))
            .collect::<Vec<_>>();

        if !lock_files.is_empty() {
            outro("Some files are '.lock' files which are excluded by default from 'git diff':");
            for file in &lock_files {
                eprintln!("{}", file);
            }
            eprintln!("No commit messages are generated for these files.");
        }

        let files_without_locks = files
            .iter()
            .filter(|file| !file.contains(".lock") && !file.contains("-lock."))
            .cloned()
            .collect::<Vec<_>>();

        let output = Command::new("git")
            .arg("diff")
            .arg("--staged")
            .args(files_without_locks)
            .output()
            .await
            .map_err(|e| anyhow!("Failed to run git command: {}", e))?
            .stdout;

        let diff = String::from_utf8_lossy(&output).trim().to_owned();
        Ok(diff)
    }

    pub fn git_add(files: &[String]) -> anyhow::Result<()> {
        let repo = Repository::open_from_env()?;
        let mut index = repo.index()?;

        for file in files {
            index.add_path(Path::new(file))?;
        }

        index.write()?;

        Ok(())
    }

    pub fn git_add_all() -> anyhow::Result<()> {
        let repo = Repository::open_from_env()?;
        let mut index = repo.index()?;

        index.add_all(["."].iter(), git2::IndexAddOption::DEFAULT, None)?;

        index.write()?;

        Ok(())
    }

    pub async fn git_commit(message: &str, name: &str, email: &str) -> anyhow::Result<String> {
        let output = Command::new("git")
            .arg("commit")
            .arg("-m")
            .arg(message)
            .arg("--author")
            .arg(format!("{} <{}>", name, email))
            .output()
            .await
            .map_err(|e| anyhow!("Command 'git commit' failed: {}", e))?;

        let stdout = String::from_utf8(output.stdout)?;
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();

        if !output.status.success() {
            error!("Failed to commit changes: {}", stderr);
            return Err(anyhow!(stderr));
        }

        let lines: Vec<&str> = stdout.trim().split('\n').collect();
        let commit_info = lines[0].trim();
        let commit_hash = commit_info.split(' ').nth(1).unwrap_or("");
        let branch_name = commit_info.split(' ').nth(0).unwrap_or("");
        let commit_info = format!("{} {}", branch_name, commit_hash);
        let last_line = lines.last().unwrap_or(&"").trim();
        let output = format!("{} {}", commit_info, last_line);
        Ok(output)
    }

    pub async fn git_pull(remote: &str) -> anyhow::Result<()> {
        let output = Command::new("git").arg("pull").arg(remote).output().await?;

        if !output.status.success() {
            let error_message = String::from_utf8_lossy(&output.stderr);
            error!(
                "Failed to pull changes from remote repository {}: {}",
                remote, error_message
            );
            return Err(anyhow!(
                "Failed to pull changes from remote repository {}: {}",
                remote,
                error_message
            ));
        }
        Ok(())
    }

    pub async fn git_push(remote: &str, branch: Option<String>) -> anyhow::Result<()> {
        let mut command = Command::new("git");
        command.arg("push").arg("--verbose").arg(remote);
        if let Some(branch_name) = branch {
            command.arg(branch_name);
        }
        let output = command.output().await?;

        if !output.status.success() {
            let error_message = String::from_utf8_lossy(&output.stderr);
            error!(
                "Failed to push changes to remote repository {}: {}",
                remote, error_message
            );
            return Err(anyhow!(
                "Failed to push changes to remote repository {}: {}",
                remote,
                error_message
            ));
        }
        Ok(())
    }

    pub async fn get_git_remotes() -> anyhow::Result<Vec<String>> {
        let output = Command::new("git")
            .arg("remote")
            .output()
            .await
            .map_err(|e| anyhow!("failed to execute 'git remote' command: {}", e))?;

        if !output.status.success() {
            let error_message = String::from_utf8_lossy(&output.stderr).trim().to_string();
            error!("Failed to get git remotes: {}", error_message);
            return Err(anyhow!(error_message));
        }

        let remotes = String::from_utf8(output.stdout)?
            .split('\n')
            .filter(|remote| !remote.trim().is_empty())
            .map(|remote| remote.to_string())
            .collect();

        Ok(remotes)
    }

    pub fn get_git_user_email() -> anyhow::Result<String> {
        let repo = Repository::open_from_env()?;
        let config = repo.config()?;
        let email = config.get_string("user.email")?;
        Ok(email)
    }

    pub fn get_git_user_name() -> anyhow::Result<String> {
        let repo = Repository::open_from_env()?;
        let config = repo.config()?;
        let email = config.get_string("user.name")?;
        Ok(email)
    }

    pub async fn git_checkout_new_branch(branch_name: &str) -> anyhow::Result<()> {
        let mut cmd = Command::new("git");
        cmd.arg("checkout").arg("-b").arg(branch_name);
        let output = cmd.output().await?;

        if !output.status.success() {
            let output: String = String::from_utf8_lossy(&output.stderr).trim().to_string();
            let error_message =
                format!("Failed to checkout new branch {}: {}", branch_name, output);
            error!("{}", error_message);
            anyhow::bail!(
                "Failed to checkout new branch {}: {}",
                branch_name,
                error_message
            );
        }

        Ok(())
    }

    pub async fn git_status() -> anyhow::Result<String> {
        let status_output = Command::new("git")
            .args(&["status", "--porcelain", "-u"])
            .output()
            .await
            .map_err(|e| anyhow!("Failed to execute git status command: {}", e))?;

        let status_lines = String::from_utf8_lossy(&status_output.stdout);
        if status_lines.trim().is_empty() {
            return Err(anyhow!("No changes to commit."));
        }

        let mut table = Table::new();
        table.set_format(*consts::FORMAT_BOX_CHARS);
        table.add_row(Row::new(vec![Cell::new("Status"), Cell::new("File")]));
        for line in status_lines.lines() {
            let mut cells = line.split_whitespace();
            let status = cells.next().unwrap_or("");
            let file = cells.next().unwrap_or("");
            table.add_row(Row::new(vec![Cell::new(status), Cell::new(file)]));
        }
        Ok(table.to_string())
    }
}
