use std::fmt;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use anyhow::Result;
use serde::{Deserialize, Serialize};
use trust_dns_resolver::config::{NameServerConfig, Protocol, ResolverConfig, ResolverOpts};
use trust_dns_resolver::{TokioAsyncResolver, TokioHandle};

/// The openDNS server host.
/// 
/// This constant is used as a default to query the public IP address
pub const OPENDNS_SERVER_HOST: &str = "208.67.222.222";

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

/// And IP address and its geographical location.
#[derive(Serialize, Deserialize, Debug)]
pub struct Ip {
    /// The IP address.
    #[serde(rename(serialize = "ip", deserialize = "ip"))]
    pub address: IpAddr,

    /// The country name.
    pub country: String,

    /// The country code in [ISO 3166-1 alpha-2 format](https://en.wikipedia.org/wiki/ISO_3166-1_alpha-2).
    #[serde(rename(deserialize = "cc"))]
    pub country_code: String,
}

impl fmt::Display for Ip {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ip: {}\tcountry: {}\tcountry code: {}", self.address, self.country, self.country_code)
    }
}

