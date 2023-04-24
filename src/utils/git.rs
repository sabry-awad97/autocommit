use anyhow::anyhow;
use tokio::process::Command;

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
        panic!("The current working directory is not a Git repository");
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

pub async fn get_staged_diff(files: &[String]) -> anyhow::Result<String> {
    let lock_files = files
        .iter()
        .filter(|file| file.contains(".lock") || file.contains("-lock."))
        .map(|s| format!(":(exclude){}", s))
        .collect::<Vec<_>>();

    if !lock_files.is_empty() {
        eprintln!("Some files are '.lock' files which are excluded by default from 'git diff':\n");
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

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn test_assert_git_repo() {
        assert!(assert_git_repo().await.is_ok());
    }
}
