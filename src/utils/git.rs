use anyhow::anyhow;
use tokio::process::Command;

struct GitRepository {}

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

        Ok(String::from_utf8(modified)?
            .split('\n')
            .chain(String::from_utf8(others)?.split('\n'))
            .filter_map(|s| {
                if s.is_empty() {
                    return None;
                }

                Some(String::from(s))
            })
            .collect())
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

        Ok(files
            .filter(|_file| {
                // ig.matched(file, false).is_none()
                true
            })
            .map(|v| v.to_owned())
            .collect())
    }

    pub async fn get_staged_diff(files: &[String]) -> anyhow::Result<String> {
        let lock_files = files
            .iter()
            .filter(|file| file.contains(".lock") || file.contains("-lock."))
            .map(|s| format!(":(exclude){}", s))
            .collect::<Vec<_>>();

        if !lock_files.is_empty() {
            eprintln!(
                "Some files are '.lock' files which are excluded by default from 'git diff':\n"
            );
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
}
