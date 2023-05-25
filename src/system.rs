use std::fmt::Display;

use anyhow::Result;
use colored::*;
use serde::Serialize;
use sysinfo::{CpuExt, CpuRefreshKind, RefreshKind, System, SystemExt};

use crate::format::human_readable_size;
use crate::output::{create_named, Named, NamedKind};

/// returns the hostname of the system as a Named enum
pub async fn hostname() -> Result<Named> {
    create_named(|| async { whoami::hostname() }, NamedKind::Hostname).await
}

/// returns the username of the system as a Named enum
pub async fn username() -> Result<Named> {
    create_named(|| async { whoami::username() }, NamedKind::Username).await
}

/// returns the device name of the system as a Named enum
pub async fn device_name() -> Result<Named> {
    create_named(|| async { whoami::devicename() }, NamedKind::DeviceName).await
}

/// returns the operating system of the system as a Named enum
pub async fn os() -> Result<Named> {
    create_named(|| async { whoami::distro() }, NamedKind::Os).await
}

/// returns the architecture of the system as a Named enum
pub async fn architecture() -> Result<Named> {
    create_named(
        || async { whoami::arch().to_string() },
        NamedKind::Architecture,
    )
    .await
}

/// returns the CPU of the system as a Cpu struct
pub async fn cpus() -> Result<Cpu> {
    let mut system = System::new_with_specifics(
        RefreshKind::new().with_cpu(CpuRefreshKind::new().with_frequency()),
    );
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
        write!(
            f,
            "{}, {} cores running at {} GHz",
            self.brand.bold(),
            self.core_count.to_string().cyan(),
            self.frequency.to_string().green()
        )
    }
}

/// returns the RAM of the system as a Ram struct
pub async fn ram() -> Result<Ram> {
    let mut system = System::new_with_specifics(RefreshKind::new().with_memory());
    system.refresh_memory();

    Ok(Ram {
        total: system.total_memory(),
        used: system.used_memory(),
        free: system.free_memory(),
        available: system.available_memory(),
    })
}

/// Describes the RAM of a system
#[derive(Serialize)]
pub struct Ram {
    #[serde(rename = "total_ram_bytes")]
    pub total: u64,

    #[serde(rename = "used_ram_bytes")]
    pub used: u64,

    #[serde(rename = "free_ram_bytes")]
    pub free: u64,

    #[serde(rename = "available_ram_bytes")]
    pub available: u64,
}

impl Display for Ram {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let total = human_readable_size(self.total);
        let used = human_readable_size(self.used);
        let used_percentage = (self.used as f64 / self.total as f64) * 100.0;

        let (used_colored, used_percentage_colored) = match used_percentage {
            _ if used_percentage > 90.0 => (used.red(), format!("{:.1}", used_percentage).red()),
            _ if used_percentage > 70.0 => {
                (used.yellow(), format!("{:.1}", used_percentage).yellow())
            }
            _ => (used.green(), format!("{:.1}", used_percentage).green()),
        };

        write!(
            f,
            "{} installed, {} in use ({}%)",
            total.bold(),
            used_colored,
            used_percentage_colored,
        )
    }
}
