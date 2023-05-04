use std::path::PathBuf;

use anyhow::{anyhow, Context};
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;
use tokio::{
    fs::{File, OpenOptions},
    io::{AsyncReadExt, AsyncWriteExt},
};

use crate::{git::GitRepository, i18n::language::Language};

use super::{
    config_data::ConfigData,
    config_keys::{ConfigItem, ConfigKey, DefaultBehaviorOption, DefaultLanguage, OptionString},
};

#[derive(Debug, Deserialize, Serialize)]
pub struct AutocommitConfig {
    #[serde(rename = "config")]
    pub config_data: ConfigData,
}

impl AutocommitConfig {
    fn new() -> anyhow::Result<Self> {
        let name = GitRepository::get_git_name()?;
        let email = GitRepository::get_git_email()?;
        let config_data = ConfigData {
            description_enabled: ConfigItem::new(false),
            emoji_enabled: ConfigItem::new(false),
            language: ConfigItem::new(DefaultLanguage(Language::English)),
            name: ConfigItem::new(name),
            email: ConfigItem::new(email),
            default_commit_message: ConfigItem::new(OptionString::default()),
            default_push_behavior: ConfigItem::new(DefaultBehaviorOption::default()),
            default_commit_behavior: ConfigItem::new(DefaultBehaviorOption::default()),
            open_ai_api_key: ConfigItem::new(OptionString::default()),
            api_host: ConfigItem::new(String::from("https://api.openai.com")),
            open_ai_model: ConfigItem::new(String::from("gpt-3.5-turbo")),
        };
        Ok(Self { config_data })
    }

    fn update_config_from_env(config: &mut AutocommitConfig) -> anyhow::Result<()> {
        let env_vars = ConfigKey::iter()
            .map(|key| {
                (
                    format!("AUTOCOMMIT_{}", key.to_string().to_uppercase()),
                    key,
                )
            })
            .collect::<Vec<_>>();

        for (var, key) in env_vars.iter() {
            if let Ok(value) = std::env::var(var) {
                config.update_config(key, &value)?;
            }
        }

        Ok(())
    }

    async fn from_file(path: &PathBuf) -> anyhow::Result<AutocommitConfig> {
        let mut file = File::open(path)
            .await
            .with_context(|| format!("Failed to open config file: {}", path.display()))?;

        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .await
            .with_context(|| format!("Failed to read config file: {}", path.display()))?;

        if contents.is_empty() {
            return Err(anyhow!("Config file is empty: {}", path.display()));
        }

        let mut config: AutocommitConfig = toml::from_str(&contents)
            .with_context(|| format!("Failed to parse config file: {}", path.display()))?;

        Self::update_config_from_env(&mut config)?;

        config.config_data.validate()?;

        Ok(config)
    }

    pub async fn to_file(&self, path: &PathBuf) -> anyhow::Result<()> {
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .open(path)
            .await
            .with_context(|| format!("Failed to create config file: {}", path.display()))?;

        let contents = toml::to_string(self)
            .with_context(|| format!("Failed to serialize config: {}", path.display()))?;

        file.write_all(contents.as_bytes())
            .await
            .with_context(|| format!("Failed to write config file: {}", path.display()))?;

        Ok(())
    }

    pub async fn from_file_or_new(path: &PathBuf) -> anyhow::Result<AutocommitConfig> {
        match AutocommitConfig::from_file(path).await {
            Ok(config) => Ok(config),
            Err(error) => {
                if let Some(io_error) = error
                    .source()
                    .and_then(|e| e.downcast_ref::<std::io::Error>())
                {
                    if io_error.kind() == std::io::ErrorKind::NotFound {
                        let new_config = AutocommitConfig::new()?;
                        new_config.to_file(path).await?;
                        Ok(new_config)
                    } else {
                        Err(error)
                            .context(format!("Failed to read config file: {}", path.display()))
                    }
                } else {
                    Err(error).context(format!("Failed to read config file: {}", path.display()))
                }
            }
        }
    }

    pub fn update_config(&mut self, key: &ConfigKey, value: &str) -> anyhow::Result<()> {
        self.config_data.update_config(key, value)?;
        self.config_data.validate()?;
        Ok(())
    }

    pub fn get_config_value(&self, key: &ConfigKey) -> String {
        self.config_data.get_value(key)
    }

    pub fn get_config_values(&self, keys: &[ConfigKey]) -> Vec<(String, String)> {
        keys.iter()
            .map(|key| (key.to_string(), self.get_config_value(key)))
            .collect()
    }
}
