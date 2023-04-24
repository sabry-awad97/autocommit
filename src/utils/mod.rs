mod git;
mod openai;
pub use git::{assert_git_repo, get_staged_diff, git_add_all};
pub use openai::{generate_message, Message, MessageRole};
