use std::error::Error;

use structopt::StructOpt;

mod commands;
mod utils;

use commands::{get_config, Command};

#[derive(Debug, StructOpt)]
#[structopt(
    name = "autocommit",
    version = "1.0.0",
    about = "A powerful CLI tool that helps you create professional and meaningful commits with ease, using AI to generate impressive commit messages in seconds. Take control of your code history and make it shine with autocommit!",
    alias = "ac"
)]
struct CLI {
    #[structopt(subcommand)]
    command: Command,

    #[structopt(short, long)]
    all: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let cli = CLI::from_args();

    match cli.command {
        Command::ConfigCommand(config) => {
            config.run()?;
        }
        Command::CommitCommand(commit) => {
            let config = get_config()?;
            commit.run(&config, cli.all).await?;
        }
    }

    Ok(())
}
