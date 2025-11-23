use std::fmt::{Display, Formatter};

use anyhow::Result;
use chrono::{DateTime, Local};
use colored::*;
use rsntp::AsyncSntpClient;
use serde::Serialize;

/// Returns the system date.
pub async fn date() -> Result<Date> {
    let dt = Local::now();
    let now_with_tz = dt.with_timezone(&Local);

    Ok(now_with_tz.into())
}

#[derive(Serialize)]
pub struct Date {
    day_name: String,
    day_number: u8,
    month_name: String,
    year: i32,
    week_number: u8,
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
        Date {
            day_name: dt.format("%A").to_string(),
            day_number: dt.format("%d").to_string().parse::<u8>().unwrap(),
            month_name: dt.format("%B").to_string(),
            year: dt.format("%Y").to_string().parse::<i32>().unwrap(),
            week_number: dt.format("%U").to_string().parse::<u8>().unwrap(),
        }
    }
}

/// Returns the system time.
pub async fn time() -> Result<Time> {
    let sntp_client = AsyncSntpClient::new();
    let sntp_time = sntp_client.synchronize("pool.ntp.org").await?;
    let now = sntp_time.datetime().into_chrono_datetime()?;
    let now_with_tz = now.with_timezone(&Local);

    let mut t = Time::from(now_with_tz);
    t.offset = sntp_time.clock_offset().as_secs_f64();

    Ok(t)
}

#[derive(Serialize)]
pub struct Time {
    hour: u8,
    minute: u8,
    second: u8,
    timezone: String,
    offset: f64,
}

impl Display for Time {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.hour.to_string().bold())?;
        write!(f, ":{}", self.minute.to_string().bold())?;
        write!(f, ":{}", self.second.to_string())?;
        write!(f, " UTC {}", self.timezone.bright_cyan())?;
        write!(
            f,
            "\n±{:.4} seconds",
            self.offset.to_string().bright_magenta()
        )
    }
}

impl From<DateTime<Local>> for Time {
    fn from(dt: DateTime<Local>) -> Self {
        Time {
            hour: dt.format("%H").to_string().parse::<u8>().unwrap(),
            minute: dt.format("%M").to_string().parse::<u8>().unwrap(),
            second: dt.format("%S").to_string().parse::<u8>().unwrap(),
            timezone: dt.format("%Z").to_string(),
            offset: 0.0,
        }
    }
}

/// Returns the system date and time.
pub async fn datetime() -> Result<Datetime> {
    let date = date().await?;
    let time = time().await?;

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
