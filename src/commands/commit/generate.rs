use colored::{Color, Colorize};

use crate::commands::config::AutocommitConfig;
use crate::utils::{outro, spinner, MessageRole};

use super::chat_context::ChatContext;

const GENERATING_MESSAGE: &str = "Generating the commit message";

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

    let separator_length = 40;
    let separator = "—"
        .repeat(separator_length)
        .color(Color::TrueColor {
            r: 128,
            g: 128,
            b: 128,
        })
        .bold();

    outro(&format!(
        "Commit message:\n{}\n{}\n{}",
        separator, commit_message, separator
    ));
    Ok(commit_message)
}

