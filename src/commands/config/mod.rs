use anyhow::{anyhow, Context, Result};
use colored::*;
use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::PathBuf;
use structopt::StructOpt;

use crate::git::GitRepository;
use crate::i18n::language::Language;
use crate::utils::outro;

// This enum represents the configuration keys
#[derive(Debug, PartialEq)]
enum ConfigKey {
    DescriptionEnabled,
    EmojiEnabled,
    Language,
    Name,
    Email,
}

impl ConfigKey {
    fn from_str(key: &str) -> Option<ConfigKey> {
        match key {
            "description" => Some(ConfigKey::DescriptionEnabled),
            "emoji" => Some(ConfigKey::EmojiEnabled),
            "language" => Some(ConfigKey::Language),
            "name" => Some(ConfigKey::Name),
            "email" => Some(ConfigKey::Email),
            _ => None,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AutocommitConfig {
    #[serde(rename = "config")]
    pub config_data: ConfigData,
}

// This struct represents the configuration data
#[derive(Debug, Deserialize, Serialize)]
pub struct ConfigData {
    pub description_enabled: bool,
    pub emoji_enabled: bool,
    pub language: Language,
    pub name: String,
    pub email: String,
}

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
}

#[derive(Debug, StructOpt)]
pub enum ConfigCommand {
    #[structopt(name = "get")]
    Get {
        #[structopt(name = "keys", possible_values = &["description", "emoji", "language"])]
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
                let config_data = config.config_data;

                let description = config_data.description_enabled.to_string();
                let emoji = config_data.emoji_enabled.to_string();
                let language = format!("{:?}", config_data.language).to_lowercase();

                if keys.is_empty() {
                    println!("{} = {}", "description".bold(), description.green());
                    println!("{} = {}", "emoji".bold(), emoji.green());
                    println!("{} = {}", "language".bold(), language.green());
                } else {
                    for key in keys {
                        match ConfigKey::from_str(key) {
                            Some(ConfigKey::DescriptionEnabled) => {
                                println!("{} = {}", key.bold(), description.green());
                            }
                            Some(ConfigKey::EmojiEnabled) => {
                                println!("{} = {}", key.bold(), emoji.green());
                            }
                            Some(ConfigKey::Language) => {
                                println!("{} = {}", key.bold(), language.green());
                            }
                            _ => {
                                return Err(anyhow!("Unsupported config key: {}", key));
                            }
                        }
                    }
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

                    match ConfigKey::from_str(key) {
                        Some(ConfigKey::DescriptionEnabled) => match value.parse() {
                            Ok(value) => config.config_data.description_enabled = value,
                            Err(_) => return Err(anyhow!("Invalid value for description")),
                        },
                        Some(ConfigKey::EmojiEnabled) => match value.parse() {
                            Ok(value) => config.config_data.emoji_enabled = value,
                            Err(_) => return Err(anyhow!("Invalid value for emoji")),
                        },
                        Some(ConfigKey::Language) => match value {
                            "english" => config.config_data.language = Language::English,
                            _ => return Err(anyhow!("Unsupported language")),
                        },
                        _ => {
                            return Err(anyhow!("Unsupported config key: {}", key));
                        }
                    }
                }

                if let Err(error) = config.to_file(&self.get_config_path()) {
                    return Err(error);
                }

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
