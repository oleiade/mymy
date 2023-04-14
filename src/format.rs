/// Convert bytes to human readable size
pub fn human_readable_size(bytes: u64) -> String {
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