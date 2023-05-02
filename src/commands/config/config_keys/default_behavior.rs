use std::{fmt, str::FromStr};

use super::config_item::ConfigValue;
use anyhow::anyhow;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use strum::{Display, EnumString};

#[derive(Debug, Serialize)]
pub struct DefaultBehaviorOption(Option<DefaultBehavior>);

impl DefaultBehaviorOption {
    pub fn get_inner_value(&self) -> Option<DefaultBehavior> {
        self.0.clone()
    }
}

impl Default for DefaultBehaviorOption {
    fn default() -> Self {
        Self(None)
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

    fn update(&mut self, value: &str) -> anyhow::Result<()> {
        match value.parse() {
            Ok(value) => self.0 = Some(value),
            Err(_) => return Err(anyhow!("Invalid value for default behavior")),
        }

        Ok(())
    }

    fn get_value(&self) -> String {
        format!("{:?}", self.0.as_ref().map(|s| s.to_string())).to_lowercase()
    }
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

impl DefaultBehavior {
    pub fn is_yes(&self) -> bool {
        matches!(self, DefaultBehavior::Yes)
    }

    pub fn is_no(&self) -> bool {
        matches!(self, DefaultBehavior::No)
    }

    #[allow(dead_code)]
    pub fn is_ask(&self) -> bool {
        matches!(self, DefaultBehavior::Ask)
    }
}
