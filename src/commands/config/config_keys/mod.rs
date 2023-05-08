use strum::{Display, EnumIter, EnumString};

mod config_item;
mod default_language;
mod option_string;

pub use config_item::ConfigItem;
pub use config_item::ConfigValue;
pub use default_language::DefaultLanguage;
pub use option_string::OptionString;

#[derive(Debug, PartialEq, Display, EnumIter, EnumString)]
pub enum ConfigKey {
    #[strum(serialize = "open_ai_api_key")]
    OpenAiApiKey,
    #[strum(serialize = "open_ai_model")]
    OpenAiModel,
    #[strum(serialize = "api_host")]
    ApiHost,
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
}
