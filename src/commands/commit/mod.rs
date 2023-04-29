use std::{thread, time::Duration};

use crate::utils::{
    assert_git_repo, generate_message, get_changed_files, get_staged_diff, get_staged_files,
    git_add, spinner, Message, MessageRole,
};

use anyhow::{anyhow, Result};
use structopt::StructOpt;

use super::config::AutocommitConfig;

#[derive(Debug, StructOpt)]
pub struct CommitCommand {}

fn get_prompt(config: &AutocommitConfig, diff: &str) -> String {
    let language = format!("{:?}", config.config_data.language).to_lowercase();
    format!("Write a git commit message in present tense for the following diff without prefacing it with anything. \
    Do not be needlessly verbose and make sure the answer is concise and to the point. \
    The response must be in the language {}: \n{}", language, diff)
}

impl CommitCommand {
    pub async fn run(&self, config: &AutocommitConfig, stage_all: bool) -> Result<()> {
        assert_git_repo().await?;

        if stage_all {
            let changed_files = get_changed_files().await?;

            if !changed_files.is_empty() {
                git_add(&changed_files).await?;
            } else {
                return Err(anyhow!(
                    "No changes detected, write some code and run again"
                ));
            }
        }

        let staged_files = get_staged_files().await?;
        let changed_files = get_changed_files().await?;

        if staged_files.is_empty() && changed_files.is_empty() {
            return Err(anyhow!(
                "No changes detected, write some code and run again"
            ));
        }

        let mut staged_spinner = spinner("Counting staged files");
        staged_spinner
            .start()
            .map_err(|_| anyhow!("Error starting spinner"))?;

        if staged_files.is_empty() {
            staged_spinner
                .stop()
                .map_err(|_| anyhow!("Error stopping spinner"))?;
        }

        let staged_diff = get_staged_diff(&[]).await?;
        let _prompt = Message {
            role: MessageRole::User,
            content: get_prompt(config, &staged_diff),
        };

        // let mesage: String = generate_message(&[prompt]).await?;
        thread::sleep(Duration::from_secs(2));
        // staged_spinner.stop();
        println!("\nDone!");
        // println!("{}", mesage);
        Ok(())
    }
}
