use std::path::PathBuf;

use log::debug;

use super::{config_keys::ConfigKey, AutocommitConfig};

pub struct AutocommitService {
    config: AutocommitConfig,
}

impl AutocommitService {
    pub async fn new(config_path: &PathBuf) -> anyhow::Result<Self> {
        debug!("Loading config from {:?}", config_path);
        let config = AutocommitConfig::from_file_or_new(config_path).await?;
        Ok(Self { config })
    }

    pub fn update_config(&mut self, key: &ConfigKey, value: &str) -> anyhow::Result<()> {
        self.config.update_config(key, value)
    }

    pub fn get_config_value(&self, key: &ConfigKey) -> String {
        self.config.get_config_value(key)
    }

    pub fn get_config_values(&self, keys: &[ConfigKey]) -> Vec<(String, String)> {
        self.config.get_config_values(keys)
    }

    pub async fn save_config_to(&self, path: &PathBuf) -> anyhow::Result<()> {
        self.config.to_file(path).await
    }

    pub fn get_config(&self) -> &AutocommitConfig {
        &self.config
    }
}
