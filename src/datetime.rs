use std::fmt::{Display, Formatter};

use anyhow::Result;
use chrono::{DateTime, Datelike, Local, Timelike};
use colored::Colorize;
use rsntp::AsyncSntpClient;
use serde::Serialize;

/// Returns the system date.
pub fn date() -> Date {
    let dt = Local::now();
    let now_with_tz = dt.with_timezone(&Local);

    now_with_tz.into()
}

#[derive(Serialize)]
pub struct Date {
    day_name: String,
    day_number: u32,
    month_name: String,
    year: i32,
    week_number: u32,
}

impl Display for Date {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.day_name)?;
        write!(f, ", {} {}", self.day_number, self.month_name)?;
        write!(f, ", {}", self.year)?;
        write!(f, ", week {}", self.week_number)
    }
}

impl From<DateTime<Local>> for Date {
    fn from(dt: DateTime<Local>) -> Self {
        Self {
            day_name: dt.format("%A").to_string(),
            day_number: dt.day(),
            month_name: dt.format("%B").to_string(),
            year: dt.year(),
            week_number: dt.iso_week().week(),
        }
    }
}

/// Returns the system time.
pub async fn time() -> Time {
    let now = Local::now();
    let mut t = Time::from(now);

    match AsyncSntpClient::new().synchronize("pool.ntp.org").await {
        Ok(sntp_time) => {
            t.offset = Some(sntp_time.clock_offset().as_secs_f64());
        },
        Err(e) => eprintln!("warning: NTP sync failed: {e}")
    }

    t
}

#[derive(Serialize)]
pub struct Time {
    hour: u32,
    minute: u32,
    second: u32,
    timezone: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    offset: Option<f64>,
}

impl Display for Time {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let hour = format!("{:02}", self.hour).bold();
        let minute = format!("{:02}", self.minute).bold();
        let second = format!("{:02}", self.second);
        write!(f, "{hour}:{minute}:{second} {}", self.timezone.bright_cyan())?;

        if let Some(offset) = self.offset {
            let sign = if offset >= 0.0 { '+' } else { '-' };
            write!(
                f,
                "\n{}{} seconds",
                sign,
                format!("{:.4}", offset.abs()).bright_magenta()
            )?;
        }

        Ok(())
    }
}

impl From<DateTime<Local>> for Time {
    fn from(dt: DateTime<Local>) -> Self {
        Self {
            hour: dt.hour(),
            minute: dt.minute(),
            second: dt.second(),
            timezone: dt.format("%Z").to_string(),
            offset: None,
        }
    }
}

/// Returns the system date and time.
pub async fn datetime() -> Result<Datetime> {
    let date = date();
    let time = time().await;

    Ok(Datetime { date, time })
}

#[derive(Serialize)]
pub struct Datetime {
    date: Date,
    time: Time,
}

impl Display for Datetime {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.date)?;
        write!(f, "\n{}", self.time)
    }
}
