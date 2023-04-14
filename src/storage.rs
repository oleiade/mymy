use std::fmt::Display;

use anyhow::{Error, Result};
use colored::*;
use itertools::Itertools;
use serde::Serialize;
use sysinfo::{DiskExt, System, SystemExt};

/// List all disks and their information
///
/// # Returns
///
/// A list of all disks and their information
///
/// # Errors
///
/// If the disk name cannot be converted to a string
///
/// # Examples
///
/// ```
/// let disks = storage::list_disks().unwrap();
/// println!("disks: {:?}", disks);
/// ```
pub async fn list_disks() -> Result<Vec<DiskInfo>> {
    let mut system = System::new_all();
    system.refresh_disks();
    system.refresh_disks_list();

    system
        .disks()
        .iter()
        .unique_by(|disk| disk.name())
        .map(|disk| {
            let name = disk
                .name()
                .to_str()
                .ok_or_else(|| "unknown")
                .map_err(|e| Error::msg(e))?;

            Ok(DiskInfo {
                name: name.to_string(),
                type_: format!("{:?}", disk.type_()),
                total_space: disk.total_space(),
                free_space: disk.available_space(),
            })
        })
        .collect()
}

/// Information about a disk
#[derive(Serialize)]
pub struct DiskInfo {
    pub name: String,

    #[serde(rename = "type")]
    pub type_: String,

    #[serde(rename = "total_space_bytes")]
    pub total_space: u64,

    #[serde(rename = "free_space_bytes")]
    pub free_space: u64,
}

impl Display for DiskInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let free_space = human_readable_size(self.free_space);
        let total_space = human_readable_size(self.total_space);
        let free_space_percentage = (self.free_space as f64 / self.total_space as f64 * 100.0).round();

        let (colored_free_space, color_free_percentage) = match free_space_percentage {
            _ if free_space_percentage < 10.0 => (free_space.red(), free_space_percentage.to_string().red()),
            _ if free_space_percentage < 20.0 => (free_space.yellow(), free_space_percentage.to_string().yellow()),
            _ => (free_space.green(), free_space_percentage.to_string().green()),
        };

        write!(
            f,
            "{}, {}, {} free of {} ({}% free)",
            self.name.cyan().bold(),
            self.type_.bright_white(),
            colored_free_space,
            total_space,
            color_free_percentage
        )
    }
}

/// Convert bytes to human readable size
fn human_readable_size(bytes: u64) -> String {
    const KILO: u64 = 1024;
    const MEGA: u64 = 1024 * KILO;
    const GIGA: u64 = 1024 * MEGA;
    const TERA: u64 = 1024 * GIGA;
    const PETA: u64 = 1024 * TERA;

    match bytes {
        _ if bytes < KILO => format!("{} B", bytes),
        _ if bytes < MEGA => format!("{:.2} KiB", bytes as f64 / KILO as f64),
        _ if bytes < GIGA => format!("{:.2} MiB", bytes as f64 / MEGA as f64),
        _ if bytes < TERA => format!("{:.2} GiB", bytes as f64 / GIGA as f64),
        _ if bytes < PETA => format!("{:.2} TiB", bytes as f64 / TERA as f64),
        _ => format!("{:.2} PiB", bytes as f64 / PETA as f64),
    }
}