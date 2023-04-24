use anyhow::Result;
use structopt::StructOpt;

use crate::utils::{
    assert_git_repo, generate_message, get_staged_diff, git_add_all, Message, MessageRole,
};

use super::config::AutocommitConfig;

#[derive(Debug, StructOpt)]
pub struct CommitCommand {}

fn get_prompt(config: &AutocommitConfig, diff: &str) -> String {
    let language = format!("{:?}", config.config_data.language).to_lowercase();
    format!("Write a git commit message in present tense for the following diff without prefacing it with anything. Do not be needlessly verbose and make sure the answer is concise and to the point. The response must be in the language {}: \n{}", language, diff)
}

impl CommitCommand {
    pub async fn run(&self, config: &AutocommitConfig, stage_all: bool) -> Result<()> {
        println!("{:?}", config);
        assert_git_repo().await?;

        if stage_all {
            git_add_all().await?;
        }

        let staged_diff = get_staged_diff(&[]).await?;
        let prompt = Message {
            role: MessageRole::User,
            content: get_prompt(config, &staged_diff),
        };

        let mesage: String = generate_message(&[prompt]).await?;
        println!("{}", mesage);
        Ok(())
    }
}
