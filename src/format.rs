use fmt::Display;
use std::fmt;

/// Convert bytes to human readable size
pub fn human_readable_size(bytes: u64) -> String {
    const KILO: u64 = 1024;
    const MEGA: u64 = 1024 * KILO;
    const GIGA: u64 = 1024 * MEGA;
    const TERA: u64 = 1024 * GIGA;
    const PETA: u64 = 1024 * TERA;

    fn format_scaled(bytes: u64, unit: u64, suffix: &str) -> String {
        let whole = bytes / unit;
        let remainder = bytes % unit;
        let decimals = remainder * 100 / unit;
        format!("{whole}.{decimals:02} {suffix}")
    }

    match bytes {
        _ if bytes < KILO => format!("{bytes} B"),
        _ if bytes < MEGA => format_scaled(bytes, KILO, "KiB"),
        _ if bytes < GIGA => format_scaled(bytes, MEGA, "MiB"),
        _ if bytes < TERA => format_scaled(bytes, GIGA, "GiB"),
        _ if bytes < PETA => format_scaled(bytes, TERA, "TiB"),
        _ => format_scaled(bytes, PETA, "PiB"),
    }
}

pub struct Percentage {
    pub(crate) tenths: u64,
}

impl Percentage {
    pub fn from_ratio(numerator: u64, denominator: u64) -> Self {
        let tenths = if denominator == 0 {
            0
        } else {
            u64::try_from(u128::from(numerator) * 1000 / u128::from(denominator))
                .unwrap_or(u64::MAX)
        };

        Self { tenths }
    }
}

impl Display for Percentage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}", self.tenths / 10, self.tenths % 10)
    }
}
