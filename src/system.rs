use anyhow::Result;

use crate::output::{create_named, Named};

/// returns the hostname of the system as a Named enum
pub async fn hostname() -> Result<Named> {
    create_named(|| async { whoami::hostname().to_string() }, "hostname").await
}

/// returns the username of the system as a Named enum
pub async fn username() -> Result<Named> {
    create_named(|| async { whoami::username().to_string() }, "username").await
}

/// returns the device name of the system as a Named enum
pub async fn device_name() -> Result<Named> {
    create_named(|| async { whoami::devicename().to_string() }, "devicename").await
}

/// returns the operating system of the system as a Named enum
pub async fn os() -> Result<Named> {
    create_named(|| async { whoami::distro().to_string() }, "os").await
}

/// returns the architecture of the system as a Named enum
pub async fn architecture() -> Result<Named> {
    create_named(|| async { whoami::arch().to_string() }, "architecture").await
}
