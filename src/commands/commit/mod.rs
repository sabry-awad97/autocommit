use anyhow::Result;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub enum CommitCommand {}

impl CommitCommand {
    pub fn run(&self) -> Result<()> {
        todo!()
    }
}
