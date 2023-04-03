use anyhow::Result;
use chrono::Local;
use clap::{Parser, Subcommand, ValueEnum};
use rsntp::AsyncSntpClient;
use trust_dns_resolver::system_conf;


mod country;
mod ip;

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
    #[command(name = "ip", about = "Find out your IP address", long_about = None)]
    Ip,

    #[command(name = "dns", about = "Find out your DNS servers", long_about = None)]
    Dns,

    #[command(name = "date", about = "Find out the current date", long_about = None)]
    Date,

    #[command(name = "time", about = "Find out the current time", long_about = None)]
    Time,

    #[command(name = "datetime", about = "Find out the current date and time", long_about = None)]
    Datetime,
}

#[tokio::main]
// async fn main() -> Result<(), Box<dyn std::error::Error>> {
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();

    match args.command {
        Commands::Date => {
            let dt = Local::now();
            let now_with_tz = dt.with_timezone(&Local);
            println!("{}", now_with_tz.format("%A, %d %B, %Y, week %U"));
        }
        Commands::Time => {
            let sntp_client = AsyncSntpClient::new();
            let sntp_time = sntp_client.synchronize("pool.ntp.org").await?;
            let now = sntp_time.datetime().into_chrono_datetime()?;
            let now_with_tz = now.with_timezone(&Local);

            println!("{}", now_with_tz.format("%H:%M:%S %Z"));
            println!("±{:.4} seconds", sntp_time.clock_offset().as_secs_f64());
        },
        Commands::Datetime => {
            let sntp_client = AsyncSntpClient::new();
            let sntp_time = sntp_client.synchronize("pool.ntp.org").await?;
            let now = sntp_time.datetime().into_chrono_datetime()?;
            let now_with_tz = now.with_timezone(&Local);

            println!("{}", now_with_tz.format("%H:%M:%S %Z, %A, %d %B, %Y, week %U"));
            println!("±{:.4} seconds", sntp_time.clock_offset().as_secs_f64());
        }
        Commands::Dns => dns(args.output).await?,
        Commands::Ip => ip(args.output).await?,
    }

    Ok(())
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum OutputFormat {
    Json,
    Text,
}

async fn dns(output: OutputFormat) -> Result<()> {
    let (conf, _) = system_conf::read_system_conf()?;
    let mut name_servers = conf
        .name_servers()
        .iter()
        .map(|ns| {
            ns.socket_addr
                .to_string()
                .splitn(2, ':')
                .next()
                .unwrap()
                .to_owned()
        })
        .collect::<Vec<_>>();

    name_servers.dedup();

    match output {
        OutputFormat::Json => {
            let json_repr = serde_json::to_string_pretty(&name_servers)?;
            Ok(println!("{}", json_repr))
        },
        OutputFormat::Text => {
            Ok(name_servers.iter().for_each(|ns| println!("{}", ns)))
        },
    }
}

async fn ip(output: OutputFormat) -> Result<()> {
    let public_ip = ip::query_public_ip(ip::OPENDNS_SERVER_HOST, 53).await?;
    
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