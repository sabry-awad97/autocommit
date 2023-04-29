use std::{any, thread, time::Duration};

use crate::utils::{
    assert_git_repo, generate_message, get_changed_files, get_staged_diff, get_staged_files,
    git_add, outro, Message, MessageRole,
};
use anyhow::{anyhow, Result};
use spinners::{Spinner, Spinners};
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

        let staged_diff = get_staged_diff(&[]).await?;
        let _prompt = Message {
            role: MessageRole::User,
            content: get_prompt(config, &staged_diff),
        };

        let mut sp = Spinner::new(
            Spinners::CircleHalves,
            "\tAI is Thinking about your changes...".into(),
        );

        // let mesage: String = generate_message(&[prompt]).await?;
        thread::sleep(Duration::from_secs(2));
        sp.stop();
        // println!("{}", mesage);
        Ok(())
    }
}
