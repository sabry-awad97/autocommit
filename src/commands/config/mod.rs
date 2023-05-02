use anyhow::{anyhow, Result};
use colored::*;
use log::{debug, info};
use std::{path::PathBuf, str::FromStr};
use structopt::StructOpt;
use strum::IntoEnumIterator;

use crate::utils::outro;

pub use autocommit_config::AutocommitConfig;

use self::config_keys::ConfigKey;

mod autocommit_config;
mod config_data;
mod config_keys;

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
        let config_path = self.get_config_path()?;
        debug!("Loading config from {:?}", config_path);
        let config = AutocommitConfig::from_file_or_new(&config_path)?;
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

                let config_path = self.get_config_path()?;
                debug!("Saving config to {:?}", config_path);
                config.to_file(&config_path)?;
                outro(&format!("{} Config successfully set", "✔".green()));
            }
            ConfigCommand::Reset => {
                let config = AutocommitConfig::new()?;
                let config_path = self.get_config_path()?;
                debug!("Saving config to {:?}", config_path);
                config.to_file(&config_path)?;
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
            ConfigCommand::Reset => None,
            ConfigCommand::Env { .. } => None,
        };
        let default_config_path = Self::default_config_path();
        let config_path = config_path.or(default_config_path);
        config_path.ok_or_else(|| anyhow!("Could not determine config path"))
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
    info!("Getting config");
    Ok(config_command.get_config()?)
}
