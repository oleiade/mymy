use std::fmt::{Display, Formatter, Write as _};

use anyhow::{Context, Result};
use chrono::Local;
use clap::{CommandFactory, Parser, Subcommand, ValueEnum};
use colored::Colorize;
use human_panic::setup_panic;
use serde::Serialize;

mod datetime;
mod format;
mod network;
mod storage;
mod system;

#[derive(Debug, Parser)]
#[command(name = "my")]
#[command(about = "Get essential information about your device")]
#[command(
    long_about = "Easily access important details about your device, such as IP addresses, DNS servers, date, time, and more."
)]
#[command(arg_required_else_help = true)]
#[command(after_long_help = "\
Exit codes:\n  \
  0  Success (including partial success, e.g. public IP unreachable but local IP found)\n  \
  1  Command failed\n  \
  2  Invalid arguments")]
pub struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    #[arg(short, long, value_enum, default_value_t = OutputFormat::Text)]
    format: OutputFormat,

    /// When to use terminal colors
    #[arg(short, long, value_enum, default_value_t = ColorMode::Auto)]
    color: ColorMode,
}

#[derive(Debug, Subcommand)]
enum Commands {
    #[command(name = "ips")]
    #[command(about = "Display your IP addresses")]
    #[command(verbatim_doc_comment)]
    #[command(
        long_about = "Find all IP addresses associated with your system, both local and external.
By default, it shows both public and local IP addresses.
Use the --only flag to display one specific category.

Examples:
  $ my ips
  public\t93.184.216.34
  local\t192.168.1.42

  $ my ips --only public
  public\t93.184.216.34"
    )]
    Ips {
        #[arg(long)]
        only: Option<network::IpCategory>,
    },

    #[command(name = "dns")]
    #[command(subcommand_help_heading = "Network")]
    #[command(about = "Display your system's DNS servers")]
    #[command(verbatim_doc_comment)]
    #[command(
        long_about = "Show the DNS servers configured on your system, listed in the order they are used.

Example:
  $ my dns
  server 1\t8.8.8.8
  server 2\t8.8.4.4"
    )]
    Dns,

    #[command(name = "date")]
    #[command(about = "Display your system's date")]
    #[command(verbatim_doc_comment)]
    #[command(
        long_about = "Show the current date on your system in a fixed, locale-independent format:
weekday, day month, year, week number.

Example:
  $ my date
  Saturday, 8 April, 2023, week 14"
    )]
    Date,

    #[command(name = "time")]
    #[command(about = "Display your system's current time")]
    #[command(verbatim_doc_comment)]
    #[command(
        long_about = "Show the current time on your system, along with the offset from the central NTP
clock server, in a 24-hour human-readable format.

Example:
  $ my time
  20:20:02 +02:00
  +0.0672 seconds"
    )]
    Time,

    #[command(name = "datetime")]
    #[command(about = "Display your system's current date and time")]
    #[command(verbatim_doc_comment)]
    #[command(
        long_about = "Show the current date and time on your system, along with the offset from
the central NTP clock server, in a human-readable format.

Example:
  $ my datetime
  Saturday, 8 April, 2023, week 14
  20:20:02 UTC +02:00
  +0.0684 seconds"
    )]
    Datetime,

    #[command(name = "hostname")]
    #[command(about = "Display your system's hostname")]
    #[command(verbatim_doc_comment)]
    #[command(long_about = "Show the hostname assigned to your system.

Example:
  $ my hostname
  MacBook-Pro.local")]
    Hostname,

    #[command(name = "username")]
    #[command(about = "Display your current system user's username")]
    #[command(verbatim_doc_comment)]
    #[command(
        long_about = "Show the username of the currently logged-in system user.

Example:
  $ my username
  alice"
    )]
    Username,

    #[command(name = "device-name")]
    #[command(about = "Display your device's name")]
    #[command(verbatim_doc_comment)]
    #[command(long_about = "Show the configured name of your device.

Example:
  $ my device-name
  Alice's MacBook Pro")]
    DeviceName,

    #[command(name = "os")]
    #[command(about = "Display your system's OS name and version")]
    #[command(verbatim_doc_comment)]
    #[command(
        long_about = "Show the name and version of the operating system installed on your system.

Example:
  $ my os
  macOS 15.2 Sequoia"
    )]
    Os,

    #[command(name = "architecture")]
    #[command(about = "Display your system's CPU architecture")]
    #[command(verbatim_doc_comment)]
    #[command(
        long_about = "Show the architecture of the CPU installed on your system.

Example:
  $ my architecture
  aarch64"
    )]
    Architecture,

    #[command(name = "interfaces")]
    #[command(about = "Display your system's network interfaces (routable addresses only)")]
    #[command(verbatim_doc_comment)]
    #[command(
        long_about = "List the network interfaces configured on your system.

Only interfaces with routable addresses are shown by default;
loopback (127.0.0.1, ::1) and link-local (169.254.x.x, fe80::...)
entries are hidden. Use --all to include them.

Examples:
  $ my interfaces
  en0\t192.168.1.42

  $ my interfaces --all
  en0\t192.168.1.42
  en0\tfe80::1a2b:3c4d:5e6f:7890
  lo0\t127.0.0.1"
    )]
    Interfaces {
        /// Show all interfaces, including loopback and link-local
        #[arg(long)]
        all: bool,
    },

    #[command(name = "disks")]
    #[command(about = "Display your system's disks")]
    #[command(verbatim_doc_comment)]
    #[command(
        long_about = "List the disks installed on your system, showing name, type, free space,
total capacity, and percentage of free space.

Example:
  $ my disks
  Macintosh HD, SSD, 142.50 GiB free of 460.43 GiB (30.9% free)"
    )]
    Disks,

    #[command(name = "cpu")]
    #[command(about = "Display your system's CPU")]
    #[command(verbatim_doc_comment)]
    #[command(
        long_about = "Show the CPU installed on your system, including model, core count, and frequency.

Example:
  $ my cpu
  Apple M1 Pro, 10 cores running at 3.2 GHz"
    )]
    Cpu,

    #[command(name = "ram")]
    #[command(about = "Display your system's RAM")]
    #[command(verbatim_doc_comment)]
    #[command(
        long_about = "Show the amount of RAM installed and used on your system.

Example:
  $ my ram
  32.00 GiB installed, 18.50 GiB in use (57.8%)"
    )]
    Ram,

    #[command(name = "everything")]
    #[command(about = "Display a full snapshot of your system")]
    #[command(verbatim_doc_comment)]
    #[command(
        long_about = "Show all available system information at once: network, system, datetime,
and storage details in a single output.

If a piece of information cannot be retrieved, it is silently skipped
with a warning on stderr.

Examples:
  $ my everything
  $ my --format json everything"
    )]
    Everything,

    #[command(name = "completions")]
    #[command(about = "Generate shell completions")]
    #[command(verbatim_doc_comment)]
    #[command(
        long_about = "Generate shell completion scripts for the specified shell.

Supported shells: bash, zsh, fish, elvish, powershell.

Examples:
  $ my completions bash > ~/.bash_completion.d/my
  $ my completions zsh > ~/.zfunc/_my
  $ my completions fish > ~/.config/fish/completions/my.fish"
    )]
    Completions {
        #[arg(value_enum)]
        shell: clap_complete::Shell,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    // Enable human-readable panic messages
    setup_panic!();

    // Parse the CLI arguments
    let cli = Cli::parse();

    // Apply color mode before any output
    match cli.color {
        ColorMode::Always => colored::control::set_override(true),
        ColorMode::Never => colored::control::set_override(false),
        ColorMode::Auto => {} // colored crate auto-detects TTY / NO_COLOR / FORCE_COLOR
    }

    // Generate shell completions and exit early (no CommandResult needed)
    if let Some(Commands::Completions { shell }) = &cli.command {
        let mut cmd = Cli::command();
        clap_complete::generate(*shell, &mut cmd, "my", &mut std::io::stdout());
        return Ok(());
    }

    // Execute the appropriate command
    if let Some(command) = &cli.command {
        let result = execute_command(command).await?;
        display_result(&result, cli.format)?;
    }

    Ok(())
}

fn display_result(result: &CommandResult, format: OutputFormat) -> Result<()> {
    match format {
        OutputFormat::Json => {
            let json_repr = serde_json::to_string_pretty(result)?;
            println!("{json_repr}");
        }
        OutputFormat::Text => println!("{result}"),
    }

    Ok(())
}

/// A full snapshot of all system information.
#[derive(Serialize)]
struct Everything {
    #[serde(skip_serializing_if = "Option::is_none")]
    ips: Option<Vec<network::Ip>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    dns: Option<Vec<network::DnsServer>>,

    date: datetime::Date,
    time: datetime::Time,

    #[serde(skip_serializing_if = "Option::is_none")]
    hostname: Option<system::Hostname>,

    #[serde(skip_serializing_if = "Option::is_none")]
    username: Option<system::Username>,

    #[serde(skip_serializing_if = "Option::is_none")]
    device_name: Option<system::DeviceName>,

    #[serde(skip_serializing_if = "Option::is_none")]
    os: Option<system::OperatingSystem>,

    architecture: system::Architecture,

    #[serde(skip_serializing_if = "Option::is_none")]
    interfaces: Option<Vec<network::Interface>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    disks: Option<Vec<storage::DiskInfo>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    cpu: Option<system::Cpu>,

    ram: system::Ram,
}

impl Display for Everything {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        // ── SYSTEM ──
        // First section has no leading newline
        writeln!(f, " {}", "SYSTEM".bold().bright_blue())?;

        if let Some(hostname) = &self.hostname {
            write_field(f, "hostname", hostname)?;
        }
        if let Some(username) = &self.username {
            write_field(f, "username", username)?;
        }
        if let Some(device_name) = &self.device_name {
            write_field(f, "device-name", device_name)?;
        }
        if let Some(os) = &self.os {
            write_field(f, "os", os)?;
        }
        write_field(f, "architecture", &self.architecture)?;
        if let Some(cpu) = &self.cpu {
            write_field(f, "cpu", cpu)?;
        }
        write_field(f, "ram", &self.ram)?;

        // ── DATETIME ──
        write_section_header(f, "DATETIME")?;

        write_field(f, "date", &self.date)?;
        write_multiline_field(f, "time", &self.time)?;

        // ── STORAGE ──
        write_section_header(f, "STORAGE")?;

        if let Some(disks) = &self.disks {
            write_vec_field(f, "disks", disks)?;
        }

        // ── NETWORK ──
        write_section_header(f, "NETWORK")?;

        self.write_network_section(f)?;

        Ok(())
    }
}

impl Everything {
    /// Writes all network fields with a common inner column width so values align.
    fn write_network_section(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        // Collect (inner_label, value) pairs across all network subsections
        let mut rows: Vec<(&str, String, String)> = Vec::new();

        if let Some(ips) = &self.ips {
            for ip in ips {
                rows.push(("ips", ip.category.to_string(), ip.address.to_string()));
            }
        }
        if let Some(dns) = &self.dns {
            for server in dns {
                rows.push((
                    "dns",
                    format!("server {}", server.order),
                    server.address.to_string(),
                ));
            }
        }
        if let Some(interfaces) = &self.interfaces {
            for iface in interfaces {
                rows.push(("interfaces", iface.name.clone(), iface.ip.to_string()));
            }
        }

        if rows.is_empty() {
            return Ok(());
        }

        // Compute inner column width from the widest inner label
        let inner_width = rows.iter().map(|(_, lbl, _)| lbl.len()).max().unwrap_or(0) + 1;

        let mut current_field: Option<&str> = None;
        for (field, inner_label, value) in &rows {
            if current_field == Some(field) {
                // Continuation row: indent past the outer label
                let indent = 2 + LABEL_WIDTH;
                writeln!(
                    f,
                    "{:indent$}{:<iw$}{}",
                    "",
                    inner_label,
                    value,
                    iw = inner_width,
                )?;
            } else {
                // First row of a new field: print the outer label
                writeln!(
                    f,
                    "  {:<lw$}{:<iw$}{}",
                    field.dimmed(),
                    inner_label,
                    value,
                    lw = LABEL_WIDTH,
                    iw = inner_width,
                )?;
                current_field = Some(field);
            }
        }

        Ok(())
    }
}

/// Width of the label column for alignment.
const LABEL_WIDTH: usize = 15;

fn write_section_header(f: &mut Formatter<'_>, title: &str) -> std::fmt::Result {
    write!(f, "\n {}\n", title.bold().bright_blue())
}

fn write_field(f: &mut Formatter<'_>, label: &str, value: &impl Display) -> std::fmt::Result {
    writeln!(
        f,
        "  {:<width$}{}",
        label.dimmed(),
        value,
        width = LABEL_WIDTH
    )
}

fn write_multiline_field(
    f: &mut Formatter<'_>,
    label: &str,
    value: &impl Display,
) -> std::fmt::Result {
    let text = format!("{value}");
    let mut lines = text.lines();

    if let Some(first) = lines.next() {
        writeln!(
            f,
            "  {:<width$}{first}",
            label.dimmed(),
            width = LABEL_WIDTH
        )?;
        let indent = 2 + LABEL_WIDTH;
        for line in lines {
            writeln!(f, "{:indent$}{line}", "")?;
        }
    }

    Ok(())
}

fn write_vec_field<T: Display>(
    f: &mut Formatter<'_>,
    label: &str,
    items: &[T],
) -> std::fmt::Result {
    let indent = 2 + LABEL_WIDTH;
    let mut iter = items.iter();

    if let Some(first) = iter.next() {
        let first_text = {
            let mut buf = String::new();
            write!(buf, "{first}")?;
            buf
        };
        let mut first_lines = first_text.lines();

        if let Some(first_line) = first_lines.next() {
            writeln!(
                f,
                "  {:<width$}{first_line}",
                label.dimmed(),
                width = LABEL_WIDTH
            )?;
            for line in first_lines {
                writeln!(f, "{:indent$}{line}", "")?;
            }
        }

        for item in iter {
            let text = {
                let mut buf = String::new();
                write!(buf, "{item}")?;
                buf
            };
            for line in text.lines() {
                writeln!(f, "{:indent$}{line}", "")?;
            }
        }
    }

    Ok(())
}

/// `CommandResult` holds the result of a command.
///
/// This is used to facilitate factorizing the command execution,
/// Wrapper structs for list-typed commands, ensuring consistent JSON output
/// with a named key (e.g. `{"ips": [...]}` instead of a bare array).
#[derive(Serialize)]
struct Ips {
    ips: Vec<network::Ip>,
}

#[derive(Serialize)]
struct DnsServers {
    dns_servers: Vec<network::DnsServer>,
}

#[derive(Serialize)]
struct Interfaces {
    interfaces: Vec<network::Interface>,
}

#[derive(Serialize)]
struct Disks {
    disks: Vec<storage::DiskInfo>,
}

/// and allow handling the serializing of the result into the desired output format
/// in a single place.
#[derive(Serialize)]
#[serde(untagged)]
enum CommandResult {
    Ips(Ips),
    Dns(DnsServers),
    Date(datetime::Date),
    Time(datetime::Time),
    Datetime(datetime::Datetime),
    Hostname(system::Hostname),
    Username(system::Username),
    DeviceName(system::DeviceName),
    Os(system::OperatingSystem),
    Architecture(system::Architecture),
    Interfaces(Interfaces),
    Disks(Disks),
    Cpu(system::Cpu),
    Ram(system::Ram),
    Everything(Box<Everything>),
}

impl Display for CommandResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Ips(wrapper) => {
                let ips = wrapper.ips.iter().map(ToString::to_string).collect::<Vec<String>>();
                write!(f, "{}", ips.join("\n"))
            }
            Self::Dns(wrapper) => {
                write!(
                    f,
                    "{}",
                    wrapper.dns_servers
                        .iter()
                        .map(ToString::to_string)
                        .collect::<Vec<String>>()
                        .join("\n")
                )
            }
            Self::Date(date) => date.fmt(f),
            Self::Time(time) => time.fmt(f),
            Self::Datetime(datetime) => datetime.fmt(f),
            Self::Hostname(hostname) => hostname.fmt(f),
            Self::Username(username) => username.fmt(f),
            Self::DeviceName(device_name) => device_name.fmt(f),
            Self::Os(os) => os.fmt(f),
            Self::Architecture(architecture) => architecture.fmt(f),
            Self::Interfaces(wrapper) => {
                write!(
                    f,
                    "{}",
                    wrapper.interfaces
                        .iter()
                        .map(ToString::to_string)
                        .collect::<Vec<String>>()
                        .join("\n")
                )
            }
            Self::Disks(wrapper) => {
                write!(
                    f,
                    "{}",
                    wrapper.disks
                        .iter()
                        .map(ToString::to_string)
                        .collect::<Vec<String>>()
                        .join("\n")
                )
            }
            Self::Cpu(cpu) => cpu.fmt(f),
            Self::Ram(ram) => ram.fmt(f),
            Self::Everything(everything) => everything.fmt(f),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum OutputFormat {
    Json,
    Text,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum ColorMode {
    /// Detect automatically (default; respects `NO_COLOR` / `FORCE_COLOR` and `TTY`)
    Auto,
    /// Always emit ANSI colors
    Always,
    /// Never emit ANSI colors
    Never,
}

async fn execute_command(command: &Commands) -> Result<CommandResult> {
    match command {
        Commands::Date => Ok(handle_date()),
        Commands::Time => Ok(handle_time().await),
        Commands::Datetime => Ok(handle_datetime().await),
        Commands::Dns => handle_dns(),
        Commands::Ips { only } => handle_ips(*only).await,
        Commands::Hostname => handle_hostname(),
        Commands::Username => handle_username(),
        Commands::DeviceName => handle_device_name(),
        Commands::Os => handle_os_command(),
        Commands::Architecture => Ok(handle_architecture()),
        Commands::Interfaces { all } => handle_interfaces(*all),
        Commands::Disks => handle_disks(),
        Commands::Cpu => handle_cpu(),
        Commands::Ram => Ok(handle_ram()),
        Commands::Everything => Ok(handle_everything().await),
        Commands::Completions { .. } => unreachable!("handled before execute_command"),
    }
}

fn handle_date() -> CommandResult {
    CommandResult::Date(datetime::date(Some(Local::now())))
}

async fn handle_time() -> CommandResult {
    CommandResult::Time(datetime::time(Some(Local::now())).await)
}

async fn handle_datetime() -> CommandResult {
    CommandResult::Datetime(datetime::datetime().await)
}

fn handle_dns() -> Result<CommandResult> {
    let servers = network::list_dns_servers().context("listing the system's dns servers failed")?;
    Ok(CommandResult::Dns(DnsServers { dns_servers: servers }))
}

async fn handle_ips(only: Option<network::IpCategory>) -> Result<CommandResult> {
    let open_dns_host = network::OPENDNS_SERVER_HOST;
    let open_dns_port = network::DNS_DEFAULT_PORT;

    match only {
        Some(network::IpCategory::Public) => {
            let public_ip = network::query_public_ip(open_dns_host, open_dns_port)
                .await
                .with_context(|| {
                    format!(
                        "looking up public ip failed; reason: querying dns server {open_dns_host} on port {open_dns_port} failed"
                    )
                })?;
            Ok(CommandResult::Ips(Ips { ips: vec![network::Ip {
                category: network::IpCategory::Public,
                address: public_ip,
            }]}))
        }
        Some(network::IpCategory::Local) => {
            let local_ip = local_ip_address::local_ip().with_context(
                || "looking up local ip failed; reason: querying local ip address failed",
            )?;
            Ok(CommandResult::Ips(Ips { ips: vec![network::Ip {
                category: network::IpCategory::Local,
                address: local_ip,
            }]}))
        }
        Some(network::IpCategory::Any) | None => {
            let ips = gather_ips()
                .await
                .ok_or_else(|| anyhow::anyhow!("could not determine any IP addresses"))?;
            Ok(CommandResult::Ips(Ips { ips }))
        }
    }
}

fn handle_hostname() -> Result<CommandResult> {
    system::hostname().map(CommandResult::Hostname)
}

fn handle_username() -> Result<CommandResult> {
    system::username().map(CommandResult::Username)
}

fn handle_device_name() -> Result<CommandResult> {
    system::device_name().map(CommandResult::DeviceName)
}

fn handle_os_command() -> Result<CommandResult> {
    system::os().map(CommandResult::Os)
}

fn handle_architecture() -> CommandResult {
    CommandResult::Architecture(system::architecture())
}

fn handle_interfaces(show_all: bool) -> Result<CommandResult> {
    let mut interfaces =
        network::interfaces().with_context(|| "listing the system's network interfaces failed")?;
    if !show_all {
        interfaces.retain(network::is_default_visible);
    }
    Ok(CommandResult::Interfaces(Interfaces { interfaces }))
}

fn handle_disks() -> Result<CommandResult> {
    let disks = storage::list_disks().context("listing the disks failed")?;
    Ok(CommandResult::Disks(Disks { disks }))
}

fn handle_cpu() -> Result<CommandResult> {
    let cpus = system::cpus().context("listing the system's cpus failed")?;
    Ok(CommandResult::Cpu(cpus))
}

fn handle_ram() -> CommandResult {
    CommandResult::Ram(system::ram())
}

/// Converts a `Result<T>` into `Option<T>`, printing a warning to stderr on error.
fn warn_on_err<T>(result: Result<T>, label: &str) -> Option<T> {
    match result {
        Ok(v) => Some(v),
        Err(e) => {
            eprintln!("warning: could not determine {label}: {e}");
            None
        }
    }
}

/// Best-effort collection of public and local IP addresses.
async fn gather_ips() -> Option<Vec<network::Ip>> {
    let mut ips = Vec::new();

    match network::query_public_ip(network::OPENDNS_SERVER_HOST, network::DNS_DEFAULT_PORT).await {
        Ok(public_ip) => ips.push(network::Ip {
            category: network::IpCategory::Public,
            address: public_ip,
        }),
        Err(e) => eprintln!("warning: could not determine public IP: {e}"),
    }

    match local_ip_address::local_ip() {
        Ok(local_ip) => ips.push(network::Ip {
            category: network::IpCategory::Local,
            address: local_ip,
        }),
        Err(e) => eprintln!("warning: could not determine local IP: {e}"),
    }

    if ips.is_empty() { None } else { Some(ips) }
}

async fn handle_everything() -> CommandResult {
    let now = Local::now();

    // Async operations run concurrently
    let (ips, time) = tokio::join!(gather_ips(), datetime::time(Some(now)));

    // Sync operations
    let date = datetime::date(Some(now));
    let dns = warn_on_err(
        network::list_dns_servers().context("listing DNS servers"),
        "dns servers",
    );
    let hostname = warn_on_err(system::hostname(), "hostname");
    let username = warn_on_err(system::username(), "username");
    let device_name = warn_on_err(system::device_name(), "device name");
    let os = warn_on_err(system::os(), "os");
    let architecture = system::architecture();
    let interfaces = warn_on_err(
        network::interfaces().context("listing network interfaces"),
        "network interfaces",
    )
    .map(|mut ifaces| {
        ifaces.retain(network::is_default_visible);
        ifaces
    });
    let disks = warn_on_err(storage::list_disks().context("listing disks"), "disks");
    let cpu = warn_on_err(system::cpus().context("listing CPUs"), "cpu");
    let ram = system::ram();

    CommandResult::Everything(Box::new(Everything {
        ips,
        dns,
        date,
        time,
        hostname,
        username,
        device_name,
        os,
        architecture,
        interfaces,
        disks,
        cpu,
        ram,
    }))
}
