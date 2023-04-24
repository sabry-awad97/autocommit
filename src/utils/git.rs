use anyhow::anyhow;
use tokio::process::Command;

pub async fn assert_git_repo() -> anyhow::Result<()> {
    let output = Command::new("git")
        .arg("rev-parse")
        .arg("--is-inside-work-tree")
        .output()
        .await
        .map_err(|e| anyhow!("Command 'git rev-parse --is-inside-work-tree' failed: {}", e))?;

    if !output.status.success() {
        panic!("The current working directory is not a Git repository");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn test_assert_git_repo() {
        assert!(assert_git_repo().await.is_ok());
    }
}
