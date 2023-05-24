use std::{fmt::Display, net::IpAddr, time::Duration};

use anyhow::{Result, Context};
use clap::{Parser, Subcommand, ValueEnum};
use human_panic::setup_panic;
use serde::{Serialize, Serializer};

mod country;
mod datetime;
mod format;
mod network;
mod output;
mod parsers;
mod storage;
mod system;


#[derive(Debug, Parser)]
#[command(name = "my")]
#[command(about = "Get essential information about your device")]
#[command(long_about = "Easily access important details about your device, such as IP addresses, DNS servers, date, time, and more.")]
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
    #[command(long_about = "Find all IP addresses associated with your system, both local and external.\n\
    By default, it shows both public and local IP addresses.\n\
    Use the --only flag to display one specific category.")]
    Ips {
        #[arg(long)]
        only: Option<network::IpCategory>,
    },

    #[command(name = "dns")]
    #[command(about = "Display your system's DNS servers")]
    #[command(long_about = "Show the DNS servers configured on your system, listed in the order they are used.")]
    Dns,

    // #[command(arg_required_else_help = true)]
    #[command(name = "date")]
    #[command(about = "Display your system's date")]
    #[command(long_about = "Show the current date on your system in a human-readable format.\n\
    Example: Saturday, 8 April, 2023, week 14")]
    Date,

    #[command(name = "time")]
    #[command(about = "Display your system's current time")]
    #[command(long_about = "Show the current time on your system, along with the offset from the central NTP\n\
    clock server, in a 24-hour human-readable format.\n
    Example: 20:20:2 UTC +02:00 ±0.0672 seconds")]
    Time,

    #[command(name = "datetime")]
    #[command(about = "Display your system's current date and time")]
    #[command(long_about = "Show the current date and time on your system, along with the offset from\n\
    the central NTP clock server, in a human-readable format.\n\
    Example: Saturday, 8 April, 2023, week 14 20:20:2 UTC +02:00 ±0.0684 seconds")]
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
    #[command(long_about = "Show the name and version of the operating system installed on your system.")]
    Os,

    #[command(name = "architecture")]
    #[command(about = "Display your system's CPU architecture")]
    #[command(long_about = "Show the architecture of the CPU installed on your system.")]
    Architecture,

    #[command(name = "interfaces")]
    #[command(about = "Display your system's network interfaces")]
    #[command(long_about = "List all the network interfaces configured on your system, presented in the order they are used.")]
    Interfaces,

    #[command(name = "disks")]
    #[command(about = "Display your system's disks")]
    #[command(long_about = "Lists all the disks installed on your system, providing details such as disk name, type, free space, total capacity, and percentage of free space.")]
    Disks,

    #[command(name = "cpu")]
    #[command(about = "Display your system's CPU")]
    #[command(long_about = "Show the name of the CPU installed on your system.")]
    Cpu,

    #[command(name = "ram")]
    #[command(about = "Display your system's RAM")]
    #[command(long_about = "Show the amount of RAM installed and used on your system.")]
    Ram,

    #[command(name = "latency")]
    #[command(about = "latency to a remote host")]
    #[command(long_about = "Measure the latency to a remote host and display the results.")]
    Latency {
        host: String,

        #[arg(long, default_value = "5")]
        timeout: String,
    },
}


#[tokio::main]
async fn main() -> Result<()> {
    // Enable human-readable panic messages
    setup_panic!();

    // Parse the CLI arguments
    let cli = Cli::parse();

    // Execute the appropriate command
    if let Some(command) = &cli.command {
        let result: CommandResult = match command {
            Commands::Date => CommandResult::Date(
                datetime::date().await
                    .with_context(|| "looking up the system's date failed")?
            ),
            Commands::Time => CommandResult::Time(
                datetime::time().await
                    .with_context(|| "looking up the system's time failed")?
            ),
            Commands::Datetime => CommandResult::Datetime(
                datetime::datetime().await
                    .with_context(|| "looking up the system's datetime failed")?
            ),
            Commands::Dns => CommandResult::Dns(
                network::list_dns_servers().await
                    .with_context(|| "listing the system's dns servers failed")?
            ),
            Commands::Ips{ only } => match only {
                Some(network::IpCategory::Public) => {
                    let public_ip = network::query_public_ip(
                        network::OPENDNS_SERVER_HOST,
                        network::DNS_DEFAULT_PORT,
                    )
                    .await
                    .with_context(|| {
                        format!(
                            "looking up public ip failed; reason: querying dns server {} on port {} failed",
                            network::OPENDNS_SERVER_HOST,
                            network::DNS_DEFAULT_PORT
                        )
                    })?;
                    CommandResult::Ips(vec![network::Ip {
                        category: network::IpCategory::Public,
                        address: public_ip,
                    }])
                },
                Some(network::IpCategory::Local) => {
                    let local_ip = local_ip_address::local_ip()
                        .with_context(|| "looking up local ip failed; reason: querying local ip address failed")?;

                    CommandResult::Ips(vec![network::Ip {
                        category: network::IpCategory::Local,
                        address: local_ip,
                    }])
                },
                Some(network::IpCategory::Any) | None => {
                    let public_ip = network::query_public_ip(
                        network::OPENDNS_SERVER_HOST,
                        network::DNS_DEFAULT_PORT,
                    )
                    .await
                    .with_context(|| {
                        format!(
                            "listing ips failed; reason: querying dns server {} on port {} failed",
                            network::OPENDNS_SERVER_HOST,
                            network::DNS_DEFAULT_PORT
                        )
                    })?;

                    let local_ip = local_ip_address::local_ip()
                        .with_context(|| "listing ips failed; reason: querying local ip address failed")?;

                    CommandResult::Ips(vec![
                        network::Ip {
                            category: network::IpCategory::Public,
                            address: public_ip,
                        },
                        network::Ip {
                            category: network::IpCategory::Local,
                            address: local_ip,
                        },
                    ])
                }
            },
            Commands::Hostname => CommandResult::Hostname(
                system::hostname().await
                    .with_context(|| "looking up the system's hostname failed")?
            ),
            Commands::Username => CommandResult::Username(
                system::username().await
                    .with_context(|| "looking up the user's username failed")?
            ),
            Commands::DeviceName => CommandResult::DeviceName(
                system::device_name().await
                    .with_context(|| "looking up the systems' device name failed")?
            ),
            Commands::Os => CommandResult::Os(
                system::os().await
                    .with_context(|| "looking up the system's OS name failed")?
            ),
            Commands::Architecture => CommandResult::Architecture(
                system::architecture().await
                    .with_context(|| "looking up the CPU's architecture fialed")?
            ),
            Commands::Interfaces => CommandResult::Interfaces(
                network::interfaces().await
                    .with_context(|| "listing the system's network interfaces failed")?
            ),
            Commands::Disks => CommandResult::Disks(
                storage::list_disks().await
                    .with_context(|| "listing the disks failed")?
            ),
            Commands::Cpu => CommandResult::Cpu(
                system::cpus().await
                    .with_context(|| "looking up the system's CPU information failed")?),
            Commands::Ram => CommandResult::Ram(
                system::ram().await
                    .with_context(|| "looking up the system's RAM information failed")?
            ),
            Commands::Latency { host, timeout } => {
                let timeout_duration = parsers::parse_duration(timeout)
                    .with_context(|| "parsing timeout expression failed")?;

                println!("{:?}", timeout_duration);

                let target = match host.parse::<IpAddr>() {
                    Ok(ip) => ip,
                    Err(_) => network::resolve_domain(host).await.unwrap_or_else(|| {
                        eprintln!("Failed to resolve domain name '{}'", host);
                        std::process::exit(1);
                    })
                };

                let ping = network::ping_once(target, timeout_duration).await?;

                CommandResult::Ping(ping)
            },
        };

        match cli.format {
            OutputFormat::Json => {
                let json_repr = serde_json::to_string_pretty(&result)?;
                println!("{}", json_repr);
            }
            OutputFormat::Text => {
                println!("{}", result);
            }
        }
    }

    Ok(())
}

/// CommandResult holds the result of a command.
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
    Ping(network::Ping),
}

impl Display for CommandResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CommandResult::Ips(ips) => {
                let ips = ips.iter().map(ToString::to_string).collect::<Vec<String>>();
                write!(f, "{}", ips.join("\n"))
            }
            CommandResult::Dns(dns) => {
                write!(f, "{}", dns.join("\n"))
            }
            CommandResult::Date(date) => date.fmt(f),
            CommandResult::Time(time) => time.fmt(f),
            CommandResult::Datetime(datetime) => datetime.fmt(f),
            CommandResult::Hostname(hostname) => hostname.fmt(f),
            CommandResult::Username(username) => username.fmt(f),
            CommandResult::DeviceName(device_name) => device_name.fmt(f),
            CommandResult::Os(os) => os.fmt(f),
            CommandResult::Architecture(architecture) => architecture.fmt(f),
            CommandResult::Interfaces(interfaces) => {
                write!(
                    f,
                    "{}",
                    interfaces
                        .iter()
                        .map(ToString::to_string)
                        .collect::<Vec<String>>()
                        .join("\n")
                )
            },
            CommandResult::Disks(disks) => {
                write!(
                    f,
                    "{}",
                    disks
                        .iter()
                        .map(ToString::to_string)
                        .collect::<Vec<String>>()
                        .join("\n")
                )
            },
            CommandResult::Cpu(cpu) => cpu.fmt(f),
            CommandResult::Ram(ram) => ram.fmt(f),
            CommandResult::Ping(ping) => ping.fmt(f),
        }
    }
}

impl Serialize for CommandResult {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            CommandResult::Ips(ips) => ips.serialize(serializer),
            CommandResult::Dns(dns) => dns.serialize(serializer),
            CommandResult::Date(date) => date.serialize(serializer),
            CommandResult::Time(time) => time.serialize(serializer),
            CommandResult::Datetime(datetime) => datetime.serialize(serializer),
            CommandResult::Hostname(hostname) => hostname.serialize(serializer),
            CommandResult::Username(username) => username.serialize(serializer),
            CommandResult::DeviceName(device_name) => device_name.serialize(serializer),
            CommandResult::Os(os) => os.serialize(serializer),
            CommandResult::Architecture(architecture) => architecture.serialize(serializer),
            CommandResult::Interfaces(interfaces) => interfaces.serialize(serializer),
            CommandResult::Disks(disks) => disks.serialize(serializer),
            CommandResult::Cpu(cpu) => cpu.serialize(serializer),
            CommandResult::Ram(ram) => ram.serialize(serializer),
            CommandResult::Ping(ping) => { ping.serialize(serializer) },
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum OutputFormat {
    Json,
    Text,
}