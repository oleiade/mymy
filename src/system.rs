use std::fmt::Display;

use anyhow::{Context, Result};
use colored::Colorize;
use serde::Serialize;
use sysinfo::{CpuRefreshKind, MemoryRefreshKind, RefreshKind, System};
use crate::format::{human_readable_size, Percentage};

#[derive(Serialize)]
pub struct Hostname {
   pub hostname: String,
}

impl Display for Hostname {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.hostname)
    }
}

/// returns the hostname of the system as a Named enum
pub fn hostname() -> Result<Hostname> {
    let hostname = whoami::hostname()?;
    Ok(Hostname{ hostname })
}

#[derive(Serialize)]
pub struct Username {
    pub username: String,
}

impl Display for Username {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.username)
    }
}

/// returns the username of the system as a Named enum
pub fn username() -> Result<Username> {
    let username = whoami::username()?;
    Ok(Username { username })
}

#[derive(Serialize)]
pub struct DeviceName {
    pub device_name: String,
}

impl Display for DeviceName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.device_name)
    }
}

/// returns the device name of the system as a Named enum
pub fn device_name() -> Result<DeviceName> {
    let device_name = whoami::devicename()?;
    Ok(DeviceName{ device_name })
}

#[derive(Serialize)]
pub struct OperatingSystem {
    pub name: String,
}

impl Display for OperatingSystem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

/// returns the operating system of the system as a Named enum
pub fn os() -> Result<OperatingSystem> {
    let name = whoami::distro()?;
    Ok(OperatingSystem{ name })
}

#[derive(Serialize)]
pub struct Architecture {
    pub architecture: String,
}

impl Display for Architecture {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.architecture)
    }
}

/// returns the architecture of the system as a Named enum
pub fn architecture() -> Architecture {
    Architecture{ architecture: whoami::cpu_arch().to_string() }
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


