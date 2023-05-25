use std::fmt::Display;

use anyhow::{Error, Result};
use colored::*;
use itertools::Itertools;
use serde::Serialize;
use sysinfo::{DiskExt, System, SystemExt};

use crate::format::human_readable_size;

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
            let name = disk.name().to_str().ok_or("unknown").map_err(Error::msg)?;

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
        let free_space_percentage =
            (self.free_space as f64 / self.total_space as f64 * 100.0).round();

        let (colored_free_space, color_free_percentage) = match free_space_percentage {
            _ if free_space_percentage < 10.0 => {
                (free_space.red(), free_space_percentage.to_string().red())
            }
            _ if free_space_percentage < 20.0 => (
                free_space.yellow(),
                free_space_percentage.to_string().yellow(),
            ),
            _ => (
                free_space.green(),
                free_space_percentage.to_string().green(),
            ),
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
