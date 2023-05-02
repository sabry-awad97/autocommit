use anyhow::{anyhow, Context, Result};
use colored::*;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::PathBuf;
use std::str::FromStr;
use std::{env, fmt};
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

pub trait ConfigValue: FromStr + ToString {
    fn validate(&self) -> anyhow::Result<()>;
    fn update(&mut self, value: &str) -> Result<()>;
    fn get_value(&self) -> String;
}

impl ConfigValue for bool {
    fn validate(&self) -> anyhow::Result<()> {
        Ok(())
    }

    fn update(&mut self, value: &str) -> Result<()> {
        match value.parse() {
            Ok(value) => *self = value,
            Err(_) => return Err(anyhow!("Invalid value for boolean")),
        }

        Ok(())
    }

    fn get_value(&self) -> String {
        self.to_string()
    }
}

#[derive(Debug, Serialize)]

pub struct DefaultLanguage(pub Language);

impl FromStr for DefaultLanguage {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "english" => Ok(Self(Language::English)),
            _ => Ok(Self(Language::English)),
        }
    }
}

impl fmt::Display for DefaultLanguage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0 {
            Language::English => write!(f, "english"),
        }
    }
}

impl<'de> serde::Deserialize<'de> for DefaultLanguage {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.to_lowercase().as_str() {
            "english" => Ok(Self(Language::English)),
            _ => Ok(Self(Language::English)),
        }
    }
}

impl ConfigValue for DefaultLanguage {
    fn validate(&self) -> anyhow::Result<()> {
        Ok(())
    }

    fn update(&mut self, value: &str) -> Result<()> {
        match value {
            "english" => self.0 = Language::English,
            _ => return Err(anyhow!("Unsupported language")),
        }

        Ok(())
    }

    fn get_value(&self) -> String {
        format!("{:?}", self.0).to_lowercase()
    }
}

impl ConfigValue for String {
    fn validate(&self) -> anyhow::Result<()> {
        Ok(())
    }

    fn update(&mut self, value: &str) -> Result<()> {
        *self = value.to_owned();
        Ok(())
    }

    fn get_value(&self) -> String {
        self.to_owned()
    }
}

#[derive(Debug, Serialize)]

pub struct OptionString(Option<String>);

impl OptionString {
    pub fn get_inner_value(&self) -> Option<String> {
        self.0.clone()
    }
}

impl FromStr for OptionString {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            Ok(Self(None))
        } else {
            Ok(Self(Some(s.to_owned())))
        }
    }
}

impl fmt::Display for OptionString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0 {
            Some(s) => write!(f, "{}", s),
            None => write!(f, ""),
        }
    }
}

impl<'de> serde::Deserialize<'de> for OptionString {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = Option::<String>::deserialize(deserializer)?;
        Ok(OptionString(s))
    }
}

impl ConfigValue for OptionString {
    fn validate(&self) -> anyhow::Result<()> {
        Ok(())
    }

    fn update(&mut self, value: &str) -> Result<()> {
        match value.parse() {
            Ok(value) => self.0 = Some(value),
            Err(_) => return Err(anyhow!("Invalid value for option string")),
        }

        Ok(())
    }

    fn get_value(&self) -> String {
        self.0.as_ref().map(|s| s.to_owned()).unwrap_or_default()
    }
}

#[derive(Debug, Serialize)]

pub struct DefaultBehaviorOption(Option<DefaultBehavior>);

impl DefaultBehaviorOption {
    pub fn get_inner_value(&self) -> Option<DefaultBehavior> {
        self.0.clone()
    }
}

impl FromStr for DefaultBehaviorOption {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "yes" => Ok(Self(Some(DefaultBehavior::Yes))),
            "no" => Ok(Self(Some(DefaultBehavior::No))),
            "ask" => Ok(Self(Some(DefaultBehavior::Ask))),
            _ => Ok(Self(None)),
        }
    }
}

impl fmt::Display for DefaultBehaviorOption {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0 {
            Some(DefaultBehavior::Yes) => write!(f, "yes"),
            Some(DefaultBehavior::No) => write!(f, "no"),
            Some(DefaultBehavior::Ask) => write!(f, "ask"),
            None => write!(f, ""),
        }
    }
}

impl<'de> serde::Deserialize<'de> for DefaultBehaviorOption {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = Option::<DefaultBehavior>::deserialize(deserializer)?;
        Ok(DefaultBehaviorOption(s))
    }
}

impl ConfigValue for DefaultBehaviorOption {
    fn validate(&self) -> anyhow::Result<()> {
        Ok(())
    }

    fn update(&mut self, value: &str) -> Result<()> {
        match value.parse() {
            Ok(value) => self.0 = Some(value),
            Err(_) => return Err(anyhow!("Invalid value for default behavior")),
        }

        Ok(())
    }

    fn get_value(&self) -> String {
        format!("{:?}", self.0.as_ref().unwrap_or(&Default::default())).to_lowercase()
    }
}

#[derive(Debug)]
pub struct ConfigItem<T>
where
    T: ConfigValue,
{
    value: T,
}

impl<T> Serialize for ConfigItem<T>
where
    T: Serialize + ConfigValue,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.value.serialize(serializer)
    }
}

impl<'de, T> Deserialize<'de> for ConfigItem<T>
where
    T: Deserialize<'de> + ConfigValue,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        T::deserialize(deserializer).map(Self::new)
    }
}

impl<T> ConfigItem<T>
where
    T: ConfigValue,
{
    fn new(value: T) -> Self {
        Self { value }
    }

    fn update(&mut self, value: &str) -> Result<()> {
        self.value.update(value)?;
        self.value.validate()?;
        Ok(())
    }

    fn get_value(&self) -> String {
        self.value.get_value()
    }

    pub fn get_value_ref(&self) -> &T {
        &self.value
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AutocommitConfig {
    #[serde(rename = "config")]
    pub config_data: ConfigData,
}

#[derive(Debug, Display, EnumString, Clone, PartialEq)]
pub enum DefaultBehavior {
    #[strum(serialize = "yes")]
    Yes,
    #[strum(serialize = "no")]
    No,
    #[strum(serialize = "ask")]
    Ask,
}

impl Default for DefaultBehavior {
    fn default() -> Self {
        Self::Ask
    }
}

impl<'de> Deserialize<'de> for DefaultBehavior {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "yes" => Ok(DefaultBehavior::Yes),
            "no" => Ok(DefaultBehavior::No),
            "ask" => Ok(DefaultBehavior::Ask),
            _ => Err(serde::de::Error::custom(format!("invalid value: {}", s))),
        }
    }
}

impl Serialize for DefaultBehavior {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            DefaultBehavior::Yes => serializer.serialize_str("yes"),
            DefaultBehavior::No => serializer.serialize_str("no"),
            DefaultBehavior::Ask => serializer.serialize_str("ask"),
        }
    }
}

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
            default_commit_message: ConfigItem::new(OptionString(None)),
            default_push_behavior: ConfigItem::new(DefaultBehaviorOption(Some(
                DefaultBehavior::default(),
            ))),
            default_commit_behavior: ConfigItem::new(DefaultBehaviorOption(Some(
                DefaultBehavior::Ask,
            ))),
        };
        Ok(Self { config_data })
    }

    fn update_config_from_env(config: &mut AutocommitConfig) -> Result<()> {
        let env_vars = ConfigKey::iter()
            .map(|key| {
                (
                    format!("AUTOCOMMIT_{}", key.to_string().to_uppercase()),
                    key,
                )
            })
            .collect::<Vec<_>>();

        for (var, key) in env_vars.iter() {
            if let Ok(value) = env::var(var) {
                config.update_config(key, &value)?;
            }
        }

        Ok(())
    }

    fn from_file(path: &PathBuf) -> Result<AutocommitConfig> {
        let mut file = File::open(path)
            .with_context(|| format!("Failed to open config file: {}", path.display()))?;

        let mut contents = String::new();
        file.read_to_string(&mut contents)
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
        self.config_data.update_config(key, value)?;
        self.config_data.validate()?;
        Ok(())
    }

    fn get_config_value(&self, key: &ConfigKey) -> String {
        self.config_data.get_value(key)
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
