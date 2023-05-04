use std::path::Path;

use crate::utils::outro;
use anyhow::anyhow;
use colored::Colorize;
use git2::{Repository, Signature, Status, StatusOptions};
use ignore::{
    gitignore::{Gitignore, GitignoreBuilder},
    WalkBuilder,
};
use log::error;
use prettytable::{Cell, Row, Table};
use tokio::process::Command;
mod tests;

pub struct GitRepository {}

impl GitRepository {
    pub async fn assert_git_repo() -> anyhow::Result<()> {
        Repository::open_from_env().map_err(|err| {
            anyhow!(
                "The current working directory is not a Git repository: {}",
                err
            )
        })?;
        Ok(())
    }

    pub fn get_changed_files() -> anyhow::Result<Vec<String>> {
        let repo = Repository::open_from_env().map_err(|err| {
            anyhow!(
                "The current working directory is not a Git repository: {}",
                err
            )
        })?;
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
        let repo = Repository::open_from_env().map_err(|err| {
            anyhow!(
                "The current working directory is not a Git repository: {}",
                err
            )
        })?;
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
        let repo = Repository::open_from_env().map_err(|err| {
            anyhow!(
                "The current working directory is not a Git repository: {}",
                err
            )
        })?;
        let mut index = repo
            .index()
            .map_err(|err| anyhow!("Failed to open the Git index: {}", err))?;

        for file in files {
            let path = Path::new(file);
            if path.is_file() {
                index.add_path(path).map_err(|err| {
                    anyhow!("Failed to add file '{}' to the Git index: {}", file, err)
                })?;
            } else {
                eprintln!("  ⏭️ {} '{}'", "Skipping directory".yellow(), file);
            }
        }

        index
            .write()
            .map_err(|err| anyhow!("Failed to write the Git index: {}", err))?;

        Ok(())
    }

    pub fn git_add_all() -> anyhow::Result<()> {
        let repo = Repository::open_from_env().map_err(|err| {
            anyhow!(
                "The current working directory is not a Git repository: {}",
                err
            )
        })?;
        let mut index = repo.index()?;

        index
            .add_all(["."].iter(), git2::IndexAddOption::DEFAULT, None)
            .map_err(|err| anyhow!("Failed to add files to the Git index: {}", err))?;

        index
            .write()
            .map_err(|err| anyhow!("Failed to write the Git index: {}", err))?;

        Ok(())
    }

    pub async fn get_commit_summary_table(message: &str, name: &str, email: &str) -> anyhow::Result<Table> {
        let repo = Repository::open_from_env()?;
        let tree_id = repo.index()?.write_tree()?;
        let tree = repo.find_tree(tree_id)?;
        let head = repo.head()?;
        let head_commit = head.peel_to_commit();
        let committer = Signature::now(&name, &email)?;
        let commit_id = if let Some(head_commit) = head_commit.ok() {
            repo.commit(
                Some("HEAD"),
                &committer,
                &committer,
                message,
                &tree,
                &[&head_commit],
            )?
        } else {
            repo.commit(Some("HEAD"), &committer, &committer, message, &tree, &[])?
        };
        let commit = repo.find_commit(commit_id)?;
        let branch_name = head.name().unwrap_or("Unknown");
        let commit_count = Self::get_commit_count(&repo)?;
        let (files_changes, insertions, deletions) = Self::get_short_stat()?;
        let commit_hash = commit.id().to_string();
        // Display a table of commit information
        let mut table = Table::new();
        table.set_titles(Row::new(vec![
            Cell::new("Commit Information").style_spec("bFy")
        ]));
        table.add_row(Row::new(vec![
            Cell::new("Branch"),
            Cell::new("Commit Hash"),
        ]));
        table.add_row(Row::new(vec![
            Cell::new(branch_name),
            Cell::new(&commit_hash),
        ]));
        table.add_row(Row::new(vec![
            Cell::new("Author"),
            Cell::new("Email"),
            Cell::new("Commit Count"),
        ]));
        table.add_row(Row::new(vec![
            Cell::new(&name),
            Cell::new(&email),
            Cell::new(&commit_count.to_string()),
        ]));
        table.add_row(Row::new(vec![
            Cell::new("Files Changed"),
            Cell::new("Insertions"),
            Cell::new("Deletions"),
        ]));
        table.add_row(Row::new(vec![
            Cell::new(&files_changes.to_string()),
            Cell::new(&insertions.to_string()),
            Cell::new(&deletions.to_string()),
        ]));

        Ok(table)
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

    pub fn get_git_remotes() -> anyhow::Result<Vec<String>> {
        let repo = Repository::open_from_env().map_err(|err| {
            anyhow!(
                "The current working directory is not a Git repository: {}",
                err
            )
        })?;
        let remotes = repo
            .remotes()?
            .into_iter()
            .filter_map(|remote| remote)
            .map(|remote| remote.to_owned())
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
        let repo =
            Repository::open_from_env().map_err(|e| anyhow!("Failed to open repository: {}", e))?;

        let mut options = StatusOptions::new();
        options.include_untracked(true);
        let statuses = repo
            .statuses(Some(&mut options))
            .map_err(|e| anyhow!("Failed to get repository status: {}", e))?;

        if statuses.is_empty() {
            return Err(anyhow!("No changes to commit."));
        }

        let mut table = Table::new();
        table.set_format(*prettytable::format::consts::FORMAT_BOX_CHARS);
        table.add_row(Row::new(vec![Cell::new("Status"), Cell::new("File")]));

        for entry in statuses.iter() {
            let status = match entry.status() {
                s if s.contains(Status::WT_NEW) => "Untracked",
                s if s.contains(Status::WT_MODIFIED) => "Modified",
                s if s.contains(Status::WT_DELETED) => "Deleted",
                s if s.contains(Status::INDEX_NEW) => "Added",
                s if s.contains(Status::INDEX_MODIFIED) => "Staged",
                s if s.contains(Status::INDEX_DELETED) => "Removed",
                _ => continue,
            };
            let file = entry.path().unwrap_or("");
            table.add_row(Row::new(vec![Cell::new(status), Cell::new(file)]));
        }

        Ok(table.to_string())
    }

    pub fn get_commit_count(repo: &Repository) -> anyhow::Result<usize> {
        let head = repo.head()?;
        let head_oid = head
            .target()
            .ok_or_else(|| anyhow!("HEAD is not a direct reference"))?;
        let mut revwalk = repo.revwalk()?;
        revwalk.push(head_oid)?;
        let count = revwalk.count();
        Ok(count)
    }

    fn get_short_stat() -> anyhow::Result<(usize, usize, usize)> {
        // Open the repository in the current directory
        let repo = Repository::open_from_env()?;

        // Get the HEAD commit
        let head = repo.head()?.peel_to_commit()?;

        // Get the tree for the HEAD commit
        let tree = head.tree()?;

        // Get the diff between the HEAD commit and its parent
        let parent_commit = head.parent(0)?;
        let parent_tree = parent_commit.tree()?;
        let diff = repo.diff_tree_to_tree(Some(&parent_tree), Some(&tree), None)?;

        // Get the number of insertions and deletions in the diff
        let stats = diff.stats()?;
        let insertions = stats.insertions();
        let deletions = stats.deletions();
        let files_changed = stats.files_changed();

        Ok((files_changed, insertions, deletions))
    }

}
