use std::fmt;

use serde::Serialize;

use super::config_item::ConfigValue;
use anyhow::anyhow;

#[derive(Debug, Serialize)]
pub struct OptionString(Option<String>);
impl OptionString {
    pub fn get_inner_value(&self) -> Option<String> {
        self.0.clone()
    }
}

impl Default for OptionString {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl std::str::FromStr for OptionString {
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

    fn update(&mut self, value: &str) -> anyhow::Result<()> {
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
