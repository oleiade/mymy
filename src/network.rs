use std::fmt::{Display, Formatter};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::time::Duration;
use std::vec;

use anyhow::Result;
use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use sysinfo::{NetworkExt, System, SystemExt};
use tokio::task::spawn_blocking;
use trust_dns_resolver::config::{NameServerConfig, Protocol, ResolverConfig, ResolverOpts};
use trust_dns_resolver::{system_conf, AsyncResolver, TokioAsyncResolver, TokioHandle};

use crate::format::human_readable_duration;

#[derive(Serialize)]
pub struct IpReport {
    #[serde(skip_serializing_if = "Option::is_none")]
    public: Option<IpAddr>,

    #[serde(skip_serializing_if = "Option::is_none")]
    local: Option<IpAddr>,
}

impl Display for IpReport {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if let Some(public) = &self.public {
            write!(f, "public\t{}", public)?;
        }

        if let Some(local) = &self.local {
            write!(f, "local\t{}", local)?;
        }

        Ok(())
    }
}

/// A categorized IP address.
#[derive(Serialize, Deserialize, Debug)]
pub struct Ip {
    /// The IP address.
    #[serde(rename(serialize = "ip", deserialize = "ip"))]
    pub address: IpAddr,

    /// The category of the IP address.
    pub category: IpCategory,
}

impl Display for Ip {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{}\t{}", self.category, self.address)
    }
}

/// Queries the public IP address from the provided dns server.
/// Only an IPv4 address is returned.
///
/// # Arguments
///
/// * `dns_server_host` - The DNS server host to query the public IP address from.
/// * `dns_server_port` - The DNS server port to query the public IP address from.
///
/// # Returns
///
/// The public IP address.
///
/// # Errors
///
/// If the DNS server host cannot be parsed, or if the DNS server cannot be queried.
///
/// # Examples
///
/// ```
/// use std::net::IpAddr;
///
/// let public_ip = ip::query_public_ip(ip::OPENDNS_SERVER_HOST, 53).unwrap();
/// println!("public ip: {}", public_ip);
/// ```
pub async fn query_public_ip(dns_server_host: &str, dns_server_port: u16) -> Result<IpAddr> {
    // Set up the resolver configuration
    let dns_server_addr = SocketAddr::new(dns_server_host.parse()?, dns_server_port);
    let nameserver_config = NameServerConfig::new(dns_server_addr, Protocol::Udp);
    let resolver_config = ResolverConfig::from_parts(None, vec![], vec![nameserver_config]);

    let mut resolver_opts = ResolverOpts::default();
    resolver_opts.ndots = 1;
    resolver_opts.timeout = std::time::Duration::from_secs(5);

    // Create the resolver
    let resolver = TokioAsyncResolver::new(resolver_config, resolver_opts, TokioHandle)?;

    // Query the public IP address from the OpenDNS server
    let ipv4_response = resolver.ipv4_lookup("myip.opendns.com").await?;

    let ipv4: &Ipv4Addr = ipv4_response.iter().next().unwrap();

    Ok(IpAddr::V4(*ipv4))
}

/// The default DNS server port.
///
/// This constant is used as a default to query the public IP address
pub const DNS_DEFAULT_PORT: u16 = 53;

/// The openDNS server host.
///
/// This constant is used as a default to query the public IP address
pub const OPENDNS_SERVER_HOST: &str = "208.67.222.222";

/// Lists the DNS servers from the system configuration.
///
/// The DNS servers are returned as a list of IP addresses.
/// The DNS servers are deduplicated.
/// The DNS servers are returned in the order they are defined in the system configuration.
///
/// # Returns
///
/// The DNS servers:
///   * The DNS servers are returned as a list of IP addresses.
///   * The DNS servers are deduplicated.
///   * The DNS servers are returned in the order they are defined in the system configuration.
///
/// # Errors
///
/// If the system configuration cannot be read.
///
/// # Examples
///
/// ```
/// let dns_servers = ip::list_dns_servers().unwrap();
/// println!("dns servers: {:?}", dns_servers);
/// ```
pub async fn list_dns_servers() -> Result<Vec<String>> {
    let (conf, _) = system_conf::read_system_conf()?;
    let mut nameservers = conf
        .name_servers()
        .iter()
        .map(|ns| {
            ns.socket_addr
                .to_string()
                .split(':')
                .next()
                .unwrap()
                .to_owned()
        })
        .collect::<Vec<_>>();

    nameservers.dedup();

    Ok(nameservers)
}

/// Holds the category of an IP address. The category can be public, local or any.
#[derive(Copy, Clone, Debug, Deserialize, PartialEq, Eq, PartialOrd, Ord, Serialize, ValueEnum)]
pub enum IpCategory {
    #[clap(name = "public")]
    Public,

    #[clap(name = "local")]
    Local,

    #[clap(name = "any")]
    Any,
}

impl Display for IpCategory {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            IpCategory::Public => write!(f, "public"),
            IpCategory::Local => write!(f, "local"),
            IpCategory::Any => write!(f, "*"),
        }
    }
}

/// Lists the network interfaces of the system.
///
/// # Returns
///
/// A vector holding the network interfaces.
/// The network interfaces are returned in the order they are defined in the system configuration.
///
/// # Errors
///
/// If the system configuration cannot be read.
///
/// # Examples
///
/// ```
/// let interfaces = ip::list_interfaces().unwrap();
/// println!("interfaces: {:?}", interfaces);
/// ```
pub async fn interfaces() -> Result<Vec<Interface>> {
    spawn_blocking(get_if_addrs::get_if_addrs)
        .await??
        .into_iter()
        .try_fold(Vec::new(), |mut acc, i| {
            acc.push(Interface {
                name: i.name.clone(),
                ip: i.ip().to_string(),
            });
            Ok(acc)
        })
}

/// A network interface.
#[derive(Serialize)]
pub struct Interface {
    /// The name of the network interface.
    name: String,

    /// The IP address of the network interface.
    ip: String,
}

impl Display for Interface {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}\t{}", self.name, self.ip)
    }
}

/// Lists those interfaces that have an associated mac address.
///
/// # Returns
///
/// A vector holding the mac address and the name of the intreface.
///
/// # Errors
///
/// If the system configuration cannot be read.
///
/// # Examples
///
/// ```
/// let mac_addresses = ip::_mac_addresses().unwrap();
/// for e in mac_addresses {
///     println!("Interface: {}, mac address: {}", e.name, e.address);
/// }
/// ```
pub async fn mac_addresses() -> Result<Vec<MacAddress>> {
    let mut system_info = System::new();

    // Get only the network information of the system
    system_info.refresh_networks_list();
    let networks = system_info.networks();
    let mut addresses: Vec<MacAddress> = Vec::new();

    for (interface_name, network) in networks {
        let mac_address = network.mac_address();

        // Some interfaces could not have a mac address
        if !mac_address.is_unspecified() {
            let mut pretty_address = String::with_capacity(17);
            for (pos, e) in mac_address.0.to_vec().iter().enumerate() {
                if pos != 0 {
                    pretty_address.push(':');
                }
                // Format the number as HEX pairs
                pretty_address.push_str(&format!("{:02X}", e));
            }
            addresses.push(MacAddress {
                name: interface_name.to_string(),
                address: pretty_address.to_owned(),
            });
        }
    }
    Ok(addresses)
}

/// A Mac Address
#[derive(Serialize, Deserialize, Debug)]
pub struct MacAddress {
    /// The  network interface name
    #[serde(rename(deserialize = "interface_name", serialize = "interface_name"))]
    name: String,
    /// The Mac address
    #[serde(rename(deserialize = "mac_address", serialize = "mac_address"))]
    address: String,
}

impl Display for MacAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}\t{}", self.name, self.address)
    }
}

pub async fn resolve_domain(domain: &str) -> Option<IpAddr> {
    let resolver = AsyncResolver::tokio_from_system_conf().expect("failed to create resolver");
    match resolver.lookup_ip(domain).await {
        Ok(lookup) => lookup.iter().next(),
        Err(_) => None,
    }
}

pub async fn ping_once(target: IpAddr, timeout: Duration) -> Result<Ping> {
    use surge_ping::ping;
    let payload = [0; 8];
    let (_, duration) = ping(target, &payload).await?;

    if duration > timeout {
        return Err(anyhow::anyhow!("pinging {} timed out", target));
    }

    Ok(Ping { target, duration })
}

// async fn median_latency(target: IpAddr, interval: Duration, timeout: Duration) -> Option<f64> {
//     let mut samples = Vec::new();

//     loop {
//         if let Ok(ping) = ping_once(target, timeout).await {
//             samples.push(ping.duration);
//             if samples.len() >= 10 {
//                 break;
//             }
//         }

//         tokio::time::sleep(interval).await;
//     }

//     if samples.is_empty() {
//         None
//     } else {
//         samples.sort_unstable();

//         let mid = samples.len() / 2;
//         if samples.len() % 2 == 0 {
//             Some((samples[mid - 1] + samples[mid]).as_secs_f64() / 2.0)
//         } else {
//             Some(samples[mid].as_secs_f64())
//         }
//     }
// }

#[derive(Serialize)]
pub struct Ping {
    target: IpAddr,
    duration: Duration,
}

impl Display for Ping {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}\t{}",
            self.target,
            human_readable_duration(self.duration)
        )
    }
}
