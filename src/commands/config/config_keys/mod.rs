use strum::{Display, EnumIter, EnumString};

mod config_item;
mod default_behavior;
mod default_language;
mod option_string;

pub use config_item::ConfigItem;
pub use config_item::ConfigValue;
pub use default_behavior::DefaultBehaviorOption;
pub use option_string::OptionString;
pub use default_language::DefaultLanguage;

#[derive(Debug, PartialEq, Display, EnumIter, EnumString)]
pub enum ConfigKey {
    #[strum(serialize = "description")]
    DescriptionEnabled,
    #[strum(serialize = "emoji")]
    EmojiEnabled,
    #[strum(serialize = "language")]
    Language,
    #[strum(serialize = "name")]
    Name,
    #[strum(serialize = "email")]
    Email,
    #[strum(serialize = "default_commit_message")]
    DefaultCommitMessage,
    #[strum(serialize = "default_push_behavior")]
    DefaultPushBehavior,
    #[strum(serialize = "default_commit_behavior")]
    DefaultCommitBehavior,
}
