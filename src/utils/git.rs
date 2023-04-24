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

fn exclude_from_diff(path: &str) -> String {
    format!(":(exclude){}", path)
}

pub fn get_detected_message(files: &[String]) -> String {
    format!(
        "Detected {} staged file{}",
        files.len(),
        if files.len() > 1 { "s" } else { "" }
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn test_assert_git_repo() {
        assert!(assert_git_repo().await.is_ok());
    }

    #[test]
    fn test_get_detected_message() {
        let files = vec!["file1.txt".to_string(), "file2.txt".to_string()];
        assert_eq!(get_detected_message(&files), "Detected 2 staged files");
        let files = vec!["file1.txt".to_string()];
        assert_eq!(get_detected_message(&files), "Detected 1 staged file");
    }
}
