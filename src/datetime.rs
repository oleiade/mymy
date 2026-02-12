use std::fmt::{Display, Formatter};
use std::time::Duration;

use chrono::{DateTime, Datelike, Local, Timelike};
use colored::Colorize;
use rsntp::AsyncSntpClient;
use serde::Serialize;

/// Returns the system date.
pub fn date(now: Option<DateTime<Local>>) -> Date {
    let now = now.unwrap_or(Local::now());
    Date::from(now)
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
pub async fn time(now: Option<DateTime<Local>>) -> Time {
    let now = now.unwrap_or_else(|| Local::now());
    let mut t = Time::from(now);

    let client = AsyncSntpClient::new();
    match tokio::time::timeout(Duration::from_secs(5), client.synchronize("pool.ntp.org")).await {
        Ok(Ok(sntp_time)) => {
            t.offset = Some(sntp_time.clock_offset().as_secs_f64());
        }
        Ok(Err(e)) => eprintln!("warning: could not sync with NTP server: {e}"),
        Err(_) => eprintln!("warning: NTP query timed out"),
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
        write!(
            f,
            "{hour}:{minute}:{second} {}",
            self.timezone.bright_cyan()
        )?;

        if let Some(offset) = self.offset {
            let sign = if offset >= 0.0 { '+' } else { '-' };
            let label = match offset.abs() {
                d if d < 0.5 => "in sync",
                d if d < 2.0 => "slightly off",
                _ => "significantly off",
            };
            write!(
                f,
                "\n{} {}{}{} {}",
                "clock offset".dimmed(),
                sign,
                format!("{:.4}", offset.abs()).bright_magenta(),
                "s".bright_magenta(),
                format!("({label})").bright_cyan()
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
pub async fn datetime() -> Datetime {
    let now = Local::now();
    let date = date(Some(now));
    let time = time(Some(now)).await;

    Datetime { date, time }
}

#[derive(Serialize)]
pub struct Datetime {
    #[serde(flatten)]
    date: Date,

    #[serde(flatten)]
    time: Time,
}

impl Display for Datetime {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.date)?;
        write!(f, "\n{}", self.time)
    }
}
