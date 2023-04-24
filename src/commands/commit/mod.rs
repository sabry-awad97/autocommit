use anyhow::Result;
use structopt::StructOpt;

use crate::utils::{generate_message, Message, MessageRole};

use super::config::AutocommitConfig;

#[derive(Debug, StructOpt)]
pub struct CommitCommand {}

impl CommitCommand {
    pub async fn run(&self, config: &AutocommitConfig) -> Result<()> {
        println!("{:?}", config);
        let prompt = Message {
            role: MessageRole::User,
            content: String::from("Say this is a test"),
        };

        let massage = generate_message(&[prompt]).await?;
        println!("{:?}", massage);
        Ok(())
    }
}
