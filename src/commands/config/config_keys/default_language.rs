use serde::Serialize;

use crate::i18n::language::Language;
use anyhow::anyhow;
use std::{fmt, str::FromStr};

use super::config_item::ConfigValue;

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

    fn update(&mut self, value: &str) -> anyhow::Result<()> {
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
