use std::fmt::Display;

use anyhow::Result;
use serde::ser::{Serialize, SerializeMap, Serializer};


pub enum Named {
    Hostname(String),
    Username(String),
    DeviceName(String),
    Os(String),
    Architecture(String),
}

impl Named {
    fn value(&self) -> &str {
        match self {
            Named::Hostname(value)
            | Named::Username(value)
            | Named::DeviceName(value)
            | Named::Os(value)
            | Named::Architecture(value) => value,
        }
    }
}

impl Display for Named {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value())
    }
}

impl Serialize for Named {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(1))?;
        match self {
            Named::Hostname(value) => map.serialize_entry("hostname", value)?,
            Named::Username(value) => map.serialize_entry("username", value)?,
            Named::DeviceName(value) => map.serialize_entry("device_name", value)?,
            Named::Os(value) => map.serialize_entry("os", value)?,
            Named::Architecture(value) => map.serialize_entry("architecture", value)?,
        }
        map.end()
    }
}

pub async fn create_named<F, Fut>(func: F, data_type: &'static str) -> Result<Named>
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = String>,
{
    let value = func().await;
    match data_type {
        "hostname" => Ok(Named::Hostname(value)),
        "username" => Ok(Named::Username(value)),
        "devicename" => Ok(Named::DeviceName(value)),
        "os" => Ok(Named::Os(value)),
        "architecture" => Ok(Named::Architecture(value)),
        _ => panic!("Invalid data type: {}", data_type)
    }
}