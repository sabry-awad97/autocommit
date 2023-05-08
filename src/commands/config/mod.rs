use anyhow::{anyhow, Result};
use colored::*;
use log::{debug, info};
use std::{path::PathBuf, str::FromStr};
use structopt::StructOpt;
use strum::IntoEnumIterator;

use crate::utils::outro;

pub use autocommit_config::AutocommitConfig;

use self::{config_keys::ConfigKey, config_service::AutocommitService};

mod autocommit_config;
mod config_data;
mod config_keys;
mod config_service;

#[derive(Debug, StructOpt)]
pub enum ConfigCommand {
    #[structopt(name = "get")]
    Get {
        #[structopt(name = "keys", short, long, help = "Configuration keys to retrieve")]
        keys: Vec<String>,

        #[structopt(
            short,
            long,
            parse(from_os_str),
            help = "Path to the configuration file"
        )]
        config_path: Option<PathBuf>,
    },

    #[structopt(name = "set")]
    Set {
        #[structopt(
            name = "key=value",
            required = true,
            min_values = 1,
            help = "Configuration keys and their new values to set"
        )]
        key_values: Vec<String>,

        #[structopt(
            short,
            long,
            parse(from_os_str),
            help = "Path to the configuration file"
        )]
        config_path: Option<PathBuf>,
    },
    #[structopt(name = "reset")]
    Reset,
    #[structopt(name = "env")]
    Env {
        #[structopt(
            short,
            long,
            help = "The shell for which to print environment variables. Options are: bash, fish, powershell"
        )]
        shell: Option<String>,
    },
}

impl ConfigCommand {
    async fn get_service(&self) -> anyhow::Result<AutocommitService> {
        let config_path = self.get_config_path()?;
        let service = AutocommitService::new(&config_path).await?;
        Ok(service)
    }

    pub async fn run(&self) -> Result<()> {
        let mut service = self.get_service().await?;
        match self {
            ConfigCommand::Get { keys, .. } => {
                let config_values = if keys.is_empty() {
                    let all_keys = ConfigKey::iter().collect::<Vec<_>>();
                    service.get_config_values(&all_keys)
                } else {
                    keys.iter()
                        .map(|key| {
                            ConfigKey::from_str(key).map(|config_key| {
                                (key.to_string(), service.get_config_value(&config_key))
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

                    service.update_config(&config_key, value)?;
                }

                let config_path = self.get_config_path()?;
                debug!("Saving config to {:?}", config_path);
                service.save_config_to(&config_path).await?;
                outro(&format!("{} Config successfully set", "✔".green()));
            }
            ConfigCommand::Reset => {
                let config_path = self.get_config_path()?;
                let service = AutocommitService::new(&config_path).await?;
                debug!("Saving config to {:?}", config_path);
                service.save_config_to(&config_path).await?;
                outro(&format!("{} Config successfully reset", "✔".green()));
            }
            ConfigCommand::Env { shell } => {
                let config = self.get_service().await?;
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

pub async fn get_service() -> Result<AutocommitService> {
    let config_command = ConfigCommand::Get {
        keys: vec![],
        config_path: None,
    };
    info!("Getting config");
    config_command.get_service().await
}
