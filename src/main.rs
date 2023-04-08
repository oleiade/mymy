use anyhow::Result;
use clap::{Parser, Subcommand, ValueEnum};


mod country;
mod datetime;
mod output;
mod network;
mod system;

#[derive(Parser, Debug)]
#[command(name = "my")]
#[command(about = "Find out about your setup", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(short, long, value_enum, default_value_t = OutputFormat::Text)]
    output: OutputFormat,
}

#[derive(Subcommand, Debug)]
enum Commands {
    #[command(name = "ips", about = "Find out your IP addresses", long_about = None)]
    Ips{
        #[arg(long)]
        only: Option<IpSelection>,
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

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum IpSelection {
    #[clap(name = "public")]
    Public,

    #[clap(name = "local")]
    Local,

    #[clap(name = "all")]
    All,
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse the CLI arguments
    let args = Cli::parse();


    // Execute the appropriate command
    match args.command {
        Commands::Date => {
            let date = datetime::date().await?;

            match args.output {
                OutputFormat::Json => {
                    let json_repr = serde_json::to_string_pretty(&date)?;
                    println!("{}", json_repr);
                },
                OutputFormat::Text => {
                    println!("{}", date);
                },
            }
        },
        Commands::Time => {
            let time = datetime::time().await?;

            match args.output {
                OutputFormat::Json => {
                    let json_repr = serde_json::to_string_pretty(&time)?;
                    println!("{}", json_repr);
                },
                OutputFormat::Text => {
                    println!("{}", time);
                },
            }
        },
        Commands::Datetime => {
            let dt = datetime::datetime().await?;

            match args.output {
                OutputFormat::Json => {
                    let json_repr = serde_json::to_string_pretty(&dt)?;
                    print!("{}", json_repr);
                },
                OutputFormat::Text => {
                    print!("{}", dt);
                },
            }
        },
        Commands::Dns => {
            let nameservers = network::list_dns_servers().await?;

            match args.output {
                OutputFormat::Json => {
                    let json_repr = serde_json::to_string_pretty(&nameservers)?;
                    println!("{}", json_repr)
                },
                OutputFormat::Text => {
                    nameservers.iter().for_each(|ns| println!("{}", ns))
                },
            }
        },
        Commands::Ips { only } => {
            ips(args.output).await?
        }
        Commands::Hostname => {
            let hostname = system::hostname().await?;

            match args.output {
                OutputFormat::Json => {
                    let json_repr = serde_json::to_string_pretty(&hostname)?;
                    println!("{}", json_repr);
                },
                OutputFormat::Text => {
                    println!("{}", hostname);
                },
            }
        },
        Commands::Username => {
            let username = system::username().await?;

            match args.output {
                OutputFormat::Json => {
                    let json_repr = serde_json::to_string_pretty(&username)?;
                    println!("{}", json_repr);
                },
                OutputFormat::Text => {
                    println!("{}", username);
                },
            }
        },
        Commands::DeviceName => {
            let devicename = system::device_name().await?;

            match args.output {
                OutputFormat::Json => {
                    let json_repr = serde_json::to_string_pretty(&devicename)?;
                    println!("{}", json_repr);
                },
                OutputFormat::Text => {
                    println!("{}", devicename);
                },
            }
        },
        Commands::Os => {
            let os = system::os().await?;

            match args.output {
                OutputFormat::Json => {
                    let json_repr = serde_json::to_string_pretty(&os)?;
                    println!("{}", json_repr);
                },
                OutputFormat::Text => {
                    println!("{}", os);
                },
            }
        },
        Commands::Architecture => {
            let arch = system::architecture().await?;

            match args.output {
                OutputFormat::Json => {
                    let json_repr = serde_json::to_string_pretty(&arch)?;
                    println!("{}", json_repr);
                },
                OutputFormat::Text => {
                    println!("{}", arch);
                },
            }
        },
        Commands::Interfaces => {
            let interfaces = network::interfaces().await?;
            match args.output {
                OutputFormat::Json => {
                    let json_repr = serde_json::to_string_pretty(&interfaces)?;
                    println!("{}", json_repr);
                },
                OutputFormat::Text => {
                    interfaces.iter().for_each(|interface| println!("{}", interface));
                },
            }
        },
    };

    Ok(())
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum OutputFormat {
    Json,
    Text,
}



async fn ips(output: OutputFormat) -> Result<()> {
    public_ip(output).await?;
    local_ip(output).await?;

    Ok(())
}

async fn public_ip(output: OutputFormat) -> Result<()> {
    let public_ip = network::query_public_ip(network::OPENDNS_SERVER_HOST, 53).await?;

    match output {
        OutputFormat::Json => {
            let json_repr = serde_json::to_string_pretty(&public_ip)?;
            Ok(println!("{}", json_repr))
        },
        OutputFormat::Text => {
            Ok(println!("{}", public_ip))
        }
    }
}

async fn local_ip(output: OutputFormat) -> Result<()> {
    let ip = local_ip_address::local_ip().unwrap();

    match output {
        OutputFormat::Json => {
            let json_repr = serde_json::to_string_pretty(&ip)?;
            Ok(println!("{}", json_repr))
        },
        OutputFormat::Text => {
            Ok(println!("{}", ip))
        }
    }
}
