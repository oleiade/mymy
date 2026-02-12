use std::fmt::Display;

use anyhow::{Error, Result};
use colored::Colorize;
use itertools::Itertools;
use serde::Serialize;
use sysinfo::Disks;

use crate::format::{Percentage, human_readable_size};

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
/// ```no_run
/// # use anyhow::Result;
/// # fn example() -> Result<()> {
/// use mymy::storage;
///
/// let disks = storage::list_disks()?;
/// println!("disks: {:?}", disks);
/// # Ok(())
/// # }
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
                type_: disk.kind().to_string(),
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
        let used_space = self.total_space - self.free_space;
        let used = human_readable_size(used_space);
        let total = human_readable_size(self.total_space);

        let percentage = Percentage::from_ratio(used_space, self.total_space);
        let percentage_display = format!("{percentage}");

        let (used_colored, used_percentage_colored, indicator) = match percentage.tenths {
            p if p > 900 => (used.red(), percentage_display.as_str().red(), " !"),
            p if p > 700 => (
                used.yellow(),
                percentage_display.as_str().yellow(),
                " \u{25b2}",
            ),
            _ => (used.green(), percentage_display.as_str().green(), ""),
        };

        write!(
            f,
            "{}, {}, {} used of {} ({}%{})",
            self.name.cyan().bold(),
            self.type_.bright_white(),
            used_colored,
            total,
            used_percentage_colored,
            indicator,
        )
    }
}
