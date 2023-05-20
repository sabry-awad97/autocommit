use anyhow::Error;
use colored::Colorize;
use structopt::StructOpt;

mod commands;
mod git;
mod i18n;
mod utils;

use commands::{get_service, Command};
use log::info;
use term_size::dimensions;
use utils::{intro, outro};
#[derive(Debug, StructOpt)]
#[structopt(
    name = "autocommit",
    version = "1.0.0",
    about = "A powerful CLI tool that helps you create professional and meaningful commits with ease, using AI to generate impressive commit messages in seconds. Take control of your code history and make it shine with autocommit!"
)]
struct Cli {
    #[structopt(subcommand)]
    command: Command,
}

#[tokio::main]
async fn main() {
    println!("{esc}c", esc = 27 as char);
    env_logger::init();
    intro("Autocommit");

    let cli = Cli::from_args();

    match cli.command {
        Command::ConfigCommand(config) => match config.run().await {
            Ok(_) => (),
            Err(e) => {
                handle_error(e);
            }
        },
        Command::CommitCommand(mut commit) => {
            let service = match get_service().await {
                Ok(s) => s,
                Err(e) => {
                    handle_error(e);
                    return;
                }
            };

            match commit.run(service.get_config()).await {
                Ok(_) => (),
                Err(e) => {
                    handle_error(e);
                }
            }
        }
    }

    info!("Autocommit finished successfully");
}

fn handle_error(e: Error) {
    let message = &format!("{} {}", "✖".red(), e);
    let lines: Vec<&str> = message.split('\n').collect();
    let longest_line = lines.iter().map(|line| line.len()).max().unwrap_or(0);
    let term_width = dimensions().unwrap_or((80, 24)).0;
    let separator_length = longest_line.min(term_width);
    let separator = "—".repeat(separator_length).red().bold();
    outro(&format!("\n{}\n{}\n{}", separator, message, separator));
}
