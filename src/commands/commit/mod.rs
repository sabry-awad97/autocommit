use anyhow::Result;
use structopt::StructOpt;

use super::config::AutocommitConfig;

#[derive(Debug, StructOpt)]
pub struct CommitCommand {}

impl CommitCommand {
    pub fn run(&self, config: &AutocommitConfig) -> Result<()> {
        println!("{:?}", config);
        Ok(())
    }
}
