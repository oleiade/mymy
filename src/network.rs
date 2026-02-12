use std::fmt::{Display, Formatter};
use std::net::{IpAddr, SocketAddr};

use anyhow::{Context, Result, anyhow};
use clap::ValueEnum;
use hickory_proto::xfer::Protocol;
use hickory_resolver::config::{NameServerConfig, ResolverConfig, ResolverOpts};
use hickory_resolver::name_server::TokioConnectionProvider;
use hickory_resolver::{Resolver, system_conf};
use itertools::Itertools;
use local_ip_address::list_afinet_netifas;
use serde::{Deserialize, Serialize};

/// A categorized IP address.
#[derive(Serialize, Deserialize, Debug)]
pub struct Ip {
    /// The IP address.
    #[serde(rename = "ip")]
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
/// ```no_run
/// # use anyhow::Result;
/// # async fn example() -> Result<()> {
/// use mymy::network;
///
/// let public_ip = network::query_public_ip(network::OPENDNS_SERVER_HOST, 53).await?;
/// println!("public ip: {}", public_ip);
/// # Ok(())
/// # }
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
    let resolver =
        Resolver::builder_with_config(resolver_config, TokioConnectionProvider::default())
            .with_options(resolver_opts)
            .build();

    // Query the public IP address from the OpenDNS server
    let ipv4_response = resolver.ipv4_lookup("myip.opendns.com").await?;

    let ipv4 = ipv4_response
        .iter()
        .next()
        .context("public IP lookup returned no IPv4 records")?;

    Ok(IpAddr::V4(**ipv4))
}

/// The default DNS server port.
///
/// This constant is used as a default to query the public IP address
pub const DNS_DEFAULT_PORT: u16 = 53;

/// The openDNS server host.
///
/// This constant is used as a default to query the public IP address
pub const OPENDNS_SERVER_HOST: &str = "208.67.222.222";

/// A DNS server with its address and ordinal position.
#[derive(Serialize)]
pub struct DnsServer {
    /// The IP address of the DNS server.
    pub address: IpAddr,

    /// The 1-indexed ordinal position of the DNS server.
    pub order: usize,
}

impl Display for DnsServer {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "server {}\t{}", self.order, self.address)
    }
}

/// Lists the DNS servers from the system configuration.
///
/// The DNS servers are returned as a list of `DnsServer` structs.
/// The DNS servers are deduplicated.
/// The DNS servers are returned in the order they are defined in the system configuration.
///
/// # Returns
///
/// The DNS servers:
///   * The DNS servers are returned as a list of `DnsServer` structs.
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
pub fn list_dns_servers() -> Result<Vec<DnsServer>> {
    let (conf, _) = system_conf::read_system_conf()?;
    let nameservers = conf
        .name_servers()
        .iter()
        .map(|ns| ns.socket_addr.ip())
        .unique()
        .enumerate()
        .map(|(i, address)| DnsServer {
            address,
            order: i + 1,
        })
        .collect::<Vec<_>>();

    Ok(nameservers)
}

/// Holds the category of an IP address. The category can be public, local or any.
#[derive(Copy, Clone, Debug, Deserialize, PartialEq, Eq, PartialOrd, Ord, Serialize, ValueEnum)]
#[serde(rename_all = "snake_case")]
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
            Self::Public => write!(f, "public"),
            Self::Local => write!(f, "local"),
            Self::Any => write!(f, "*"),
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
pub fn interfaces() -> Result<Vec<Interface>> {
    let netifs = list_afinet_netifas().map_err(|e| anyhow!(e))?;

    Ok(netifs
        .into_iter()
        .map(|(name, ip)| Interface {
            name,
            ip,
        })
        .collect())
}

/// A network interface.
#[derive(Serialize)]
pub struct Interface {
    /// The name of the network interface.
    pub(crate) name: String,

    /// The IP address of the network interface.
    pub(crate) ip: IpAddr,
}

impl Display for Interface {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}\t{}", self.name, self.ip)
    }
}

/// Whether an interface entry should be shown in the default (filtered) view.
///
/// Hides loopback and link-local addresses. Interfaces that only carry
/// such addresses effectively disappear from the output.
pub const fn is_default_visible(iface: &Interface) -> bool {
    if iface.ip.is_loopback() {
        return false;
    }

    match iface.ip {
        IpAddr::V4(v4) => !v4.is_link_local(),
        IpAddr::V6(v6) => !v6.is_unicast_link_local(),
    }
}
