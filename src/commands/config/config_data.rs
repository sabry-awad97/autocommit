use serde::{Deserialize, Deserializer, Serialize};

use crate::commands::config::config_keys::{ConfigItem, DefaultLanguage, OptionString};

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
    pub open_ai_api_key: ConfigItem<OptionString>,
    pub api_host: ConfigItem<String>,
    pub open_ai_model: ConfigItem<OptionString>,
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
            open_ai_api_key: ConfigItem<OptionString>,
            api_host: ConfigItem<String>,
            open_ai_model: ConfigItem<OptionString>,
        }

        let inner = InnerConfigData::deserialize(deserializer)?;
        Ok(Self {
            description_enabled: inner.description_enabled,
            emoji_enabled: inner.emoji_enabled,
            language: inner.language,
            name: inner.name,
            email: inner.email,
            open_ai_api_key: inner.open_ai_api_key,
            api_host: inner.api_host,
            open_ai_model: inner.open_ai_model,
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
        Ok(())
    }

    pub fn update_config(&mut self, key: &ConfigKey, value: &str) -> anyhow::Result<()> {
        match key {
            ConfigKey::DescriptionEnabled => self.description_enabled.update(value)?,
            ConfigKey::EmojiEnabled => self.emoji_enabled.update(value)?,
            ConfigKey::Language => self.language.update(value)?,
            ConfigKey::Name => self.name.update(value)?,
            ConfigKey::Email => self.email.update(value)?,
            ConfigKey::OpenAiApiKey => self.open_ai_api_key.update(value)?,
            ConfigKey::ApiHost => self.api_host.update(value)?,
            ConfigKey::OpenAiModel => self.open_ai_model.update(value)?,
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
            ConfigKey::OpenAiApiKey => self.open_ai_api_key.get_value(),
            ConfigKey::ApiHost => self.api_host.get_value(),
            ConfigKey::OpenAiModel => self.open_ai_model.get_value(),
        }
    }
}
