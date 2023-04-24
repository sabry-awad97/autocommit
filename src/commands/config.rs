use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
enum Language {
    English,
    // Add more languages as needed
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    #[serde(rename = "config")]
    config_data: ConfigData,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ConfigData {
    description: bool,
    emoji: bool,
    language: Language,
}

impl Config {
    fn new() -> Config {
        Config {
            config_data: ConfigData {
                description: false,
                emoji: false,
                language: Language::English,
            },
        }
    }

    fn from_file(path: &PathBuf) -> Result<Config, String> {
        let mut file = match File::open(path) {
            Ok(file) => file,
            Err(_) => return Err(format!("Failed to open config file: {}", path.display())),
        };

        let mut contents = String::new();
        if let Err(_) = file.read_to_string(&mut contents) {
            return Err(format!("Failed to read config file: {}", path.display()));
        }

        if contents.is_empty() {
            return Err(format!("Config file is empty: {}", path.display()));
        }

        match toml::from_str(&contents) {
            Ok(config) => Ok(config),
            Err(_) => Err(format!("Failed to parse config file: {}", path.display())),
        }
    }

    fn to_file(&self, path: &PathBuf) -> Result<(), String> {
        let mut file = match OpenOptions::new().write(true).create(true).open(path) {
            Ok(file) => file,
            Err(_) => return Err(format!("Failed to create config file: {}", path.display())),
        };

        let contents = match toml::to_string(self) {
            Ok(contents) => contents,
            Err(err) => return Err(format!("Failed to serialize config: {}", err)),
        };

        if let Err(_) = file.write_all(contents.as_bytes()) {
            return Err(format!("Failed to write config file: {}", path.display()));
        }

        Ok(())
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
    fn get_config(&self) -> Result<Config, String> {
        let config = match Config::from_file(&self.get_config_path()) {
            Ok(config) => config,
            Err(_) => Config::new(),
        };
        Ok(config)
    }

    pub fn run(&self) -> Result<(), String> {
        let mut config = self.get_config()?;
        match self {
            ConfigCommand::Get { keys, .. } => {
                for key in keys {
                    match key.as_str() {
                        "description" => {
                            println!("{}={}", key, config.config_data.description);
                        }
                        "emoji" => {
                            println!("{}={}", key, config.config_data.emoji);
                        }
                        "language" => {
                            println!("{}={:?}", key, config.config_data.language);
                        }
                        _ => {
                            return Err(format!("Unsupported config key: {}", key));
                        }
                    }
                }
            }
            ConfigCommand::Set { key_values, .. } => {
                for key_value in key_values {
                    let parts: Vec<&str> = key_value.splitn(2, '=').collect();
                    if parts.len() != 2 {
                        return Err(String::from("Invalid argument format"));
                    }

                    let key = parts[0];
                    let value = parts[1];

                    match key {
                        "description" => match value.parse() {
                            Ok(value) => config.config_data.description = value,
                            Err(_) => return Err(String::from("Invalid value for description")),
                        },
                        "emoji" => match value.parse() {
                            Ok(value) => config.config_data.emoji = value,
                            Err(_) => return Err(String::from("Invalid value for emoji")),
                        },
                        "language" => match value {
                            "english" => config.config_data.language = Language::English,
                            _ => return Err(String::from("Unsupported language")),
                        },
                        _ => {
                            return Err(format!("Unsupported config key: {}", key));
                        }
                    }
                }

                if let Err(error) = config.to_file(&self.get_config_path()) {
                    return Err(error);
                }

                println!("Config successfully set");
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

pub fn get_config_data() -> Result<ConfigData, String> {
    let config_command = ConfigCommand::Get {
        keys: vec![],
        config_path: None,
    };
    let config_data = config_command.get_config()?.config_data;
    Ok(config_data)
}
