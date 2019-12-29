use crate::sources::interfaces::{Error, IpFuture, IpResult, Source};
use std::net::IpAddr;
use log::trace;

#[derive(Debug, Clone)]
pub enum QueryType {
    TXT,
    A,
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
    fn source<S: Into<String>, R: Into<String>>(
        server: S,
        record_type: QueryType,
        record: R,
    ) -> Box<dyn Source> {
        Box::new(DNSSource {
            server: server.into(),
            record_type: record_type,
            record: record.into(),
        })
    }
}

impl std::fmt::Display for DNSSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "DnsSource: {} {:?} {}",
            self.server, self.record_type, self.record
        )
    }
}

/// Used internally to resolve the IP of the target DNS servers
async fn resolve_server(ares: &c_ares_resolver::FutureResolver, server: &str) -> Result<String, Error> {
    for query in ares
        .query_a(server.into())
        .await
        .map_err(|x| Error::Dns(c_ares_resolver::Error::Ares(x)))?
        .iter()
    {
        return Ok(query.ipv4().to_string());
    }
    Err(Error::DnsResolutionEmpty)
}

impl Source for DNSSource {
    fn get_ip<'a>(&'a self) -> IpFuture<'a> {
        async fn run(_self: &DNSSource) -> IpResult {
            trace!("Contacting {} for {}", _self.server, _self.record);
            let ares = c_ares_resolver::FutureResolver::new().map_err(Error::Dns)?;

            // Resolve DNS Server name
            let server = resolve_server(&ares, &_self.server).await?;
            trace!("DNS IP {}", server);

            let ares = ares
                .set_servers(&[&server])
                .map_err(|x| Error::Dns(c_ares_resolver::Error::Ares(x)))?;

            match _self.record_type {
                QueryType::TXT => {
                    for query in ares
                        .query_txt(&_self.record)
                        .await
                        .map_err(|x| Error::Dns(c_ares_resolver::Error::Ares(x)))?
                        .iter()
                    {
                        let data = std::str::from_utf8(query.text());
                        if data.is_err() {
                            continue;
                        }
                        return Ok(data.unwrap().parse().map_err(Error::InvalidAddress)?);
                    }
                }
                QueryType::A => {
                    for query in ares
                        .query_a(&_self.record)
                        .await
                        .map_err(|x| Error::Dns(c_ares_resolver::Error::Ares(x)))?
                        .iter()
                    {
                        return Ok(IpAddr::V4(query.ipv4()));
                    }
                }
            }
            Err(Error::DnsResolutionEmpty)
        };
        Box::pin(run(self))
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
        DNSSource::source("resolver1.opendns.com", QueryType::A, "myip.opendns.com"),
        DNSSource::source("ns1.google.com", QueryType::TXT, "o-o.myaddr.l.google.com"),
        DNSSource::source("ns2.google.com", QueryType::TXT, "o-o.myaddr.l.google.com"),
        DNSSource::source("ns3.google.com", QueryType::TXT, "o-o.myaddr.l.google.com"),
        DNSSource::source("ns4.google.com", QueryType::TXT, "o-o.myaddr.l.google.com"),
    ]
    .into_iter()
    .collect()
}
