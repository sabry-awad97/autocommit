use serde::{Deserialize, Deserializer, Serialize};

use crate::commands::config::config_keys::{
    ConfigItem, DefaultBehaviorOption, DefaultLanguage, OptionString,
};

use super::config_keys::{ConfigKey, ConfigValue};

#[derive(Debug, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ConfigData {
    #[serde(rename = "description")]
    pub description_enabled: ConfigItem<bool>,
    #[serde(rename = "emoji")]
    pub emoji_enabled: ConfigItem<bool>,
    pub language: ConfigItem<DefaultLanguage>,
    pub name: ConfigItem<String>,
    pub email: ConfigItem<String>,
    pub default_commit_message: ConfigItem<OptionString>,
    pub default_push_behavior: ConfigItem<DefaultBehaviorOption>,
    pub default_commit_behavior: ConfigItem<DefaultBehaviorOption>,
}

impl<'de> Deserialize<'de> for ConfigData {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Debug, Deserialize)]
        struct InnerConfigData {
            #[serde(rename = "description")]
            description_enabled: ConfigItem<bool>,
            #[serde(rename = "emoji")]
            emoji_enabled: ConfigItem<bool>,
            language: ConfigItem<DefaultLanguage>,
            name: ConfigItem<String>,
            email: ConfigItem<String>,
            default_commit_message: ConfigItem<OptionString>,
            default_push_behavior: ConfigItem<DefaultBehaviorOption>,
            default_commit_behavior: ConfigItem<DefaultBehaviorOption>,
        }

        let inner = InnerConfigData::deserialize(deserializer)?;
        Ok(Self {
            description_enabled: inner.description_enabled,
            emoji_enabled: inner.emoji_enabled,
            language: inner.language,
            name: inner.name,
            email: inner.email,
            default_commit_message: inner.default_commit_message,
            default_push_behavior: inner.default_push_behavior,
            default_commit_behavior: inner.default_commit_behavior,
        })
    }
}

impl ConfigData {
    pub fn validate(&self) -> anyhow::Result<()> {
        self.description_enabled.value.validate()?;
        self.emoji_enabled.value.validate()?;
        self.language.value.validate()?;
        self.name.value.validate()?;
        self.email.value.validate()?;
        self.default_commit_message.value.validate()?;
        self.default_push_behavior.value.validate()?;
        self.default_commit_behavior.value.validate()?;
        Ok(())
    }

    pub fn update_config(&mut self, key: &ConfigKey, value: &str) -> anyhow::Result<()> {
        match key {
            ConfigKey::DescriptionEnabled => self.description_enabled.update(value)?,
            ConfigKey::EmojiEnabled => self.emoji_enabled.update(value)?,
            ConfigKey::Language => self.language.update(value)?,
            ConfigKey::Name => self.name.update(value)?,
            ConfigKey::Email => self.email.update(value)?,
            ConfigKey::DefaultCommitMessage => self.default_commit_message.update(value)?,
            ConfigKey::DefaultPushBehavior => self.default_push_behavior.update(value)?,
            ConfigKey::DefaultCommitBehavior => self.default_commit_behavior.update(value)?,
        }
        Ok(())
    }

    pub fn get_value(&self, key: &ConfigKey) -> String {
        match key {
            ConfigKey::DescriptionEnabled => self.description_enabled.get_value(),
            ConfigKey::EmojiEnabled => self.emoji_enabled.get_value(),
            ConfigKey::Language => self.language.get_value(),
            ConfigKey::Name => self.name.get_value(),
            ConfigKey::Email => self.email.get_value(),
            ConfigKey::DefaultCommitMessage => self.default_commit_message.get_value(),
            ConfigKey::DefaultPushBehavior => self.default_push_behavior.get_value(),
            ConfigKey::DefaultCommitBehavior => self.default_push_behavior.get_value(),
        }
    }
}
