use std::path::Path;

use crate::utils::outro;
use anyhow::anyhow;
use colored::Colorize;
use git2::{DiffOptions, Repository, RepositoryOpenFlags, Signature, Status, StatusOptions};
use ignore::{
    gitignore::{Gitignore, GitignoreBuilder},
    WalkBuilder,
};
use log::error;
use prettytable::{Cell, Row, Table};
mod commit_table;
use tokio::process::Command;

use self::commit_table::CommitSummary;
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
            if (status.status().contains(Status::INDEX_MODIFIED)
                || status.status().contains(Status::INDEX_NEW))
                && ignore_patterns
                    .matched_path_or_any_parents(&path, false)
                    .is_none()
            {
                files.push(path);
            }
        }

        files.sort();
        Ok(files)
    }

    pub fn get_staged_file_diffs(files: &[String]) -> anyhow::Result<Vec<String>> {
        let mut diff_opts = DiffOptions::new();
        let mut excluded_files = Vec::new();
        for file in files {
            if file.ends_with(".lock") {
                excluded_files.push(file.clone());
            } else {
                diff_opts.pathspec(file);
            }
        }

        let repo =
            Repository::open_ext(".", RepositoryOpenFlags::empty(), std::path::Path::new(""))
                .map_err(|e| anyhow!("Failed to open repository: {}", e))?;

        let head_tree = match repo.head().and_then(|head| head.peel_to_tree()) {
            Ok(tree) => Some(tree),
            Err(e) => {
                if e.code() == git2::ErrorCode::UnbornBranch {
                    None
                } else {
                    return Err(anyhow!("Failed to get HEAD tree: {}", e));
                }
            }
        };

        let mut index = repo
            .index()
            .map_err(|e| anyhow!("Failed to get index: {}", e))?;

        let staged_tree_oid = index
            .write_tree()
            .map_err(|e| anyhow!("Failed to write index tree: {}", e))?;

        let staged_tree = repo
            .find_tree(staged_tree_oid)
            .map_err(|e| anyhow!("Failed to find staged tree: {}", e))?;

        let diff = repo
            .diff_tree_to_tree(head_tree.as_ref(), Some(&staged_tree), Some(&mut diff_opts))
            .map_err(|e| anyhow!("Failed to get diff: {}", e))?;

        let mut diff_text = Vec::new();
        diff.print(git2::DiffFormat::Patch, |_delta, _, line| {
            let text = String::from_utf8_lossy(line.content());
            let line_text = format!("{}{}", line.origin(), text);
            match line.origin() {
                '+' | '-' => {
                    diff_text.push(line_text);
                }
                _ => {
                    diff_text.push(line_text[1..].to_owned());
                }
            }
            true
        })
        .map_err(|e| anyhow!("Failed to print diff: {}", e))?;

        if !excluded_files.is_empty() {
            outro("Some files are '.lock' files which are excluded by default from 'git diff':");
            for file in &excluded_files {
                eprintln!("  {} {}", ":(exclude)".red(), file);
            }
            eprintln!("No commit messages are generated for these files.");
        }

        Ok(diff_text)
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
                eprintln!("  {} '{}'", "Skipping directory".yellow(), file);
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

    pub async fn get_commit_summary_table(
        message: &str,
        name: &str,
        email: &str,
    ) -> anyhow::Result<Table> {
        let repo = Repository::open_from_env()?;
        let status = repo.statuses(None)?;
        let mut has_staged_changes = false;
        for entry in status.iter() {
            if entry.status() != git2::Status::CURRENT {
                has_staged_changes = true;
                break;
            }
        }

        if !has_staged_changes {
            let message = String::from("Failed to commit. Have you manually committed recently?");
            return Err(anyhow::anyhow!(message));
        }

        let tree_id = if let Ok(_) = repo.head()?.peel_to_commit() {
            repo.index()?
                .write_tree()
                .map_err(|e| anyhow::anyhow!("Failed to write tree: {}", e))?
        } else {
            let tree = repo.treebuilder(None)?;
            tree.write()
                .map_err(|e| anyhow::anyhow!("Failed to write tree: {}", e))?
        };

        let tree = repo
            .find_tree(tree_id)
            .map_err(|e| anyhow::anyhow!("Failed to find tree: {}", e))?;
        let head = repo
            .head()
            .map_err(|e| anyhow::anyhow!("Failed to get repository head: {}", e))?;
        let head_commit = head.peel_to_commit();
        let committer = Signature::now(name, email)
            .map_err(|e| anyhow::anyhow!("Failed to create signature: {}", e))?;
        let commit_id = if let Ok(head_commit) = head_commit {
            repo.commit(
                Some("HEAD"),
                &committer,
                &committer,
                message,
                &tree,
                &[&head_commit],
            )
            .map_err(|e| anyhow::anyhow!("Failed to commit changes: {}", e))?
        } else {
            repo.commit(Some("HEAD"), &committer, &committer, message, &tree, &[])
                .map_err(|e| anyhow::anyhow!("Failed to commit changes: {}", e))?
        };
        let commit = repo
            .find_commit(commit_id)
            .map_err(|e| anyhow::anyhow!("Failed to find commit: {}", e))?;
        let branch_name = head.name().unwrap_or("Unknown");
        let commit_count = Self::get_commit_count(&repo)
            .map_err(|e| anyhow::anyhow!("Failed to get commit count: {}", e))?;
        let (files_changed, insertions, deletions) = Self::get_short_stat()
            .map_err(|e| anyhow::anyhow!("Failed to get short stat: {}", e))?;
        let commit_hash = commit.id().to_string();
        let commit_summary = CommitSummary {
            branch_name: branch_name.to_string(),
            commit_hash,
            author_name: name.to_string(),
            author_email: email.to_string(),
            commit_count,
            files_changed,
            insertions,
            deletions,
        };
        let table = commit_summary.get_table();

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
            .flatten()
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
        let head = repo
            .head()
            .map_err(|e| anyhow!("Failed to get HEAD reference: {}", e))?;
        let head_oid = head
            .target()
            .ok_or_else(|| anyhow!("HEAD is not a direct reference"))?;
        let mut revwalk = repo
            .revwalk()
            .map_err(|e| anyhow!("Failed to create Revwalk object: {}", e))?;
        revwalk
            .push(head_oid)
            .map_err(|e| anyhow!("Failed to push HEAD commit onto Revwalk: {}", e))?;
        let count = revwalk.count();
        Ok(count)
    }

    fn get_short_stat() -> anyhow::Result<(usize, usize, usize)> {
        // Open the repository in the current directory
        let repo =
            Repository::open_from_env().map_err(|e| anyhow!("Failed to open repository: {}", e))?;

        // Get the HEAD commit
        let head = repo
            .head()?
            .peel_to_commit()
            .map_err(|e| anyhow!("Failed to get HEAD commit: {}", e))?;

        // Get the tree for the HEAD commit
        let tree = head
            .tree()
            .map_err(|e| anyhow!("Failed to get tree for HEAD commit: {}", e))?;

        // Get the parent commit of the HEAD commit
        let parent_commit = head
            .parent(0)
            .map_err(|e| anyhow!("Failed to get parent commit: {}", e))?;

        // Get the tree for the parent commit
        let parent_tree = parent_commit
            .tree()
            .map_err(|e| anyhow!("Failed to get tree for parent commit: {}", e))?;

        // Get the diff between the parent tree and the HEAD tree
        let diff = repo
            .diff_tree_to_tree(Some(&parent_tree), Some(&tree), None)
            .map_err(|e| anyhow!("Failed to get diff: {}", e))?;

        // Get the number of insertions and deletions in the diff
        let stats = diff
            .stats()
            .map_err(|e| anyhow!("Failed to get diff stats: {}", e))?;
        let insertions = stats.insertions();
        let deletions = stats.deletions();
        let files_changed = stats.files_changed();

        Ok((files_changed, insertions, deletions))
    }
}
