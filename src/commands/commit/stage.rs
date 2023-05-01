use anyhow::anyhow;

use crate::git::GitRepository;

pub async fn stage_all_changed_files() -> anyhow::Result<()> {
    let changed_files = GitRepository::get_changed_files().await?;

    if !changed_files.is_empty() {
        GitRepository::git_add(&changed_files).await?;
    } else {
        return Err(anyhow!(
            "No changes detected, write some code and run again"
        ));
    }

    Ok(())
}
