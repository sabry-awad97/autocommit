mod chroma;
mod is_unicode_supported;
mod openai;
mod prompts;
mod spinner;

pub use chroma::get_colors;
pub use is_unicode_supported::get_unicode_string;
pub use openai::{generate_message, Message, MessageRole};
pub use prompts::{intro, outro};
pub use spinner::spinner;
