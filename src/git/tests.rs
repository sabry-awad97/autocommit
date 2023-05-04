#[tokio::test]
async fn test_assert_git_repo_success() -> anyhow::Result<()> {
    use super::GitRepository;
    use std::env;
    use tokio::process::Command;
    // Store the original directory path
    let original_dir = env::current_dir()?;

    // Create a temporary directory and initialize a Git repository
    let temp_dir = tempfile::tempdir()?;

    // Switch to the temporary directory
    env::set_current_dir(&temp_dir)?;

    // Initialize a Git repository inside the temporary directory
    Command::new("git").arg("init").output().await.unwrap();

    // Assert that the Git repository assertion succeeds
    let result = GitRepository::assert_git_repo().await;
    assert!(result.is_ok());

    // Switch back to the original directory
    env::set_current_dir(original_dir)?;

    // Clean up the temporary directory
    temp_dir.close()?;

    Ok(())
}

#[tokio::test]
async fn test_assert_git_repo_failure() -> anyhow::Result<()> {
    use super::GitRepository;
    use std::env;

    // Store the original directory path
    let original_dir = env::current_dir()?;

    // Create a temporary directory without a Git repository
    let temp_dir = tempfile::tempdir()?;
    // Switch to the temporary directory
    env::set_current_dir(&temp_dir)?;

    // Assert that the Git repository assertion fails
    let result = GitRepository::assert_git_repo().await;

    assert!(result.is_err());

    // Switch back to the original directory
    env::set_current_dir(original_dir)?;

    // Clean up the temporary directory
    temp_dir.close()?;

    Ok(())
}
