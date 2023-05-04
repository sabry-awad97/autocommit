use std::process::Output;

use crate::utils::outro;
use anyhow::anyhow;
use colored::Colorize;
use log::error;
use prettytable::{format::consts, Cell, Row, Table};
use tokio::process::Command;

pub struct GitRepository {}

impl GitRepository {
    pub async fn assert_git_repo() -> anyhow::Result<()> {
        let output = Command::new("git")
            .arg("rev-parse")
            .arg("--is-inside-work-tree")
            .output()
            .await
            .map_err(|e| {
                anyhow!(
                    "Command 'git rev-parse --is-inside-work-tree' failed: {}",
                    e
                )
            })?;

        if !output.status.success() {
            return Err(anyhow!(
                "The current working directory is not a Git repository"
            ));
        }

        Ok(())
    }

    pub async fn get_changed_files() -> anyhow::Result<Vec<String>> {
        let modified = Command::new("git")
            .arg("ls-files")
            .arg("--modified")
            .output()
            .await
            .map_err(|e| anyhow!("Command 'git ls-files --modified' failed: {}", e))?
            .stdout;

        let others = Command::new("git")
            .arg("ls-files")
            .arg("--others")
            .arg("--exclude-standard")
            .output()
            .await
            .map_err(|e| {
                anyhow!(
                    "Command 'git ls-files --others --exclude-standard' failed: {}",
                    e
                )
            })?
            .stdout;

        let mut files: Vec<_> = String::from_utf8(modified)?
            .split('\n')
            .chain(String::from_utf8(others)?.split('\n'))
            .filter_map(|s| {
                if s.is_empty() {
                    return None;
                }

                Some(String::from(s))
            })
            .collect();

        files.sort();
        Ok(files)
    }

    pub async fn get_staged_files() -> anyhow::Result<Vec<String>> {
        let top_level_dir = Command::new("git")
            .arg("rev-parse")
            .arg("--show-toplevel")
            .output()
            .await
            .map_err(|e| anyhow!("Command 'git rev-parse --show-toplevel' failed: {}", e))?
            .stdout;

        let top_level_dir_str = String::from_utf8_lossy(&top_level_dir);

        let output = Command::new("git")
            .arg("diff")
            .arg("--name-only")
            .arg("--cached")
            .arg("--relative")
            .arg(top_level_dir_str.trim_end())
            .output()
            .await
            .map_err(|e| {
                anyhow!(
                    "Command 'git diff --name-only --cached --relative {}' failed: {}",
                    top_level_dir_str.trim_end(),
                    e
                )
            })?;

        let output_str = String::from_utf8(output.stdout)?;
        let files = output_str.split('\n').filter(|s| !s.is_empty());

        // let ig = get_ignore_patterns().await?;

        let mut allowed_files: Vec<_> = files
            .filter(|_file| {
                // ig.matched(file, false).is_none()
                true
            })
            .map(|v| v.to_owned())
            .collect();

        allowed_files.sort();
        Ok(allowed_files)
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

    pub async fn git_add(files: &[String]) -> anyhow::Result<()> {
        let mut command = Command::new("git");
        command.arg("add").args(files);

        let mut child = command.spawn()?;

        let status = child.wait().await?;

        if !status.success() {
            return Err(anyhow!("Command 'git add' failed"));
        }

        Ok(())
    }

    pub async fn git_add_all() -> anyhow::Result<()> {
        let output = Command::new("git").arg("add").arg("--all").output().await?;

        if !output.status.success() {
            let error_message = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("Error: {}", error_message));
        }
        Ok(())
    }

    pub async fn git_commit(message: &str) -> anyhow::Result<String> {
        let output = Command::new("git")
            .arg("commit")
            .arg("-m")
            .arg(message)
            .output()
            .await
            .map_err(|e| anyhow!("Command 'git commit' failed: {}", e))?;

        if !output.status.success() {
            let error_message = String::from_utf8_lossy(&output.stderr).trim().to_string();
            error!("Failed to commit changes: {}", error_message);
            return Err(anyhow!(error_message));
        }

        let commit_output = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok(commit_output)
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

    pub fn get_git_email() -> anyhow::Result<String> {
        let output = std::process::Command::new("git")
            .arg("config")
            .arg("user.email")
            .output()?;

        parse_output(output)
    }

    pub fn get_git_name() -> anyhow::Result<String> {
        let output = std::process::Command::new("git")
            .arg("config")
            .arg("user.name")
            .output()?;

        parse_output(output)
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

fn parse_output(output: Output) -> anyhow::Result<String> {
    if output.status.success() {
        Ok(String::from_utf8(output.stdout)?.trim().to_owned())
    } else {
        Err(anyhow!(
            "Command failed with exit code {}: {}",
            output.status,
            String::from_utf8(output.stderr)?.trim()
        ))
    }
}
