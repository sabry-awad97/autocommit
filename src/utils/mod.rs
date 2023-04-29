mod chroma;
mod git;
mod is_unicode_supported;
mod openai;
mod prompts;
mod spinner;

pub use chroma::Colors;
pub use git::{assert_git_repo, get_changed_files, get_staged_diff, get_staged_files, git_add};
pub use is_unicode_supported::get_unicode_string;
pub use openai::{generate_message, Message, MessageRole};
pub use prompts::{intro, outro};
pub use spinner::spinner;
