use std::fmt::Display;
use std::future::Future;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand, ValueEnum};
use human_panic::setup_panic;
use serde::{Serialize, Serializer};

mod country;
mod datetime;
mod format;
mod network;
mod output;
mod storage;
mod system;

#[derive(Debug, Parser)]
#[command(name = "my")]
#[command(about = "Get essential information about your device")]
#[command(
    long_about = "Easily access important details about your device, such as IP addresses, DNS servers, date, time, and more."
)]
pub struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    #[arg(short, long, value_enum, default_value_t = OutputFormat::Text)]
    format: OutputFormat,
}

#[derive(Debug, Subcommand)]
enum Commands {
    #[command(name = "ips")]
    #[command(about = "Display your IP addresses")]
    #[command(
        long_about = "Find all IP addresses associated with your system, both local and external.\n\
    By default, it shows both public and local IP addresses.\n\
    Use the --only flag to display one specific category."
    )]
    Ips {
        #[arg(long)]
        only: Option<network::IpCategory>,
    },

    #[command(name = "dns")]
    #[command(about = "Display your system's DNS servers")]
    #[command(
        long_about = "Show the DNS servers configured on your system, listed in the order they are used."
    )]
    Dns,

    // #[command(arg_required_else_help = true)]
    #[command(name = "date")]
    #[command(about = "Display your system's date")]
    #[command(
        long_about = "Show the current date on your system in a human-readable format.\n\
    Example: Saturday, 8 April, 2023, week 14"
    )]
    Date,

    #[command(name = "time")]
    #[command(about = "Display your system's current time")]
    #[command(
        long_about = "Show the current time on your system, along with the offset from the central NTP\n\
    clock server, in a 24-hour human-readable format.\n
    Example: 20:20:2 UTC +02:00 ±0.0672 seconds"
    )]
    Time,

    #[command(name = "datetime")]
    #[command(about = "Display your system's current date and time")]
    #[command(
        long_about = "Show the current date and time on your system, along with the offset from\n\
    the central NTP clock server, in a human-readable format.\n\
    Example: Saturday, 8 April, 2023, week 14 20:20:2 UTC +02:00 ±0.0684 seconds"
    )]
    Datetime,

    #[command(name = "hostname")]
    #[command(about = "Display your system's hostname")]
    #[command(long_about = "Show the hostname assigned to your system.")]
    Hostname,

    #[command(name = "username")]
    #[command(about = "Display your current system user's username")]
    #[command(long_about = "Show the username of the currently logged-in system user.")]
    Username,

    #[command(name = "device-name")]
    #[command(about = "Display your device's name")]
    #[command(long_about = "Show the configured name of your device.")]
    DeviceName,

    #[command(name = "os")]
    #[command(about = "Display your system's OS name and version")]
    #[command(
        long_about = "Show the name and version of the operating system installed on your system."
    )]
    Os,

    #[command(name = "architecture")]
    #[command(about = "Display your system's CPU architecture")]
    #[command(long_about = "Show the architecture of the CPU installed on your system.")]
    Architecture,

    #[command(name = "interfaces")]
    #[command(about = "Display your system's network interfaces")]
    #[command(
        long_about = "List all the network interfaces configured on your system, presented in the order they are used."
    )]
    Interfaces,

    #[command(name = "disks")]
    #[command(about = "Display your system's disks")]
    #[command(
        long_about = "Lists all the disks installed on your system, providing details such as disk name, type, free space, total capacity, and percentage of free space."
    )]
    Disks,

    #[command(name = "cpu")]
    #[command(about = "Display your system's CPU")]
    #[command(long_about = "Show the name of the CPU installed on your system.")]
    Cpu,

    #[command(name = "ram")]
    #[command(about = "Display your system's RAM")]
    #[command(long_about = "Show the amount of RAM installed and used on your system.")]
    Ram,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Enable human-readable panic messages
    setup_panic!();

    // Parse the CLI arguments
    let cli = Cli::parse();

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

/// `CommandResult` holds the result of a command.
///
/// This is used to facilitate factorizing the command execution,
/// and allow handling the serializing of the result into the desired output format
/// in a single place.
enum CommandResult {
    Ips(Vec<network::Ip>),
    Dns(Vec<String>),
    Date(datetime::Date),
    Time(datetime::Time),
    Datetime(datetime::Datetime),
    Hostname(output::Named),
    Username(output::Named),
    DeviceName(output::Named),
    Os(output::Named),
    Architecture(output::Named),
    Interfaces(Vec<network::Interface>),
    Disks(Vec<storage::DiskInfo>),
    Cpu(system::Cpu),
    Ram(system::Ram),
}

impl Display for CommandResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Ips(ips) => {
                let ips = ips.iter().map(ToString::to_string).collect::<Vec<String>>();
                write!(f, "{}", ips.join("\n"))
            }
            Self::Dns(dns) => {
                write!(f, "{}", dns.join("\n"))
            }
            Self::Date(date) => date.fmt(f),
            Self::Time(time) => time.fmt(f),
            Self::Datetime(datetime) => datetime.fmt(f),
            Self::Hostname(hostname) => hostname.fmt(f),
            Self::Username(username) => username.fmt(f),
            Self::DeviceName(device_name) => device_name.fmt(f),
            Self::Os(os) => os.fmt(f),
            Self::Architecture(architecture) => architecture.fmt(f),
            Self::Interfaces(interfaces) => {
                write!(
                    f,
                    "{}",
                    interfaces
                        .iter()
                        .map(ToString::to_string)
                        .collect::<Vec<String>>()
                        .join("\n")
                )
            }
            Self::Disks(disks) => {
                write!(
                    f,
                    "{}",
                    disks
                        .iter()
                        .map(ToString::to_string)
                        .collect::<Vec<String>>()
                        .join("\n")
                )
            }
            Self::Cpu(cpu) => cpu.fmt(f),
            Self::Ram(ram) => ram.fmt(f),
        }
    }
}

impl Serialize for CommandResult {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::Ips(ips) => ips.serialize(serializer),
            Self::Dns(dns) => dns.serialize(serializer),
            Self::Date(date) => date.serialize(serializer),
            Self::Time(time) => time.serialize(serializer),
            Self::Datetime(datetime) => datetime.serialize(serializer),
            Self::Hostname(hostname) => hostname.serialize(serializer),
            Self::Username(username) => username.serialize(serializer),
            Self::DeviceName(device_name) => device_name.serialize(serializer),
            Self::Os(os) => os.serialize(serializer),
            Self::Architecture(architecture) => architecture.serialize(serializer),
            Self::Interfaces(interfaces) => interfaces.serialize(serializer),
            Self::Disks(disks) => disks.serialize(serializer),
            Self::Cpu(cpu) => cpu.serialize(serializer),
            Self::Ram(ram) => ram.serialize(serializer),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum OutputFormat {
    Json,
    Text,
}

async fn execute_command(command: &Commands) -> Result<CommandResult> {
    match command {
        Commands::Date => Ok(handle_date()),
        Commands::Time => handle_time().await,
        Commands::Datetime => handle_datetime().await,
        Commands::Dns => handle_dns(),
        Commands::Ips { only } => handle_ips(*only).await,
        Commands::Hostname => handle_hostname().await,
        Commands::Username => handle_username().await,
        Commands::DeviceName => handle_device_name().await,
        Commands::Os => handle_os_command().await,
        Commands::Architecture => handle_architecture().await,
        Commands::Interfaces => handle_interfaces().await,
        Commands::Disks => handle_disks(),
        Commands::Cpu => handle_cpu(),
        Commands::Ram => Ok(handle_ram()),
    }
}

async fn fetch_named_command<F, Fut>(
    fetcher: F,
    wrap: fn(output::Named) -> CommandResult,
    context: &'static str,
) -> Result<CommandResult>
where
    F: FnOnce() -> Fut,
    Fut: Future<Output = Result<output::Named>>,
{
    let value = fetcher().await.with_context(|| context)?;
    Ok(wrap(value))
}

fn fetch_sync_command<T, F>(
    fetcher: F,
    wrap: fn(T) -> CommandResult,
    context: &'static str,
) -> Result<CommandResult>
where
    F: FnOnce() -> Result<T>,
{
    let value = fetcher().with_context(|| context)?;
    Ok(wrap(value))
}

fn handle_date() -> CommandResult {
    CommandResult::Date(datetime::date())
}

async fn handle_time() -> Result<CommandResult> {
    let time = datetime::time()
        .await
        .with_context(|| "looking up the system's time failed")?;
    Ok(CommandResult::Time(time))
}

async fn handle_datetime() -> Result<CommandResult> {
    let datetime = datetime::datetime()
        .await
        .with_context(|| "looking up the system's datetime failed")?;
    Ok(CommandResult::Datetime(datetime))
}

fn handle_dns() -> Result<CommandResult> {
    fetch_sync_command(
        network::list_dns_servers,
        CommandResult::Dns,
        "listing the system's dns servers failed",
    )
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
            Ok(CommandResult::Ips(vec![network::Ip {
                category: network::IpCategory::Public,
                address: public_ip,
            }]))
        }
        Some(network::IpCategory::Local) => {
            let local_ip = local_ip_address::local_ip().with_context(
                || "looking up local ip failed; reason: querying local ip address failed",
            )?;
            Ok(CommandResult::Ips(vec![network::Ip {
                category: network::IpCategory::Local,
                address: local_ip,
            }]))
        }
        Some(network::IpCategory::Any) | None => {
            let public_ip = network::query_public_ip(open_dns_host, open_dns_port)
                .await
                .with_context(|| {
                    format!(
                        "listing ips failed; reason: querying dns server {open_dns_host} on port {open_dns_port} failed"
                    )
                })?;

            let local_ip = local_ip_address::local_ip()
                .with_context(|| "listing ips failed; reason: querying local ip address failed")?;

            Ok(CommandResult::Ips(vec![
                network::Ip {
                    category: network::IpCategory::Public,
                    address: public_ip,
                },
                network::Ip {
                    category: network::IpCategory::Local,
                    address: local_ip,
                },
            ]))
        }
    }
}

async fn handle_hostname() -> Result<CommandResult> {
    fetch_named_command(
        system::hostname,
        CommandResult::Hostname,
        "looking up the system's hostname failed",
    )
    .await
}

async fn handle_username() -> Result<CommandResult> {
    fetch_named_command(
        system::username,
        CommandResult::Username,
        "looking up the user's username failed",
    )
    .await
}

async fn handle_device_name() -> Result<CommandResult> {
    fetch_named_command(
        system::device_name,
        CommandResult::DeviceName,
        "looking up the system's device name failed",
    )
    .await
}

async fn handle_os_command() -> Result<CommandResult> {
    fetch_named_command(
        system::os,
        CommandResult::Os,
        "looking up the system's OS name failed",
    )
    .await
}

async fn handle_architecture() -> Result<CommandResult> {
    fetch_named_command(
        system::architecture,
        CommandResult::Architecture,
        "looking up the CPU's architecture failed",
    )
    .await
}

async fn handle_interfaces() -> Result<CommandResult> {
    let interfaces = network::interfaces()
        .await
        .with_context(|| "listing the system's network interfaces failed")?;
    Ok(CommandResult::Interfaces(interfaces))
}

fn handle_disks() -> Result<CommandResult> {
    fetch_sync_command(
        storage::list_disks,
        CommandResult::Disks,
        "listing the disks failed",
    )
}

fn handle_cpu() -> Result<CommandResult> {
    fetch_sync_command(
        system::cpus,
        CommandResult::Cpu,
        "looking up the system's CPU information failed",
    )
}

fn handle_ram() -> CommandResult {
    CommandResult::Ram(system::ram())
}
