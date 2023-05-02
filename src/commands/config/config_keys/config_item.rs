use anyhow::anyhow;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::str::FromStr;
pub trait ConfigValue: FromStr + ToString {
    fn validate(&self) -> anyhow::Result<()>;
    fn update(&mut self, value: &str) -> anyhow::Result<()>;
    fn get_value(&self) -> String;
}

impl ConfigValue for bool {
    fn validate(&self) -> anyhow::Result<()> {
        Ok(())
    }

    fn update(&mut self, value: &str) -> anyhow::Result<()> {
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

impl ConfigValue for String {
    fn validate(&self) -> anyhow::Result<()> {
        Ok(())
    }

    fn update(&mut self, value: &str) -> anyhow::Result<()> {
        *self = value.to_owned();
        Ok(())
    }

    fn get_value(&self) -> String {
        self.to_owned()
    }
}

#[derive(Debug)]
pub struct ConfigItem<T>
where
    T: ConfigValue,
{
    pub value: T,
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
    pub fn new(value: T) -> Self {
        Self { value }
    }

    pub fn update(&mut self, value: &str) -> anyhow::Result<()> {
        self.value.update(value)?;
        self.value.validate()?;
        Ok(())
    }

    pub fn get_value(&self) -> String {
        self.value.get_value()
    }

    pub fn get_value_ref(&self) -> &T {
        &self.value
    }
}
