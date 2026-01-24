use std::convert::TryFrom;
use std::fmt::Display;

use anyhow::{Context, Result};
use colored::Colorize;
use serde::Serialize;
use sysinfo::{CpuRefreshKind, MemoryRefreshKind, RefreshKind, System};

use crate::format::{human_readable_size, Percentage};
use crate::output::{Named};

/// returns the hostname of the system as a Named enum
pub fn hostname() -> Result<Named> {
    let hostname = whoami::fallible::hostname()?;
    Ok(Named::Hostname(hostname))
}

/// returns the username of the system as a Named enum
pub fn username() -> Named {
    Named::Username(whoami::username())
}

/// returns the device name of the system as a Named enum
pub fn device_name() -> Named {
    Named::DeviceName(whoami::devicename())
}

/// returns the operating system of the system as a Named enum
pub fn os() -> Named {
    Named::Os(whoami::distro())
}

/// returns the architecture of the system as a Named enum
pub fn architecture() -> Named {
    Named::Architecture(whoami::arch().to_string())
}

/// returns the CPU of the system as a Cpu struct
pub fn cpus() -> Result<Cpu> {
    let system =
        System::new_with_specifics(RefreshKind::nothing().with_cpu(CpuRefreshKind::everything()));

    let cpus = system.cpus();
    let reference_cpu = cpus
        .first()
        .context("no CPU information available from sysinfo")?;

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
            format!("{}", self.core_count).cyan(),
            format!("{}", self.frequency / 1000).green()
        )
    }
}

/// returns the RAM of the system as a Ram struct
pub fn ram() -> Ram {
    let system = System::new_with_specifics(
        RefreshKind::nothing().with_memory(MemoryRefreshKind::everything()),
    );

    Ram {
        total: system.total_memory(),
        used: system.used_memory(),
        free: system.free_memory(),
        available: system.available_memory(),
    }
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

        let percentage = Percentage::from_ratio(self.used, self.total);
        let percentage_display = format!("{percentage}");

        let (used_colored, used_percentage_colored) = match percentage.tenths {
            p if p > 900 => (used.red(), percentage_display.as_str().red()),
            p if p > 700 => (used.yellow(), percentage_display.as_str().yellow()),
            _ => (used.green(), percentage_display.as_str().green()),
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
