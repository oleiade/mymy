use std::fmt::Display;

use anyhow::Result;
use colored::*;
use serde::Serialize;
use sysinfo::{CpuExt, CpuRefreshKind, RefreshKind, System, SystemExt};

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
    create_named(|| async { whoami::arch().to_string() }, NamedKind::Architecture).await
}

/// returns the CPU of the system as a Cpu struct
pub async fn cpus() -> Result<Cpu> {
    let mut system = System::new_with_specifics(RefreshKind::new().with_cpu(CpuRefreshKind::new().with_frequency()));
    system.refresh_cpu();

    let cpus = system.cpus();
    let reference_cpu = cpus.get(0).unwrap();

    Ok(Cpu {
        brand: reference_cpu.brand().to_string(),
        core_count: cpus.len(),
        frequency: reference_cpu.frequency(),
    })
}

/// Describes a CPU
#[derive(Serialize)]
pub struct Cpu {
    // The CPU's brand
    pub brand: String,

    // The CPU's name
    pub core_count: usize,

    // The CPU's frequency in MHz
    pub frequency: u64,
}

impl Display for Cpu {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}, {} cores running at {} GHz", self.brand.bold(), self.core_count.to_string().cyan(), self.frequency.to_string().green())
    }
}