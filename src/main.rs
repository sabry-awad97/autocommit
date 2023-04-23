use std::error::Error;

use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "commit-master",
    version = "1.0.0",
    about = "A powerful CLI tool that helps you create professional and meaningful commits with ease, using AI to generate impressive commit messages in seconds. Take control of your code history and make it shine with commit-master!",
    alias = "cm"
)]
struct Cli {}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    Ok(())
}
