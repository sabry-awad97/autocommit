use std::error::Error;

use structopt::StructOpt;

mod commands;

use commands::config::{get_config, ConfigCommand};

#[derive(Debug, StructOpt)]
#[structopt(
    name = "autocommit",
    version = "1.0.0",
    about = "A powerful CLI tool that helps you create professional and meaningful commits with ease, using AI to generate impressive commit messages in seconds. Take control of your code history and make it shine with autocommit!",
    alias = "cm"
)]
enum CLI {
    #[structopt(name = "config")]
    ConfigCommand(ConfigCommand),
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    match CLI::from_args() {
        CLI::ConfigCommand(config) => {
            config.run()?;
        }
    }

    let config = get_config()?;
    println!("{:?}", config);

    Ok(())
}
