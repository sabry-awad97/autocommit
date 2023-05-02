use anyhow::{anyhow, Result};
use colored::*;
use serde::{Deserialize, Deserializer, Serialize};
use std::path::PathBuf;
use std::str::FromStr;
use structopt::StructOpt;

use crate::utils::outro;
use strum::IntoEnumIterator;

pub use autocommit_config::AutocommitConfig;

use self::config_keys::{
    ConfigItem, ConfigKey, ConfigValue, DefaultBehaviorOption, DefaultLanguage, OptionString,
};

mod autocommit_config;
mod config_data;
mod config_keys;

#[derive(Debug, Serialize)]
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
    fn validate(&self) -> Result<()> {
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

    fn update_config(&mut self, key: &ConfigKey, value: &str) -> Result<()> {
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

    fn get_value(&self, key: &ConfigKey) -> String {
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

#[derive(Debug, StructOpt)]
pub enum ConfigCommand {
    #[structopt(name = "get")]
    Get {
        #[structopt(name = "keys", short, long)]
        keys: Vec<String>,

        #[structopt(short, long, parse(from_os_str))]
        config_path: Option<PathBuf>,
    },

    #[structopt(name = "set")]
    Set {
        #[structopt(name = "key=value", required = true, min_values = 1)]
        key_values: Vec<String>,

        #[structopt(short, long, parse(from_os_str))]
        config_path: Option<PathBuf>,
    },
    #[structopt(name = "reset")]
    Reset,
    #[structopt(name = "env")]
    Env {
        #[structopt(short, long)]
        shell: Option<String>,
    },
}

impl ConfigCommand {
    fn get_config(&self) -> Result<AutocommitConfig> {
        let config = AutocommitConfig::from_file_or_new(&self.get_config_path()?)?;
        Ok(config)
    }

    pub fn run(&self) -> Result<()> {
        let mut config = self.get_config()?;
        match self {
            ConfigCommand::Get { keys, .. } => {
                let config_values = if keys.is_empty() {
                    let all_keys = ConfigKey::iter().collect::<Vec<_>>();
                    config.get_config_values(&all_keys)
                } else {
                    keys.iter()
                        .map(|key| {
                            ConfigKey::from_str(key).map(|config_key| {
                                (key.to_string(), config.get_config_value(&config_key))
                            })
                        })
                        .filter_map(Result::ok)
                        .collect()
                };
                let filtered_config_values = config_values
                    .into_iter()
                    .filter(|(_, value)| !value.is_empty())
                    .collect::<Vec<_>>();
                for (key, value) in filtered_config_values {
                    println!("{} = {}", key.bold(), value.green());
                }
            }
            ConfigCommand::Set { key_values, .. } => {
                for key_value in key_values {
                    let parts: Vec<&str> = key_value.splitn(2, '=').collect();
                    if parts.len() != 2 {
                        return Err(anyhow!("Invalid argument format"));
                    }

                    let key = parts[0].trim();
                    let value = parts[1].trim();

                    let config_key = ConfigKey::from_str(key)
                        .map_err(|_| anyhow!("Unsupported config key: {}", key))?;

                    config.update_config(&config_key, value)?;
                }

                config.to_file(&self.get_config_path()?)?;
                outro(&format!("{} Config successfully set", "✔".green()));
            }
            ConfigCommand::Reset => {
                let config = AutocommitConfig::new()?;
                config.to_file(&self.get_config_path()?)?;
                outro(&format!("{} Config successfully reset", "✔".green()));
            }
            ConfigCommand::Env { shell } => {
                let config = self.get_config()?;
                let config_values =
                    config.get_config_values(&ConfigKey::iter().collect::<Vec<_>>());

                match shell.as_deref() {
                    Some("bash") => {
                        for (key, value) in config_values {
                            println!("export AUTOCOMMIT_{}={}", key.to_uppercase(), value);
                        }
                    }
                    Some("fish") => {
                        for (key, value) in config_values {
                            println!("set -gx AUTOCOMMIT_{} {}", key.to_uppercase(), value);
                        }
                    }
                    Some("powershell") => {
                        for (key, value) in config_values {
                            println!("$env:AUTOCOMMIT_{} = '{}'", key.to_uppercase(), value);
                        }
                    }
                    _ => {
                        for (key, value) in config_values {
                            println!("{}={}", key.to_uppercase(), value);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    fn get_config_path(&self) -> anyhow::Result<PathBuf> {
        let config_path = match self {
            ConfigCommand::Get { config_path, .. } => config_path.clone(),
            ConfigCommand::Set { config_path, .. } => config_path.clone(),
            _ => None,
        };
        config_path.map(Ok).unwrap_or_else(|| {
            Self::default_config_path()
                .ok_or_else(|| anyhow!("Could not determine default config path"))
        })
    }

    fn default_config_path() -> Option<PathBuf> {
        dirs::home_dir().map(|mut path| {
            path.push(".autocommit");
            path
        })
    }
}

pub fn get_config() -> Result<AutocommitConfig> {
    let config_command = ConfigCommand::Get {
        keys: vec![],
        config_path: None,
    };
    Ok(config_command.get_config()?)
}
