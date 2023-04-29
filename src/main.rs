use structopt::StructOpt;

mod commands;
mod utils;

use commands::{get_config, Command};
use utils::{outro, Colors};

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
async fn main() {
    let cli = CLI::from_args();

    match cli.command {
        Command::ConfigCommand(config) => match config.run() {
            Ok(_) => (),
            Err(e) => {
                outro(&e.to_string());
            }
        },
        Command::CommitCommand(commit) => {
            let config = match get_config() {
                Ok(c) => c,
                Err(e) => {
                    outro(&e.to_string());
                    return;
                }
            };
            match commit.run(&config, cli.all).await {
                Ok(_) => (),
                Err(e) => {
                    outro(&Colors.red(&e.to_string()));
                }
            }
        }
    }
}
