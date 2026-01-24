use std::fmt::Display;

use anyhow::Result;
use serde::ser::{Serialize, SerializeMap, Serializer};

/// Named is an enum that represents a named value.
///
/// It is used to represent the output of a command, while also
/// providing a name for the value.
///
/// That is to say, it is used to represent the output of a command
/// that returns a single value, but also provides a name for that
/// value. So that the output can be serialized to JSON in a meaningful
/// way, for example.
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
            Self::Hostname(value)
            | Self::Username(value)
            | Self::DeviceName(value)
            | Self::Os(value)
            | Self::Architecture(value) => value,
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
            Self::Hostname(value) => map.serialize_entry("hostname", value)?,
            Self::Username(value) => map.serialize_entry("username", value)?,
            Self::DeviceName(value) => map.serialize_entry("device_name", value)?,
            Self::Os(value) => map.serialize_entry("os", value)?,
            Self::Architecture(value) => map.serialize_entry("architecture", value)?,
        }
        map.end()
    }
}
