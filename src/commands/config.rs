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

#[derive(Serialize, Deserialize)]
struct Config {
    description: bool,
    emoji: bool,
    language: Language,
}

impl Config {
    fn new() -> Config {
        Config {
            description: false,
            emoji: false,
            language: Language::English,
        }
    }

    fn from_file(path: &PathBuf) -> Result<Config, String> {
        let mut file = match File::open(path) {
            Ok(file) => file,
            Err(_) => return Err(String::from("Failed to open config file")),
        };

        let mut contents = String::new();
        if let Err(_) = file.read_to_string(&mut contents) {
            return Err(String::from("Failed to read config file"));
        }

        match serde_yaml::from_str(&contents) {
            Ok(config) => Ok(config),
            Err(_) => Err(String::from("Failed to parse config file")),
        }
    }

    fn to_file(&self, path: &PathBuf) -> Result<(), String> {
        let mut file = match OpenOptions::new().write(true).create(true).open(path) {
            Ok(file) => file,
            Err(_) => return Err(String::from("Failed to create config file")),
        };

        let contents = match serde_yaml::to_string(self) {
            Ok(contents) => contents,
            Err(err) => return Err(format!("Failed to serialize config: {}", err)),
        };

        if let Err(_) = file.write_all(contents.as_bytes()) {
            return Err(String::from("Failed to write config file"));
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
    pub fn run(&self) -> Result<(), String> {
        let mut config = match Config::from_file(&self.get_config_path()) {
            Ok(config) => config,
            Err(_) => Config::new(),
        };
        match self {
            ConfigCommand::Get { keys, .. } => {
                for key in keys {
                    match key.as_str() {
                        "description" => {
                            println!("{}={}", key, config.description);
                        }
                        "emoji" => {
                            println!("{}={}", key, config.emoji);
                        }
                        "language" => {
                            println!("{}={:?}", key, config.language);
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
                            Ok(value) => config.description = value,
                            Err(_) => return Err(String::from("Invalid value for description")),
                        },
                        "emoji" => match value.parse() {
                            Ok(value) => config.emoji = value,
                            Err(_) => return Err(String::from("Invalid value for emoji")),
                        },
                        "language" => match value {
                            "english" => config.language = Language::English,
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
