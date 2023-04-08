use anyhow::Result;

use crate::output::{Named, create_named};

pub async fn hostname() -> Result<Named> {
    create_named(|| async { whoami::hostname().to_string() }, "hostname").await
}

pub async fn username() -> Result<Named> {
    create_named(|| async { whoami::username().to_string() }, "username").await
}

pub async fn device_name() -> Result<Named> {
    create_named(|| async { whoami::devicename().to_string() }, "devicename").await
}

pub async fn os() -> Result<Named> {
    create_named(|| async { whoami::distro().to_string() }, "os").await
}

pub async fn architecture() -> Result<Named> {
    create_named(|| async { whoami::arch().to_string() }, "architecture").await
}

