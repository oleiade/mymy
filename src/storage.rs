use std::convert::TryFrom;
use std::fmt::Display;

use anyhow::{Error, Result};
use colored::Colorize;
use itertools::Itertools;
use serde::Serialize;
use sysinfo::Disks;

use crate::format::{human_readable_size, Percentage};

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
pub fn list_disks() -> Result<Vec<DiskInfo>> {
    let disks = Disks::new_with_refreshed_list();

    disks
        .iter()
        .unique_by(|disk| disk.name())
        .map(|disk| {
            let name = disk
                .name()
                .to_str()
                .ok_or_else(|| Error::msg("invalid disk name"))?;

            Ok(DiskInfo {
                name: name.to_string(),
                type_: format!("{:?}", disk.kind()),
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

        let percentage = Percentage::from_ratio(self.free_space, self.total_space);
        let percentage_display = format!("{percentage}");

        let (colored_free_space, color_free_percentage) = match percentage.tenths {
            _ if percentage.tenths < 100 => (free_space.red(), percentage_display.as_str().red()),
            _ if percentage.tenths < 200 => {
                (free_space.yellow(), percentage_display.as_str().yellow())
            }
            _ => (free_space.green(), percentage_display.as_str().green()),
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
