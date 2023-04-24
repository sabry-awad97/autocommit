use structopt::StructOpt;

mod commit;
mod config;

pub use config::get_config;

#[derive(Debug, StructOpt)]
pub enum Command {
    #[structopt(name = "config")]
    ConfigCommand(config::ConfigCommand),
    #[structopt(name = "commit")]
    CommitCommand(commit::CommitCommand),
}
