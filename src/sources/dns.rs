use crate::sources::interfaces::{Error, Family, IpFuture, IpResult, Source};
use log::trace;

use hickory_resolver::config::*;
use hickory_resolver::net::runtime::TokioRuntimeProvider;
use hickory_resolver::proto::rr::RData;
use hickory_resolver::TokioResolver;

#[derive(Debug, Clone, Copy)]
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
    server: String,
    record_type: QueryType,
    record: String,
}

impl DNSSource {
    pub fn new<S: Into<String>, R: Into<String>>(
        server: S,
        record_type: QueryType,
        record: R,
    ) -> Self {
        DNSSource {
            server: server.into(),
            record_type,
            record: record.into(),
        }
    }
    fn source<R: Into<String>>(
        server: String,
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
    async fn get_resolver(self: &DNSSource, family: Family) -> Result<TokioResolver, Error> {
        let mut resolver_opts = ResolverOpts::default();
        resolver_opts.ip_strategy = match family {
            Family::IPv4 => LookupIpStrategy::Ipv4Only,
            Family::IPv6 => LookupIpStrategy::Ipv6Only,
            Family::Any => resolver_opts.ip_strategy,
        };

        trace!("Bootstrapping resolver for {} with strategy {:?}", self.server, resolver_opts.ip_strategy);
        let mut builder = TokioResolver::builder_with_config(
            ResolverConfig::udp_and_tcp(&GOOGLE),
            TokioRuntimeProvider::default(),
        );
        *builder.options_mut() = resolver_opts.clone();
        let resolver = builder.build()?;

        let mut name_servers = Vec::new();
        for found_ip in resolver.lookup_ip(&self.server).await?.iter() {
            trace!("DNS address {}", found_ip);
            name_servers.push(NameServerConfig::udp(found_ip));
        }

        let config = ResolverConfig::from_parts(None, Vec::new(), name_servers);

        let mut builder =
            TokioResolver::builder_with_config(config, TokioRuntimeProvider::default());
        *builder.options_mut() = resolver_opts;
        Ok(builder.build()?)
    }
}

impl Source for DNSSource {
    fn get_ip(&self, family: Family) -> IpFuture<'_> {
        async fn run(_self: &DNSSource, family: Family) -> IpResult {
            if matches!(
                (family, _self.record_type),
                (Family::IPv4, QueryType::AAAA) | (Family::IPv6, QueryType::A)
            ) {
                return Err(Error::UnsupportedFamily);
            }
            trace!("Contacting {:?} for {}", _self.server, _self.record);
            let resolver: TokioResolver = _self
                .get_resolver(match _self.record_type {
                    QueryType::A => Family::IPv4,
                    QueryType::AAAA => Family::IPv6,
                    _ => family,
                })
                .await?;

            match _self.record_type {
                QueryType::TXT => {
                    for reply in resolver.txt_lookup(_self.record.clone()).await?.answers() {
                        if let RData::TXT(txt) = &reply.data {
                            for txt in txt.txt_data.iter() {
                                let data = std::str::from_utf8(txt);
                                if data.is_err() {
                                    continue;
                                }

                                let ip = data.unwrap().parse()?;
                                if family == Family::Any {
                                    return Ok(ip);
                                } else if family == Family::IPv4 {
                                    if ip.is_ipv4() {
                                        return Ok(ip);
                                    }
                                    return Err(Error::DnsResolutionEmpty);
                                } else {
                                    // if family == Family::IPv6
                                    if ip.is_ipv6() {
                                        return Ok(ip);
                                    }
                                    return Err(Error::UnsupportedFamily);
                                }
                            }
                        }
                    }
                }
                QueryType::A => {
                    if family == Family::IPv4 || family == Family::Any {
                        for reply in resolver.lookup_ip(_self.record.clone()).await?.iter() {
                            if reply.is_ipv4() {
                                return Ok(reply);
                            }
                        }
                    }
                    return Err(Error::UnsupportedFamily);
                }
                QueryType::AAAA => {
                    if family == Family::IPv6 || family == Family::Any {
                        for reply in resolver.lookup_ip(_self.record.clone()).await?.iter() {
                            if reply.is_ipv6() {
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
            String::from("resolver1.opendns.com"),
            QueryType::A,
            "myip.opendns.com",
        ),
        DNSSource::source(
            String::from("resolver1.opendns.com"),
            QueryType::AAAA,
            "myip.opendns.com",
        ),
        DNSSource::source(
            String::from("ns1.google.com"),
            QueryType::TXT,
            "o-o.myaddr.l.google.com",
        ),
    ]
    .into_iter()
    .collect()
}
