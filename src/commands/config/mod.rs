use anyhow::{anyhow, Context, Result};
use colored::*;
use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::PathBuf;
use std::str::FromStr;
use structopt::StructOpt;

use crate::git::GitRepository;
use crate::i18n::language::Language;
use crate::utils::outro;
use strum::{Display, EnumIter, EnumString, IntoEnumIterator};

#[derive(Debug, PartialEq, Display, EnumIter, EnumString)]
enum ConfigKey {
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

#[derive(Debug, Deserialize, Serialize)]
pub struct AutocommitConfig {
    #[serde(rename = "config")]
    pub config_data: ConfigData,
}

#[derive(Debug, Display, EnumString, Deserialize, Serialize, Clone, PartialEq)]
#[serde(untagged)]
#[serde(rename_all = "lowercase")]
pub enum YesNo {
    Yes,
    No,
}

impl Default for YesNo {
    fn default() -> Self {
        Self::No
    }
}

// This struct represents the configuration data
#[derive(Debug, Deserialize, Serialize)]
pub struct ConfigData {
    pub description_enabled: bool,
    pub emoji_enabled: bool,
    pub language: Language,
    pub name: String,
    pub email: String,
    pub default_commit_message: Option<String>,
    pub default_push_behavior: Option<YesNo>,
    pub default_commit_behavior: Option<YesNo>,
}

const POSSIBLE_VALUES: &[&str; 7] = &[
    "description",
    "emoji",
    "language",
    "name",
    "email",
    "default_commit_message",
    "default_commit_behavior",
];
impl AutocommitConfig {
    fn new() -> anyhow::Result<Self> {
        let name = GitRepository::get_git_name()?;
        let email = GitRepository::get_git_email()?;
        Ok(Self {
            config_data: ConfigData {
                description_enabled: false,
                emoji_enabled: false,
                language: Language::English,
                name,
                email,
                default_commit_message: None,
                default_push_behavior: None,
                default_commit_behavior: None,
            },
        })
    }

    // This function reads the configuration from a file
    fn from_file(path: &PathBuf) -> Result<AutocommitConfig> {
        let mut file = File::open(path)
            .with_context(|| format!("Failed to open config file: {}", path.display()))?;

        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .with_context(|| format!("Failed to read config file: {}", path.display()))?;

        if contents.is_empty() {
            return Err(anyhow!("Config file is empty: {}", path.display()));
        }

        toml::from_str(&contents)
            .with_context(|| format!("Failed to parse config file: {}", path.display()))
    }

    // This function writes the configuration to a file
    fn to_file(&self, path: &PathBuf) -> Result<()> {
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .open(path)
            .with_context(|| format!("Failed to create config file: {}", path.display()))?;

        let contents = toml::to_string(self)
            .with_context(|| format!("Failed to serialize config: {}", path.display()))?;

        file.write_all(contents.as_bytes())
            .with_context(|| format!("Failed to write config file: {}", path.display()))?;

        Ok(())
    }

    fn from_file_or_new(path: &PathBuf) -> Result<AutocommitConfig> {
        match AutocommitConfig::from_file(path) {
            Ok(config) => Ok(config),
            Err(error) => {
                if let Some(io_error) = error
                    .source()
                    .and_then(|e| e.downcast_ref::<std::io::Error>())
                {
                    if io_error.kind() == std::io::ErrorKind::NotFound {
                        let new_config = AutocommitConfig::new()?;
                        new_config.to_file(path)?;
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

    fn update_config(&mut self, key: &ConfigKey, value: &str) -> Result<()> {
        match key {
            ConfigKey::DescriptionEnabled => match value.parse() {
                Ok(value) => self.config_data.description_enabled = value,
                Err(_) => return Err(anyhow!("Invalid value for description")),
            },
            ConfigKey::EmojiEnabled => match value.parse() {
                Ok(value) => self.config_data.emoji_enabled = value,
                Err(_) => return Err(anyhow!("Invalid value for emoji")),
            },
            ConfigKey::Language => match value {
                "english" => self.config_data.language = Language::English,
                _ => return Err(anyhow!("Unsupported language")),
            },
            ConfigKey::Name => self.config_data.name = value.to_owned(),
            ConfigKey::Email => self.config_data.email = value.to_owned(),
            ConfigKey::DefaultCommitMessage => {
                self.config_data.default_commit_message = Some(value.to_owned())
            }
            ConfigKey::DefaultPushBehavior => match value.parse() {
                Ok(value) => self.config_data.default_push_behavior = Some(value),
                Err(_) => return Err(anyhow!("Invalid value for default_push_behavior")),
            },
            ConfigKey::DefaultCommitBehavior => match value.parse() {
                Ok(value) => self.config_data.default_commit_behavior = Some(value),
                Err(_) => return Err(anyhow!("Invalid value for default_commit_behavior")),
            },
        }

        Ok(())
    }

    fn get_config_value(&self, key: &ConfigKey) -> String {
        match key {
            ConfigKey::DescriptionEnabled => self.config_data.description_enabled.to_string(),
            ConfigKey::EmojiEnabled => self.config_data.emoji_enabled.to_string(),
            ConfigKey::Language => format!("{:?}", self.config_data.language).to_lowercase(),
            ConfigKey::Name => format!("{:?}", self.config_data.name).to_lowercase(),
            ConfigKey::Email => format!("{:?}", self.config_data.email).to_lowercase(),
            ConfigKey::DefaultCommitMessage => format!(
                "{:?}",
                self.config_data
                    .default_commit_message
                    .clone()
                    .unwrap_or_default()
            )
            .to_lowercase(),
            ConfigKey::DefaultPushBehavior => format!(
                "{:?}",
                self.config_data
                    .default_push_behavior
                    .clone()
                    .unwrap_or_default()
            )
            .to_lowercase(),
            ConfigKey::DefaultCommitBehavior => format!(
                "{:?}",
                self.config_data
                    .default_push_behavior
                    .clone()
                    .unwrap_or_default()
            )
            .to_lowercase(),
        }
    }

    fn get_config_values(&self, keys: &[ConfigKey]) -> Vec<(String, String)> {
        keys.iter()
            .map(|key| (key.to_string(), self.get_config_value(key)))
            .collect()
    }
}

#[derive(Debug, StructOpt)]
pub enum ConfigCommand {
    #[structopt(name = "get")]
    Get {
        #[structopt(name = "keys", possible_values = POSSIBLE_VALUES)]
        keys: Vec<String>,

        #[structopt(short, long, parse(from_os_str))]
        config_path: Option<PathBuf>,
    },

    #[structopt(name = "set")]
    Set {
        #[structopt(name = "key=value")]
        key_values: Vec<String>,

        #[structopt(short, long, parse(from_os_str))]
        config_path: Option<PathBuf>,
    },
}

impl ConfigCommand {
    fn get_config(&self) -> Result<AutocommitConfig> {
        let config = AutocommitConfig::from_file_or_new(&self.get_config_path())?;
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

                for (key, value) in config_values {
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

                config.to_file(&self.get_config_path())?;
                outro(&format!("{} Config successfully set", "✔".green()));
            }
        }

        Ok(())
    }

    fn get_config_path(&self) -> PathBuf {
        match self {
            ConfigCommand::Get { config_path, .. } => config_path.clone(),
            ConfigCommand::Set { config_path, .. } => config_path.clone(),
        }
        .unwrap_or_else(|| {
            let mut path = dirs::home_dir().unwrap_or_default();
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
