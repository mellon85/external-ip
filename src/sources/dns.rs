use crate::sources::interfaces::{Error, Family, IpFuture, IpResult, Source};
use log::trace;
use std::net::IpAddr;
use std::net::SocketAddr;

use trust_dns_resolver::config::*;
use trust_dns_resolver::TokioAsyncResolver;

#[derive(Debug, Clone)]
pub enum QueryType {
    TXT,
    A,
    AAAA,
}

/// DNS Source of the external ip
///
/// It expects a DNS server to target for a query (currently only A and TXT), to retrive in the
/// reply of the message the IP.
/// A few services are known for replying with the IP of the query sender.
#[derive(Debug, Clone)]
pub struct DNSSource {
    server: Option<String>, // if not present use the system DNS
    record_type: QueryType,
    record: String,
}

impl DNSSource {
    fn source<R: Into<String>>(
        server: Option<String>,
        record_type: QueryType,
        record: R,
    ) -> Box<dyn Source> {
        Box::new(DNSSource {
            server,
            record_type,
            record: record.into(),
        })
    }
}

impl std::fmt::Display for DNSSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "DnsSource: {:?} {:?} {}",
            self.server, self.record_type, self.record
        )
    }
}

impl DNSSource {
    async fn get_resolver(self: &DNSSource) -> Result<TokioAsyncResolver, Error> {
        let resolver =
            TokioAsyncResolver::tokio(ResolverConfig::default(), ResolverOpts::default());

        if let Some(server) = &self.server {
            let response = resolver.lookup_ip(server.as_str()).await;
            match response.iter().next() {
                None => return Err(Error::DnsResolutionEmpty),
                Some(lookup) => {
                    let ip = lookup.iter().next();
                    match ip {
                        None => return Err(Error::DnsResolutionEmpty),
                        Some(found_ip) => {
                            let mut config = ResolverConfig::new();
                            let address = SocketAddr::new(found_ip, 53);
                            trace!("DNS address {}", address);
                            config.add_name_server(NameServerConfig {
                                bind_addr: None,
                                socket_addr: address,
                                protocol: trust_dns_resolver::config::Protocol::Udp,
                                tls_dns_name: Some(server.clone()),
                                trust_negative_responses: true,
                            });
                            let mut resolver_opts = ResolverOpts::default();
                            resolver_opts.ip_strategy = LookupIpStrategy::Ipv4AndIpv6;
                            return Ok(TokioAsyncResolver::tokio(config, resolver_opts));
                        }
                    }
                }
            }
        }
        Ok(resolver)
    }
}

impl Source for DNSSource {
    fn get_ip<'a>(&'a self, family: Family) -> IpFuture<'a> {
        async fn run(_self: &DNSSource, family: Family) -> IpResult {
            trace!("Contacting {:?} for {}", _self.server, _self.record);
            let resolver = _self.get_resolver().await?;

            match _self.record_type {
                QueryType::TXT => {
                    for reply in resolver.txt_lookup(_self.record.clone()).await?.iter() {
                        for txt in reply.txt_data().iter() {
                            let data = std::str::from_utf8(txt);
                            if data.is_err() {
                                continue;
                            }
                   
                            let ip = data.unwrap().parse()?;
                            if family == Family::Any {
                                return Ok(ip)
                            } else if family == Family::IPv4 {
                                if let IpAddr::V4(_) = ip {
                                    return Ok(ip);
                                }
                                return Err(Error::DnsResolutionEmpty);
                            } else {// if family == Family::IPv6
                                if let IpAddr::V6(_) = ip {
                                    return Ok(ip);
                                }
                                return Err(Error::UnsupportedFamily);
                            }
                        }
                    }
                }
                QueryType::A => {
                    if family == Family::IPv4 || family == Family::Any {
                        for reply in resolver.lookup_ip(_self.record.clone()).await?.iter() {
                            if let IpAddr::V4(_) = reply {
                                return Ok(reply);
                            }
                        }
                    }
                    return Err(Error::UnsupportedFamily)
                }
                QueryType::AAAA => {
                    if family == Family::IPv6 || family == Family::Any {
                        for reply in resolver.lookup_ip(_self.record.clone()).await?.iter() {
                            if let IpAddr::V6(_) = reply {
                                return Ok(reply);
                            }
                        }
                    }
                    return Err(Error::UnsupportedFamily);
                }
            }
            Err(Error::DnsResolutionEmpty)
        }
        Box::pin(run(self, family))
    }

    fn box_clone(&self) -> Box<dyn Source> {
        Box::new(self.clone())
    }
}

/// Returns a collection of DNS sources to use to retrieve the external ip
pub fn get_dns_sources<T>() -> T
where
    T: std::iter::FromIterator<Box<dyn Source>>,
{
    vec![
        DNSSource::source(
            Some(String::from("resolver1.opendns.com")),
            QueryType::A,
            "myip.opendns.com",
        ),
        DNSSource::source(
            Some(String::from("resolver1.opendns.com")),
            QueryType::AAAA,
            "myip.opendns.com",
        ),
        DNSSource::source(None, QueryType::TXT, "o-o.myaddr.l.google.com"),
    ]
    .into_iter()
    .collect()
}
