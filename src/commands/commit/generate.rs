use std::io;
use std::io::Write;
use std::time::Duration;

use colored::{Color, Colorize};

use crate::commands::config::AutocommitConfig;
use crate::git::GitRepository;
use crate::utils::{outro, spinner, MessageRole};

use super::chat_context::ChatContext;

const GENERATING_MESSAGE: &str = "Generating the commit message";
const COMMITTING_CHANGES: &str = "Committing changes...";

pub async fn generate_autocommit_message(
    config: &AutocommitConfig,
    content: &str,
) -> anyhow::Result<String> {
    let mut commit_spinner = spinner();
    commit_spinner.start(GENERATING_MESSAGE);

    let mut chat_context = ChatContext::get_initial_context(config);
    chat_context.add_message(MessageRole::User, content.to_owned());

    let commit_message = chat_context.generate_message().await?;
    commit_spinner.stop("📝 Commit message generated successfully");

    let separator = "—".repeat(80).color(Color::TrueColor {
        r: 128,
        g: 128,
        b: 128,
    });

    outro(&format!("Commit message:\n{}", separator));
    print_with_delay(&commit_message).await;
    println!("{}", separator);
    Ok(commit_message)
}

pub async fn commit_changes(commit_message: &str) -> anyhow::Result<()> {
    let mut commit_spinner = spinner();
    commit_spinner.start(COMMITTING_CHANGES);
    GitRepository::git_commit(&commit_message).await?;
    commit_spinner.stop(format!("{} Changes committed successfully", "✔".green()));
    Ok(())
}

async fn print_with_delay(code: &str) {
    // Delay between printing each character
    let delay = Duration::from_millis(50);
    for c in code.chars() {
        print!("{}", c);
        io::stdout().flush().unwrap();
        tokio::time::sleep(delay).await;
    }
}
