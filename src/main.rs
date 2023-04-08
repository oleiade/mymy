use std::fmt::Display;

use anyhow::Result;
use clap::{Parser, Subcommand, ValueEnum};
use serde::{Serialize, Serializer};

mod country;
mod datetime;
mod network;
mod output;
mod system;

#[derive(Parser, Debug)]
#[command(name = "my")]
#[command(about = "Find out common information about your system", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(short, long, value_enum, default_value_t = OutputFormat::Text)]
    output: OutputFormat,
}

#[derive(Subcommand, Debug)]
enum Commands {
    #[command(name = "ips", about = "Find out your IP addresses", long_about = None)]
    Ips {
        #[arg(long)]
        only: Option<network::IpCategory>,
    },

    #[command(name = "dns", about = "Find out your DNS servers", long_about = None)]
    Dns,

    #[command(name = "date", about = "Find out the current date", long_about = None)]
    Date,

    #[command(name = "time", about = "Find out the current time", long_about = None)]
    Time,

    #[command(name = "datetime", about = "Find out the current date and time", long_about = None)]
    Datetime,

    #[command(name = "hostname", about = "Find out your hostname", long_about = None)]
    Hostname,

    #[command(name = "username", about = "Find out your country", long_about = None)]
    Username,

    #[command(name = "device-name", about = "Find out the name of your device", long_about = None)]
    DeviceName,

    #[command(name = "os", about = "Find out your OS name and version", long_about = None)]
    Os,

    #[command(name = "architecture", about = "Find out your CPU architecture", long_about = None)]
    Architecture,

    #[command(name = "interfaces", about = "Find out your network interfaces", long_about = None)]
    Interfaces,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse the CLI arguments
    let args = Cli::parse();

    // Execute the appropriate command
    let result: CommandResult = match args.command {
        Commands::Date => CommandResult::Date(datetime::date().await?),
        Commands::Time => CommandResult::Time(datetime::time().await?),
        Commands::Datetime => CommandResult::Datetime(datetime::datetime().await?),
        Commands::Dns => CommandResult::Dns(network::list_dns_servers().await?),
        Commands::Ips { only } => match only {
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
            }
            Some(network::IpCategory::Local) => {
                let local_ip = local_ip_address::local_ip().unwrap();
                CommandResult::Ips(vec![network::Ip {
                    category: network::IpCategory::Local,
                    address: local_ip,
                }])
            }
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

    match args.output {
        OutputFormat::Json => {
            let json_repr = serde_json::to_string_pretty(&result)?;
            println!("{}", json_repr);
        }
        OutputFormat::Text => {
            println!("{}", result);
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
