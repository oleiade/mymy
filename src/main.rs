use std::fmt::Display;

use anyhow::Result;
use clap::{Parser, Subcommand, ValueEnum};
use serde::{Serialize, Serializer};

mod country;
mod datetime;
mod network;
mod output;
mod system;


#[derive(Debug, Parser)]
#[command(name = "my")]
#[command(about = "Get essential information about your device")]
#[command(long_about = "Easily access important details about your device, such as IP addresses, DNS servers, date, time, and more.")]
pub struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    #[arg(short, long, value_enum, default_value_t = OutputFormat::Text)]
    output: OutputFormat,
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
}


#[tokio::main]
async fn main() -> Result<()> {
    // Parse the CLI arguments
    let cli = Cli::parse();

    // Execute the appropriate command
    if let Some(command) = &cli.command {
        let result: CommandResult = match command {
            Commands::Date => CommandResult::Date(datetime::date().await?),
            Commands::Time => CommandResult::Time(datetime::time().await?),
            Commands::Datetime => CommandResult::Datetime(datetime::datetime().await?),
            Commands::Dns => CommandResult::Dns(network::list_dns_servers().await?),
            Commands::Ips{ only } => match only {
                Some(network::IpCategory::Public) => {
                    let public_ip = network::query_public_ip(
                        network::OPENDNS_SERVER_HOST,
                        network::DNS_DEFAULT_PORT,
                    )
                    .await?;
                    CommandResult::Ips(vec![network::Ip {
                        category: network::IpCategory::Public,
                        address: public_ip,
                    }])
                },
                Some(network::IpCategory::Local) => {
                    let local_ip = local_ip_address::local_ip().unwrap();
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
                    .await?;
                    let local_ip = local_ip_address::local_ip().unwrap();

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
            Commands::Hostname => CommandResult::Hostname(system::hostname().await?),
            Commands::Username => CommandResult::Username(system::username().await?),
            Commands::DeviceName => CommandResult::DeviceName(system::device_name().await?),
            Commands::Os => CommandResult::Os(system::os().await?),
            Commands::Architecture => CommandResult::Architecture(system::architecture().await?),
            Commands::Interfaces => CommandResult::Interfaces(network::interfaces().await?),
        };

        match cli.output {
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
                        .collect::<String>()
                )
            }
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
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum OutputFormat {
    Json,
    Text,
}
