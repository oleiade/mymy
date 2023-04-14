use anyhow::Result;

use crate::output::{create_named, Named, NamedKind};

/// returns the hostname of the system as a Named enum
pub async fn hostname() -> Result<Named> {
    create_named(|| async { whoami::hostname().to_string() }, NamedKind::Hostname).await
}

/// returns the username of the system as a Named enum
pub async fn username() -> Result<Named> {
    create_named(|| async { whoami::username().to_string() }, NamedKind::Username).await
}

/// returns the device name of the system as a Named enum
pub async fn device_name() -> Result<Named> {
    create_named(|| async { whoami::devicename().to_string() }, NamedKind::DeviceName).await
}

/// returns the operating system of the system as a Named enum
pub async fn os() -> Result<Named> {
    create_named(|| async { whoami::distro().to_string() }, NamedKind::Os).await
}

/// returns the architecture of the system as a Named enum
pub async fn architecture() -> Result<Named> {
    create_named(|| async { whoami::arch().to_string() }, "architecture").await
}
