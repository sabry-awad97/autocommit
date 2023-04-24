use anyhow::Result;
use structopt::StructOpt;

use super::config::AutocommitConfig;

#[derive(Debug, StructOpt)]
pub enum CommitCommand {}

impl CommitCommand {
    pub fn run(&self, config: &AutocommitConfig) -> Result<()> {
        todo!()
    }
}
