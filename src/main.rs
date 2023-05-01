use colored::Colorize;
use structopt::StructOpt;

mod commands;
mod git;
mod i18n;
mod utils;

use commands::{get_config, Command};
use utils::{intro, outro};

#[derive(Debug, StructOpt)]
#[structopt(
    name = "autocommit",
    version = "1.0.0",
    about = "A powerful CLI tool that helps you create professional and meaningful commits with ease, using AI to generate impressive commit messages in seconds. Take control of your code history and make it shine with autocommit!"
)]
struct CLI {
    #[structopt(subcommand)]
    command: Command,
}

#[tokio::main]
async fn main() {
    intro("Autocommit");

    let cli = CLI::from_args();

    match cli.command {
        Command::ConfigCommand(config) => match config.run() {
            Ok(_) => (),
            Err(e) => {
                outro(&format!("{} {}", "✖".red(), e));
            }
        },
        Command::CommitCommand(mut commit) => {
            let config = match get_config() {
                Ok(c) => c,
                Err(e) => {
                    let message = &format!("{} {}", "✖".red(), e);
                    let separator_length = 80;
                    let separator = "—".repeat(separator_length).red().bold();
                    outro(&format!(
                        "Commit message:\n{}\n{}\n{}",
                        separator,
                        message.red(),
                        separator
                    ));
                    return;
                }
            };

            match commit.run(&config).await {
                Ok(_) => (),
                Err(e) => {
                    let message = &format!("{} {}", "✖".red(), e);
                    let separator_length = 80;
                    let separator = "—".repeat(separator_length).red().bold();
                    outro(&format!(
                        "Commit message:\n{}\n{}\n{}",
                        separator, message, separator
                    ));
                }
            }
        }
    }
}
